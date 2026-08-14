//! `atom_data` 的 derive 宏：`DataAsset` —— 为行类型（Bean）生成索引系统。
//!
//! 权威 spec：`.dsh/plans/atom-data.md` §4（issue #3 Batch 1，任务 B1-4）+ §5（issue #4
//! Batch 2，任务 B2-2）。关键决策：D1（行类型 + `DataTable<T>` 泛型表容器）、D2（索引形态表）、
//! D3（`data_ref` 跨表引用）。
//!
//! [`DataAsset`] 解析 `#[index(...)]` helper attribute，生成 4 组代码：
//!
//! 1. **索引容器** `{Row}Index`（`HashMap<K, usize>` 族，multi 为 `HashMap<K, Vec<usize>>`）
//!    + `DataIndexed`/`DataIndex` impl——`build` 遍历行构建映射，唯一索引重复键报错；
//!      Batch 2 起含主索引查询 `get` 与 `PrimaryKey`/`primary_key`。
//! 2. **本地查询 trait** `{Row}Queries` + `DataTable<{Row}>` impl——孤儿规则解法：
//!    `DataTable<T>` 的 inherent impl 只能在 `atom_data` crate 内，宏生成代码位于用户 crate，
//!    因此查询方法必须经**本地生成的 trait** 提供（trait 与行类型同模块声明，方法调用自动在 scope）。
//! 3. **`TypePath` impl**——`DataTable<T>: Asset` 的 TypePath derive 会给 `T` 加 `TypePath` bound，
//!    因此行类型必须实现 `TypePath`，由宏代生成。
//! 4. **行类型 inherent impl**（Batch 2）：主索引提取 `primary_key`（宏生成方法调用无需
//!    导入 `DataIndexed` trait——测试契约锁定）+ `#[data_ref(...)]` 字段的 `resolve_{field}`
//!    惰性跨表引用解析方法。
//!
//! 无任何 `#[index]` 的行类型：仍生成 `DataIndexed`（`type Index = ()`、`PrimaryKey = ()`）+
//! `DataIndex for ()`（`get` 返回 None），不生成查询 trait（仅 `iter()` 全量迭代）。

#![deny(missing_docs)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Fields, FieldsNamed, Ident, Lit,
    Result as SynResult, Type, parse_macro_input, spanned::Spanned,
};

/// 单个 `#[index(...)]` 声明的解析结果。
struct IndexSpec {
    /// 键形态（单字段名 / 复合字段名列表）。
    key: IndexKey,
    /// 多值索引（`#[index(key = "x", multi)]`）。
    multi: bool,
}

/// 索引键形态。
enum IndexKey {
    /// `key = "field"`
    Single(String),
    /// `key = ("a", "b")`——2 元组复合键
    Composite(Vec<String>),
}

/// 单个 `#[data_ref(...)]` 字段声明的解析结果（Batch 2，D3）。
struct DataRefSpec {
    /// 目标行类型（`table = "X"` 或 `"crate::foo::X"` 字符串字面量解析为 Path）。
    table: syn::Path,
    /// 声明字段名（保留 `r#` 前缀）。
    field: Ident,
    /// 目标索引名（本批仅主索引；保留用于文档/校验，实际查询用目标表主索引）。
    key: String,
}

/// 已解析并绑定到结构体字段的索引。
struct ResolvedIndex {
    /// 命名基础名：单键=剥 `r#` 的字段名；复合键=`"pair"`（`by_{base}` / `get_by_{base}`）。
    base: String,
    /// 键字段（单键 1 个；复合键 2 个），保留 `r#` 前缀（如 `r#type`）。
    fields: Vec<Ident>,
    /// 键类型：单键=字段类型；复合键=`(A, B)` 元组。
    key_ty: Type,
    /// 多值索引（容器字段 `HashMap<K, Vec<usize>>`）。
    multi: bool,
    /// 主索引（第一个非 multi 单键）→ 方法名 `get`。
    is_primary: bool,
}

impl ResolvedIndex {
    /// 容器字段名：`by_{base}`（`by_id` / `by_pair` / `by_type`）。
    fn map_field(&self) -> Ident {
        format_ident!("by_{}", self.base)
    }

    /// 查询方法名：主索引 `get`；次单键 `get_by_{base}`；复合键 `get_by_pair`；multi `get_all_by_{base}`。
    fn method(&self) -> Ident {
        if self.is_primary {
            format_ident!("get")
        } else if self.multi {
            format_ident!("get_all_by_{}", self.base)
        } else {
            format_ident!("get_by_{}", self.base)
        }
    }

    /// key 提取表达式：单键 `row.{field}.to_owned()`；复合键 `(row.a.to_owned(), row.b.to_owned())`。
    /// 统一 `to_owned` 兼容 Copy（i32）与非 Copy（String）键类型，且不触发 clippy `clone_on_copy`。
    fn key_expr(&self) -> TokenStream2 {
        let idents = self.fields.iter().map(|f| quote!(row.#f.to_owned()));
        quote! { (#(#idents),*) }
    }

    /// 查询方法参数类型（`&K`；键类型为 `String` 时用 `&T where T: AsRef<str>`）：
    /// clippy `ptr_arg` 禁止 `&String` 参数（`&K` 会触发），而具体 `&str` 会让测试调用点的
    /// `&"...".to_string()` 触发 `unnecessary_to_owned`（测试为 RED 契约不可改）。
    /// 泛型 `AsRef<str>` 对两类调用点均 clippy 干净。
    fn key_param(&self) -> TokenStream2 {
        if self.fields.len() == 1 && is_string_ty(&self.key_ty) {
            quote!(&T)
        } else {
            let key_ty = &self.key_ty;
            quote!(&#key_ty)
        }
    }

    /// 查询方法泛型参数声明（仅 String 键索引需要）。
    fn key_generics(&self) -> TokenStream2 {
        if self.fields.len() == 1 && is_string_ty(&self.key_ty) {
            quote!(<T: ::core::convert::AsRef<str> + ?Sized>)
        } else {
            quote!()
        }
    }

    /// 查询方法体内 key 访问表达式（String 键经 `as_ref()` 查 `HashMap<String, _>`）。
    fn key_lookup(&self) -> TokenStream2 {
        if self.fields.len() == 1 && is_string_ty(&self.key_ty) {
            quote!(key.as_ref())
        } else {
            quote!(key)
        }
    }

    /// 索引字段名（错误信息用）。
    fn field_label(&self) -> String {
        self.base.clone()
    }
}

/// 键类型是否为 `String`（`std::string::String` 路径末段）。
fn is_string_ty(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "String"),
        _ => false,
    }
}

/// `DataAsset` derive 入口。
///
/// 用于行类型（Bean），生成索引容器 + `DataIndexed`/`DataIndex` impl + 本地查询 trait +
/// `TypePath` impl + 主索引提取/跨表引用解析 inherent impl。
#[proc_macro_derive(DataAsset, attributes(index, data_ref))]
pub fn derive_data_asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> SynResult<TokenStream2> {
    let name = &input.ident;
    let data = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "`DataAsset` 仅支持具名字段结构体（named-field struct）",
            ));
        }
    };

    let specs = parse_index_attrs(&input.attrs)?;
    let data_ref_specs = parse_data_ref_attrs(data)?;
    let type_path = gen_type_path(name);
    let resolve_impl = gen_resolve_impl(name, &data_ref_specs);
    let primary_key_impl = gen_primary_key_impl(name, quote!(()), quote!());

    if specs.is_empty() {
        // 无索引：unit 索引 + DataIndexed/DataIndex impl，无查询 trait（仅 iter()）。
        return Ok(quote! {
            impl ::atom_data::DataIndexed for #name {
                type Index = ();
                type PrimaryKey = ();

                fn primary_key(&self) -> Self::PrimaryKey {}
            }

            impl ::atom_data::DataIndex<#name> for () {
                fn build(_rows: &[#name]) -> Result<Self, String> {
                    Ok(())
                }

                fn get<'a>(&self, rows: &'a [#name], key: &<#name as ::atom_data::DataIndexed>::PrimaryKey) -> Option<&'a #name> {
                    let _ = key;
                    None
                }
            }

            #primary_key_impl

            #resolve_impl

            #type_path
        });
    }

    let resolved = resolve_indexes(data, &specs)?;
    let primary = resolved.iter().find(|r| r.is_primary);
    let index_name = format_ident!("{}Index", name);
    let trait_name = format_ident!("{}Queries", name);

    let (primary_key_ty, primary_key_expr, get_body) = match primary {
        Some(r) => {
            let ty = &r.key_ty;
            let field = &r.fields[0];
            let map_field = r.map_field();
            (
                quote!(#ty),
                quote!(self.#field.to_owned()),
                quote! {
                    let idx = *self.#map_field.get(key)?;
                    rows.get(idx)
                },
            )
        }
        None => (
            quote!(()),
            quote!(),
            quote! {
                let _ = key;
                None
            },
        ),
    };
    let primary_key_impl =
        gen_primary_key_impl(name, primary_key_ty.clone(), primary_key_expr.clone());

    let container_fields = resolved
        .iter()
        .map(|r| {
            let field = r.map_field();
            let key_ty = &r.key_ty;
            let value_ty = if r.multi {
                quote!(::std::vec::Vec<usize>)
            } else {
                quote!(usize)
            };
            quote! {
                #field: ::std::collections::HashMap<#key_ty, #value_ty>,
            }
        })
        .collect::<Vec<_>>();

    let map_inits = resolved
        .iter()
        .map(|r| {
            let field = r.map_field();
            let key_ty = &r.key_ty;
            let value_ty = if r.multi {
                quote!(::std::vec::Vec<usize>)
            } else {
                quote!(usize)
            };
            quote! {
                let mut #field: ::std::collections::HashMap<#key_ty, #value_ty> =
                    ::std::collections::HashMap::new();
            }
        })
        .collect::<Vec<_>>();

    let build_stmts = resolved
        .iter()
        .map(|r| {
            let field = r.map_field();
            let key_expr = r.key_expr();
            let label = r.field_label();
            if r.multi {
                // 多值索引：一 key 多行，不报错。
                quote! {
                    #field.entry(#key_expr).or_default().push(row_idx);
                }
            } else {
                // 唯一索引：重复 key → error 分支（D2 锁定，拒绝静默 last-wins）。
                quote! {
                    let key = #key_expr;
                    match #field.entry(key) {
                        ::std::collections::hash_map::Entry::Occupied(_) => {
                            return Err(format!(
                                "duplicate key {:?} for index \"{}\"",
                                #key_expr,
                                #label,
                            ));
                        }
                        ::std::collections::hash_map::Entry::Vacant(slot) => {
                            slot.insert(row_idx);
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let map_idents = resolved
        .iter()
        .map(ResolvedIndex::map_field)
        .collect::<Vec<_>>();

    let method_sigs = resolved
        .iter()
        .map(|r| {
            let method = r.method();
            let generics = r.key_generics();
            let key_param = r.key_param();
            if r.multi {
                quote! {
                    fn #method #generics(&self, key: #key_param) -> ::std::vec::Vec<&#name>;
                }
            } else {
                quote! {
                    fn #method #generics(&self, key: #key_param) -> Option<&#name>;
                }
            }
        })
        .collect::<Vec<_>>();

    let method_bodies = resolved
        .iter()
        .map(|r| {
            let method = r.method();
            let generics = r.key_generics();
            let key_param = r.key_param();
            let key_lookup = r.key_lookup();
            let field = r.map_field();
            if r.multi {
                quote! {
                    fn #method #generics(&self, key: #key_param) -> ::std::vec::Vec<&#name> {
                        self.index()
                            .#field
                            .get(#key_lookup)
                            .map(|idxs| idxs.iter().filter_map(|i| self.rows().get(*i)).collect())
                            .unwrap_or_default()
                    }
                }
            } else {
                quote! {
                    fn #method #generics(&self, key: #key_param) -> Option<&#name> {
                        let idx = *self.index().#field.get(#key_lookup)?;
                        self.rows().get(idx)
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        /// 索引容器：宏生成（`#[derive(DataAsset)]` 的 `#[index(...)]` 属性驱动）。
        /// `Clone`：`DataTable<T>` 的 `Clone` derive（B2-1 集成层）要求 `T::Index: Clone`。
        ///
        /// 必须 `pub`：`DataIndexed::Index` 是公开关联类型（pub trait），私有 struct 会让
        /// **pub 行类型** 触发 E0446（`private type in public interface`，rustc 1.95 为
        /// 硬错误，allow 无法降级）——atom_ability Batch 3 集成实证（见 TENSIONS.md）；
        /// `{Row}Queries` trait 仍保持私有（测试契约锁定，跨 crate 只能走公共 API）。
        #[derive(Debug, Default, Clone)]
        pub struct #index_name {
            #(#container_fields)*
        }

        impl ::atom_data::DataIndexed for #name {
            type Index = #index_name;
            type PrimaryKey = #primary_key_ty;

            fn primary_key(&self) -> Self::PrimaryKey {
                #primary_key_expr
            }
        }

        impl ::atom_data::DataIndex<#name> for #index_name {
            fn build(rows: &[#name]) -> Result<Self, String> {
                #(#map_inits)*
                for (row_idx, row) in rows.iter().enumerate() {
                    #(#build_stmts)*
                }
                Ok(Self {
                    #(#map_idents,)*
                })
            }

            fn get<'a>(&self, rows: &'a [#name], key: &<#name as ::atom_data::DataIndexed>::PrimaryKey) -> Option<&'a #name> {
                #get_body
            }
        }

        /// 查询 trait：宏生成（孤儿规则解法——`DataTable<T>` 的 inherent impl 只能在 `atom_data` crate）。
        trait #trait_name {
            #(#method_sigs)*
        }

        impl #trait_name for ::atom_data::DataTable<#name> {
            #(#method_bodies)*
        }

        #primary_key_impl

        #resolve_impl

        #type_path
    })
}

/// 解析全部 `#[index(...)]` helper attributes。
fn parse_index_attrs(attrs: &[Attribute]) -> SynResult<Vec<IndexSpec>> {
    let mut specs = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("index")) {
        let mut key: Option<IndexKey> = None;
        let mut multi = false;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                let expr: Expr = meta.value()?.parse()?;
                key = Some(match expr {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) => IndexKey::Single(s.value()),
                    Expr::Tuple(tuple) => {
                        let mut names = Vec::new();
                        for elem in tuple.elems {
                            match elem {
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(s), ..
                                }) => names.push(s.value()),
                                _ => {
                                    return Err(
                                        meta.error("`index` 的 key 元组元素必须是字符串字面量")
                                    );
                                }
                            }
                        }
                        IndexKey::Composite(names)
                    }
                    _ => return Err(meta.error("`index` 的 key 必须是字符串字面量或字符串元组")),
                });
                Ok(())
            } else if meta.path.is_ident("multi") {
                multi = true;
                Ok(())
            } else {
                Err(meta.error("未知的 `index` 属性（仅支持 `key` 与 `multi`）"))
            }
        })?;
        let key = key.ok_or_else(|| syn::Error::new(attr.span(), "`#[index]` 缺少 `key` 属性"))?;
        specs.push(IndexSpec { key, multi });
    }
    Ok(specs)
}

/// 将索引声明绑定到结构体字段，解析键类型。
fn resolve_indexes(data: &FieldsNamed, specs: &[IndexSpec]) -> SynResult<Vec<ResolvedIndex>> {
    let mut resolved = Vec::new();
    let mut composite_count = 0usize;
    let mut primary_claimed = false;

    for spec in specs {
        match &spec.key {
            IndexKey::Single(name) => {
                let field = find_field(data, name)?;
                let fields = vec![field.ident.clone().expect("具名字段应有 ident")];
                let is_primary = !primary_claimed && !spec.multi;
                if is_primary {
                    primary_claimed = true;
                }
                resolved.push(ResolvedIndex {
                    base: name.clone(),
                    fields,
                    key_ty: field.ty.clone(),
                    multi: spec.multi,
                    is_primary,
                });
            }
            IndexKey::Composite(names) => {
                if spec.multi {
                    return Err(syn::Error::new_spanned(
                        data,
                        "多值复合索引暂不支持（`multi` 仅用于单键索引）",
                    ));
                }
                composite_count += 1;
                if composite_count > 1 {
                    return Err(syn::Error::new_spanned(
                        data,
                        "仅支持一个复合索引（`get_by_pair` 命名锁定；多复合键需重新评估命名）",
                    ));
                }
                let mut fields = Vec::new();
                let mut key_tys = Vec::new();
                for name in names {
                    let field = find_field(data, name)?;
                    fields.push(field.ident.clone().expect("具名字段应有 ident"));
                    key_tys.push(field.ty.clone());
                }
                let key_ty = Type::Tuple(syn::TypeTuple {
                    paren_token: Default::default(),
                    elems: key_tys.into_iter().collect(),
                });
                resolved.push(ResolvedIndex {
                    base: "pair".to_string(),
                    fields,
                    key_ty,
                    multi: false,
                    is_primary: false, // 复合键不做主索引（主索引=单键，测试契约 `get`）
                });
            }
        }
    }
    Ok(resolved)
}

/// 按字段名查找字段（剥 `r#` 前缀后匹配），未找到报错。
fn find_field<'a>(data: &'a FieldsNamed, name: &str) -> SynResult<&'a syn::Field> {
    data.named
        .iter()
        .find(|f| strip_raw(f) == name)
        .ok_or_else(|| {
            syn::Error::new_spanned(data, format!("`#[index]` 引用了不存在的字段 `{name}`"))
        })
}

/// 字段名剥 `r#` 前缀（`r#type` → `type`），用于 `#[index(key = "type")]` 匹配。
fn strip_raw(field: &syn::Field) -> String {
    field
        .ident
        .as_ref()
        .expect("具名字段应有 ident")
        .to_string()
        .trim_start_matches("r#")
        .to_string()
}

/// 解析全部 `#[data_ref(...)]` helper attributes（字段级，Batch 2 / D3）。
fn parse_data_ref_attrs(fields: &FieldsNamed) -> SynResult<Vec<DataRefSpec>> {
    let mut specs = Vec::new();
    for field in &fields.named {
        for attr in field.attrs.iter().filter(|a| a.path().is_ident("data_ref")) {
            let mut table: Option<syn::Path> = None;
            let mut key: Option<String> = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table") {
                    let expr: Expr = meta.value()?.parse()?;
                    let lit = match expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => s,
                        _ => return Err(meta.error("`data_ref` 的 table 必须是字符串字面量")),
                    };
                    table = Some(syn::parse_str::<syn::Path>(&lit.value()).map_err(|_| {
                        meta.error(
                            "`data_ref` 的 table 必须是合法的类型路径（如 \"LayerTagConfig\"）",
                        )
                    })?);
                    Ok(())
                } else if meta.path.is_ident("key") {
                    let expr: Expr = meta.value()?.parse()?;
                    key = Some(match expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => {
                            let value = s.value();
                            // key 是字段名（标识符）——非法标识符直接编译错误，
                            // 避免静默忽略后解析行为与声明不一致（QA 发现 #2）。
                            syn::parse_str::<Ident>(&value).map_err(|_| {
                                meta.error("`data_ref` 的 key 必须是字段名字符串（合法标识符）")
                            })?;
                            value
                        }
                        _ => return Err(meta.error("`data_ref` 的 key 必须是字符串字面量")),
                    });
                    Ok(())
                } else {
                    Err(meta.error("未知的 `data_ref` 属性（仅支持 `table` 与 `key`）"))
                }
            })?;
            let table = table
                .ok_or_else(|| syn::Error::new(attr.span(), "`#[data_ref]` 缺少 `table` 属性"))?;
            let key =
                key.ok_or_else(|| syn::Error::new(attr.span(), "`#[data_ref]` 缺少 `key` 属性"))?;
            specs.push(DataRefSpec {
                table,
                field: field.ident.clone().expect("具名字段应有 ident"),
                key,
            });
        }
    }
    Ok(specs)
}

/// 生成行类型 inherent `primary_key` 方法。
///
/// 必须生成 inherent 方法（而非仅 trait impl）：测试契约在**未导入 `DataIndexed` trait** 的
/// 模块内直接调用 `row.primary_key()`——trait 方法需要 trait 在 scope，inherent 方法无需；
/// 二者同名共存时方法解析优先 inherent，不冲突。
fn gen_primary_key_impl(name: &Ident, ty: TokenStream2, expr: TokenStream2) -> TokenStream2 {
    quote! {
        impl #name {
            /// 主索引键提取（宏生成；无主索引行类型返回 `()`）。
            pub fn primary_key(&self) -> #ty {
                #expr
            }
        }
    }
}

/// 生成 `resolve_{field}` 惰性跨表引用解析方法（D3）。
///
/// 语义（测试锁定）：目标表未加载 → `None`；已加载 → 逐键解析，键缺失/转换失败跳过，
/// 保持引用列表顺序，全部缺失 → `Some(空)`。输出生命周期**显式绑定 registry 参数**——
/// 不能依赖 elision 绑 `&self`（否则解析出的引用借用 self 而非 registry，调用点 borrow-check
/// 失败，test-agent 已实证）。
///
/// 键转换走 [`DataRefKey`](atom_data::DataRefKey)（blanket impl 覆盖 `FromStr` 键），
/// 方法 where 子句要求目标表主键实现它——目标表无主索引（`PrimaryKey = ()`）时报错
/// 指向本 trait 而非 `FromStr`，诊断更可读（QA 发现 #3）。
fn gen_resolve_impl(name: &Ident, specs: &[DataRefSpec]) -> TokenStream2 {
    if specs.is_empty() {
        return TokenStream2::new();
    }
    let methods = specs.iter().map(|spec| {
        let field = &spec.field;
        let table = &spec.table;
        let key = &spec.key;
        let method = format_ident!("resolve_{}", strip_raw_ident(field));
        // quote! 的 `///` 注释内不做 `#` 插值——doc 属性须用 `#[doc = #doc]` 显式构造。
        let doc = format!(
            "解析跨表引用（惰性：目标表未加载 → None；已加载 → 逐键解析，键缺失/转换失败跳过）。\n\
             目标索引：`{key}`（**仅文档标记**——`data_ref` 解析始终走目标表主索引，\n\
             声明非主索引字段名不会改变解析行为）。\n\
             约束：目标表主键类型必须实现 `atom_data::DataRefKey`（`FromStr` 键自动满足）。"
        );
        quote! {
            #[doc = #doc]
            pub fn #method<'a>(
                &self,
                registry: &'a ::atom_data::DataRegistry,
            ) -> Option<::std::vec::Vec<&'a #table>>
            where
                <#table as ::atom_data::DataIndexed>::PrimaryKey: ::atom_data::DataRefKey,
            {
                let table = registry.table::<#table>()?;
                Some(
                    self.#field
                        .iter()
                        .filter_map(|k| {
                            let pk: <#table as ::atom_data::DataIndexed>::PrimaryKey =
                                ::atom_data::DataRefKey::from_ref_str(k)?;
                            table.get_primary(&pk)
                        })
                        .collect::<::std::vec::Vec<_>>(),
                )
            }
        }
    });
    quote! {
        impl #name {
            #(#methods)*
        }
    }
}

/// 标识符剥 `r#` 前缀（`r#type` → `type`），用于方法名生成。
fn strip_raw_ident(ident: &Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_string()
}

/// TypePath impl：`DataTable<T>: Asset` 的前提（TypePath derive 给 `T` 加 `TypePath` bound）。
fn gen_type_path(name: &Ident) -> TokenStream2 {
    quote! {
        impl ::atom_data::TypePath for #name {
            fn type_path() -> &'static str {
                concat!(module_path!(), "::", stringify!(#name))
            }

            fn short_type_path() -> &'static str {
                stringify!(#name)
            }
        }
    }
}
