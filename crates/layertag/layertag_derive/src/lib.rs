use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Attribute, Ident, Result, Token, parse_macro_input, DeriveInput,
};

#[proc_macro_derive(LayerTag)]
pub fn derive_layertag(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let ident = derive_input.ident;
    let tags = get_layer_tag_attribute(&derive_input.attrs);
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let ts = quote!(
        impl #impl_generics layertag::layertag::LayerTag for #ident #ty_generics #where_clause {
            fn tag(&self) -> &[layertag::tag::Tag] {
                static CELL:once_cell::sync::OnceCell<Vec<layertag::tag::Tag>> = once_cell::sync::OnceCell::new();
                CELL.get_or_init(||{
                    vec![#tags]
                }).as_slice()
            }
        }
    );

    ts.into()
}

#[proc_macro_attribute]
pub fn layer_tag(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

enum Arg {
    Str(String),
    Ident(Ident),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let ident = input.parse::<Ident>()?;
            Ok(Arg::Ident(ident))
        } else if lookahead.peek(syn::LitStr) {
            let lit_str = input.parse::<syn::LitStr>()?;
            Ok(Arg::Str(lit_str.value()))
        } else {
            Err(lookahead.error())
        }
    }
}

struct Args {
    args: Punctuated<Arg, Token![,]>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::parse_terminated(input)?;
        Ok(Args { args })
    }
}

fn get_layer_tag_attribute(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let mut tags: proc_macro2::TokenStream = proc_macro2::TokenStream::new();
    for attr in attrs.iter() {
        if let syn::Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("layer_tag") {
                let parser = Args::parse;
                if let Ok(Args { args }) = parser.parse(meta_list.tokens.clone().into()) {
                    for arg in args.iter() {
                        match arg {
                            Arg::Str(s) => {
                                tags.extend::<proc_macro2::TokenStream>(quote!(
                                    layertag::tag::Tag::new(#s),
                                ));
                            }
                            Arg::Ident(ident) => {
                                tags.extend::<proc_macro2::TokenStream>(quote!(
                                    layertag::tag::Tag::new(#ident),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    tags
}
