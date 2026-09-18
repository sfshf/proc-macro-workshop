use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Field, Fields, GenericParam, LitStr, Meta};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let name = ident.to_string();
    // generic types
    let mut generics = input.generics.clone();
    for param in &mut generics.params {
        match param {
            GenericParam::Lifetime(_) => {
                unimplemented!()
            }
            GenericParam::Const(_) => {
                unimplemented!()
            }
            GenericParam::Type(type_param) => {
                type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
            }
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut debug_fields: Vec<proc_macro2::TokenStream> = Vec::new();
    // input data type
    match &input.data {
        Data::Union(_) => {
            unimplemented!()
        }
        Data::Enum(_) => {
            unimplemented!()
        }
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(_) => {
                unimplemented!()
            }
            Fields::Unit => {
                unimplemented!()
            }
            Fields::Named(fields) => {
                for field in fields.named.iter() {
                    let field_name = &field.ident;
                    let field_name_str = field_name.as_ref().unwrap().to_string();
                    match get_debug_attribute(field) {
                        Some(format_str) => {
                            debug_fields.push(quote! {
                                .field(#field_name_str, &format_args!(#format_str, &self.#field_name))
                            });
                        }
                        None => {
                            debug_fields.push(quote! {
                                .field(#field_name_str, &self.#field_name)
                            });
                        }
                    }
                }
            }
        },
    }

    let expanded = quote! {
        impl #impl_generics ::std::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(#name)
                #(#debug_fields)*
                .finish()
            }
        }
    };
    TokenStream::from(expanded)
}

fn get_debug_attribute(field: &Field) -> Option<&LitStr> {
    for attr in &field.attrs {
        let path = attr.path();
        if path.is_ident("debug") {
            match &attr.meta {
                Meta::Path(_) => {
                    unimplemented!()
                }
                Meta::List(_) => {
                    unimplemented!()
                }
                Meta::NameValue(name_value) => match &name_value.value {
                    Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) => return Some(lit_str),
                    _ => {
                        unimplemented!()
                    }
                },
            }
        }
    }
    None
}
