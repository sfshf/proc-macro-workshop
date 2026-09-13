use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, LitStr, Meta};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let field_name = ident.to_string();
    let mut generics = input.generics.clone();
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let Data::Struct(data) = &input.data else {
        panic!("CustomDebug can only be derived for structs");
    };
    let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
    for field in &data.fields {
        let field_name = &field.ident;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let mut field_format: Option<&LitStr> = None;
        for attr in &field.attrs {
            let path = attr.path();
            if path.is_ident("debug") {
                let Meta::NameValue(name_value) = &attr.meta else {
                    panic!("debug attribute must be a name-value pair");
                };
                let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &name_value.value
                else {
                    panic!("debug attribute value must be a string literal");
                };
                field_format = Some(lit_str);
            }
        }
        if let Some(format_str) = field_format {
            fields.push(quote! {
                .field(#field_name_str, &format_args!(#format_str, &self.#field_name))
            });
        } else {
            fields.push(quote! {
                .field(#field_name_str, &self.#field_name)
            });
        }
    }
    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#field_name)
                #(#fields)*
                .finish()
            }
        }
    };
    TokenStream::from(expanded)
}
