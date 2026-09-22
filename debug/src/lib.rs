use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Field, Fields, GenericArgument, GenericParam,
    LitStr, Meta, PathArguments, Type,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let name = ident.to_string();
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
            GenericParam::Type(ty) => {
                ty.bounds.push(syn::parse_quote!(::std::fmt::Debug));
            }
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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

fn is_phantom_data_of(ty: &Type, param: &syn::Ident) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(last_segment) = type_path.path.segments.last() else {
        return false;
    };
    if last_segment.ident != "PhantomData" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments else {
        return false;
    };
    if args.args.len() != 1 {
        return false;
    }
    let Some(syn::GenericArgument::Type(Type::Path(inner))) = &args.args.first() else {
        return false;
    };
    let Some(inner_last_segment) = inner.path.segments.last() else {
        return false;
    };
    inner_last_segment.ident == *param
}

fn phantom_data_params(input: &DeriveInput) -> HashSet<syn::Ident> {
    let mut params = HashSet::new();
    let Data::Struct(data) = &input.data else {
        return params;
    };
    for field in &data.fields {
        for param in input.generics.type_params() {
            if is_phantom_data_of(&field.ty, &param.ident) {
                params.insert(param.ident.clone());
            }
        }
    }
    params
}

fn type_mentions_param(ty: &Type, param: &syn::Ident) -> bool {
    match ty {
        Type::Path(type_path) => {
            type_path
                .path
                .segments
                .iter()
                .any(|segment| match &segment.arguments {
                    PathArguments::None => segment.ident == *param,

                    PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| {
                        if let GenericArgument::Type(ty) = arg {
                            type_mentions_param(ty, param)
                        } else {
                            false
                        }
                    }),

                    _ => false,
                })
        }

        _ => false,
    }
}

fn is_phantom_data(ty: &Type, param: &syn::Ident) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    if segment.ident != "PhantomData" {
        return false;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };

    if args.args.len() != 1 {
        return false;
    }

    match args.args.first() {
        Some(GenericArgument::Type(Type::Path(path))) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == *param)
            .unwrap_or(false),

        _ => false,
    }
}
