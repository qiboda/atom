use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Data, DeriveInput, Ident, Result, Token,
};

#[proc_macro_derive(LayerTag, attributes(layer_tag, layer_tag_counter))]
pub fn derive_layertag(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let ident = derive_input.ident.clone();
    let tags = get_layer_tag_attribute(&derive_input.attrs);
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let counter = get_layer_tag_counter_attribute(&derive_input);
    let data = impl_layer_tag_data(&derive_input);

    let ts = quote!(
        impl #impl_generics layertag::layertag::LayerTagClone for #ident #ty_generics #where_clause {
            fn box_clone(&self) -> Box<dyn layertag::layertag::LayerTag> {
                Box::new(self.clone())
            }
        }

        impl #impl_generics layertag::layertag::LayerTag for #ident #ty_generics #where_clause {
            fn tag(&self) -> &[layertag::tag::Tag] {
                static CELL:once_cell::sync::OnceCell<Vec<layertag::tag::Tag>> = once_cell::sync::OnceCell::new();
                CELL.get_or_init(||{
                    vec![#tags]
                }).as_slice()
            }
        }

        #counter

        #data
    );

    ts.into()
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

fn get_layer_tag_counter_attribute(derive_input: &DeriveInput) -> proc_macro2::TokenStream {
    let mut counter: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    let ident = derive_input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    if let Data::Struct(data) = &derive_input.data {
        data.fields.iter().for_each(|field| {
            if let Some(field_ident) = &field.ident {
                let attrs = &field.attrs;
                for attr in attrs {
                    if let syn::Meta::Path(path) = &attr.meta {
                        if path.is_ident("layer_tag_counter") {
                            counter.extend::<proc_macro2::TokenStream>(quote!(
                                impl #impl_generics layertag::layertag::LayerTagCounter for #ident #ty_generics #where_clause {
                                    fn increase_count(&mut self) {
                                        self.#field_ident += 1;
                                    }

                                    fn decrease_count(&mut self) {
                                        self.#field_ident -= 1;
                                    }

                                    fn count(&self) -> usize {
                                        self.#field_ident
                                    }

                                    fn reset_count(&mut self) {
                                        self.#field_ident = 0;
                                    }
                                }
                            ));
                        }
                    }
                }
            }
        });
    }

    if counter.is_empty() {
        counter.extend::<proc_macro2::TokenStream>(quote!(
            impl #impl_generics layertag::layertag::LayerTagCounter for #ident #ty_generics #where_clause {
                fn increase_count(&mut self) {}

                fn decrease_count(&mut self) {}

                fn count(&self) -> usize { 0 }

                fn reset_count(&mut self) {}
            }
        ));
    }

    counter
}

fn impl_layer_tag_data(derive_input: &DeriveInput) -> proc_macro2::TokenStream {
    let mut data: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    let ident = derive_input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let mut non_counter_field_count = 0;
    if let Data::Struct(data) = &derive_input.data {
        data.fields.iter().for_each(|field| {
            non_counter_field_count += 1;
            let attrs = &field.attrs;
            for attr in attrs {
                if let syn::Meta::Path(path) = &attr.meta {
                    if path.is_ident("layer_tag_counter") {
                        non_counter_field_count -= 1;
                    }
                }
            }
        });
    }

    if non_counter_field_count <= 0 {
        data.extend::<proc_macro2::TokenStream>(quote!(
            impl #impl_generics layertag::layertag::LayerTagData for #ident #ty_generics #where_clause {
                fn cmp_data_same_type_inner(&self, _rhs: &dyn layertag::layertag::LayerTag) -> bool {
                    true
                }
            }
        ));
    }

    data
}
