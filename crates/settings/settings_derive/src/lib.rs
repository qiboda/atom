use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Data, DeriveInput, Ident, Result, Token,
};

#[proc_macro_derive(Setting)]
pub fn derive_layertag(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let ident = derive_input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let ts = quote!(
        impl #impl_generics settings::Setting for #ident #ty_generics #where_clause {}
    );

    ts.into()
}
