use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, GenericArgument, Ident, PathArguments,
    Type::Path,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(name, "Builder can only be derived for structs")
            .to_compile_error()
            .into();
    };
    let mut methods: Vec<proc_macro2::TokenStream> = Vec::new();
    for field in &data.fields {
        for attr in &field.attrs {
            if attr.path().is_ident("builder") {
                _ = attr.parse_nested_meta(|meta| {
                    if !meta.path.is_ident("each") {
                        panic!("expected `builder(each = \"...\")`");
                    };
                    let method_name = meta.value()?.parse::<syn::LitStr>()?.value();
                    let method_name = Ident::new(&method_name, method_name.span());
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = match &field.ty {
                        Path(path) => path,
                        _ => panic!("expected a path type for the field"),
                    };
                    let segment = field_type.path.segments.last().unwrap();
                    if segment.ident != "Vec" {
                        panic!("expected a Vec type for the field");
                    };
                    let PathArguments::AngleBracketed(args) = &segment.arguments else {
                        panic!("expected angle bracketed arguments for the Vec type");
                    };
                    let GenericArgument::Type(ty) = args.args.first().unwrap() else {
                        panic!("expected a type argument for the Vec type");
                    };
                    methods.push(quote! {
                        pub fn #method_name(&mut self, arg: #ty) -> &mut Self {
                            if let Some(ref mut #field_name) = self.#field_name {
                                #field_name.push(arg);
                            } else {
                                self.#field_name = Some(vec![arg]);
                            }
                            self
                        }
                    });
                    Ok(())
                });
            };
        }
    }
    let expanded = quote! {
        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }
        impl #name {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
        impl CommandBuilder {
            #(#methods)*
            pub fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
            pub fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }
            pub fn envs(&mut self, envs: Vec<String>) -> &mut Self {
                self.env = Some(envs);
                self
            }
            pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    executable: self.executable.clone().ok_or("executable is not set")?,
                    args: self.args.clone().unwrap_or_default(),
                    env: self.env.clone().unwrap_or_default(),
                    current_dir: self.current_dir.clone().unwrap_or_default(),
                })
            }
        }
    };
    expanded.into()
}
