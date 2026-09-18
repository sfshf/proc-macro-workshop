use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Ident, PathArguments,
    Type::{self},
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let builder_name = format_ident!("{}Builder", ident);
    let mut builder_fields: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut builder_fields_inits: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut builder_methods: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut build_fields: Vec<proc_macro2::TokenStream> = Vec::new();
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
                    let field_vis = &field.vis;
                    let field_name = field.ident.as_ref().unwrap();
                    let mut field_ty = &field.ty;
                    // builder fields
                    builder_fields.push(quote! {
                        #field_name : #field_ty,
                    });
                    // builder fields inits
                    builder_fields_inits.push(quote! {
                        #field_name : ::std::default::Default::default(),
                    });
                    // builder methods
                    match get_inner_type("Option", field_ty) {
                        Some(ty) => {
                            builder_methods.push(quote! {
                                #field_vis fn #field_name(&mut self, arg: #ty) -> &mut Self {
                                    self.#field_name = ::std::option::Option::Some(arg);
                                    self
                                }
                            });
                            field_ty = ty
                        }
                        None => {
                            builder_methods.push(quote! {
                                #field_vis fn #field_name(&mut self, arg: #field_ty) -> &mut Self {
                                    self.#field_name = arg;
                                    self
                                }
                            });
                        }
                    };
                    // build fields
                    build_fields.push(quote! {
                        #field_name : self.#field_name.clone(),
                    });
                    // builder each methods
                    for attr in &field.attrs {
                        if !attr.path().is_ident("builder") {
                            continue;
                        }
                        let ret = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("each") {
                                let value = meta.value()?;
                                let lit: syn::LitStr = value.parse()?;
                                let lit = lit.value();
                                let field_method = Ident::new(&lit, Span::call_site());
                                match get_inner_type("Vec", field_ty) {
                                    Some(ty) => {
                                        builder_methods.push(quote! {
                                            #field_vis fn #field_method(&mut self, arg: #ty) -> &mut Self {
                                                self.#field_name.push(arg);
                                                self
                                            }
                                        });
                                    }
                                    None => {}
                                }
                                Ok(())
                            } else {
                                Err(meta.error("expected `builder(each = \"...\")`"))
                            }
                        });
                        match ret {
                            Ok(_) => {}
                            Err(err) => return err.to_compile_error().into(),
                        }
                    }
                }
            }
        },
    }

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields)*
        }
        impl #ident {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_fields_inits)*
                }
            }
        }
        impl #builder_name {

            #(#builder_methods)*

            pub fn build(&mut self) -> ::std::result::Result<#ident, ::std::boxed::Box<dyn ::std::error::Error>> {
                ::std::result::Result::Ok(#ident {
                    #(#build_fields)*
                })
            }
        }
    };
    expanded.into()
}

fn get_inner_type<'a>(ident: &'a str, ty: &'a Type) -> Option<&'a Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != ident {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}
