/// 用于宏构建结构体成员。
#[allow(unused_macros)]
macro_rules! impl_extra {
    ( @ $name:ident { } -> ($($result:tt)*) ) => (
        pub struct $name {
            $($result)*
        }
    );

    ( @ $name:ident { $param:ident : Option<$type:ty>, $($rest:tt)* } -> ($($result:tt)*) ) => (
        impl_extra!(@ $name { $($rest)* } -> (
            $($result)*
            pub $param : Option<$type>,
        ));
    );

    ( @ $name:ident { $param:ident : Vec<$type:ty>, $($rest:tt)* } -> ($($result:tt)*) ) => (
        impl_extra!(@ $name { $($rest)* } -> (
            $($result)*
            pub $param : Vec<$type>,
        ));
    );

     ( @ $name:ident { $param:ident : $default:tt, $($rest:tt)* } -> ($($result:tt)*) ) => (
        impl_extra!(@ $name { $($rest)* } -> (
            $($result)*
            pub $param : $default,
        ));
    );

    ( $name:ident { $( $param:ident  ($($type:tt)*) ),* $(,)? } ) => (
        impl_extra!(@ $name { $($param : $($type)*,)* } -> ());
    );
}

/// 为指定 shader 类型生成「shader 资源结构体 + 加载插件」的声明宏。
///
/// 展开后生成：
/// - `{Module}{Type}Shaders`：包含各 shader [`Handle<Shader>`](bevy::asset::Handle) 字段的资源结构体，
///   字段名由 `member_name` 指定，路径为 `shaders_path`；
/// - `{Module}{Type}ShadersPlugin`：Bevy 插件，在构建时通过
///   [`DirectAssetAccessExt`](bevy::asset::DirectAssetAccessExt) 加载全部 shader 并注册为
///   [`ExtractResource`](bevy::render::extract_resource::ExtractResource)，供渲染端直接使用。
///
/// 示例：
/// ```ignore
/// shaders_plugin!(Atom, Noise, (fbm_shader -> "shaders/noise/fbm.wgsl"));
/// ```
#[macro_export]
macro_rules! shaders_plugin {
    (
        $module_name: ident,
        $shader_type:ident,
        ($($member_name:ident -> $shaders_path:expr),*)
    ) => {
        paste::paste! {
            shaders_plugin!(_construct -> [<$module_name $shader_type Shaders>] (
                $($member_name),*
            ));

            #[derive(Debug, Default)]
            #[doc = concat!("`", stringify!([<$module_name $shader_type ShadersPlugin>]), "` 插件：在 `build` 时加载全部对应 shader 资源并注册为 `ExtractResource`，供渲染端直接读取。" )]
            pub struct [<$module_name $shader_type ShadersPlugin>];

            impl bevy::app::Plugin for [<$module_name $shader_type ShadersPlugin>] {
                fn build(&self, app: &mut bevy::prelude::App) {
                    use bevy::asset::DirectAssetAccessExt;
                    app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<[<$module_name $shader_type Shaders>]>::default());
                    let world = app.world();
                    app.insert_resource(
                        shaders_plugin!(_init -> world, [<$module_name $shader_type Shaders>] ( $($member_name, $shaders_path),* ))
                    );
                }
            }
        }
    };
    // 参考了impl_extra!宏的实现
    (_init -> $world: ident, $name:ident ( $($member_name: ident, $shaders_path: expr),* ) ) => (
        $name {
            $(
                $member_name: $world.load_asset($shaders_path),
            )*
        }
    );
    // 参考了impl_extra!宏的实现
    (_construct -> $name:ident ( $($member_name: ident),* ) ) => (
        #[derive(Debug, Default, Clone, bevy::prelude::Resource, bevy::prelude::Reflect, bevy::render::extract_resource::ExtractResource)]
        #[doc = concat!("`", stringify!($name), "` 资源结构体：持有该类 shader 的全部 `Handle<Shader>` 句柄。")]
        pub struct $name {
            $(
                #[doc = concat!("`", stringify!($member_name), "` shader 的资源句柄。")]
                pub $member_name: bevy::asset::Handle<bevy::shader::Shader>,

            )*
        }
    )
}

macro_rules! atom_shaders_plugin {
    (
        $shader_type:ident,
        ($($member_name:ident -> $shaders_path:expr),*)
    ) => {
        shaders_plugin!(Atom, $shader_type, ($($member_name -> $shaders_path),*));
    }
}

/// 同一个crate中跨文件共享 macro的方法
pub(crate) use atom_shaders_plugin;
