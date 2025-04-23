use super::CONF_OPTION;
use crate::impl_conf_functions;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Field};

pub fn impl_conf(ast: &syn::DeriveInput) -> TokenStream {
    // Name of generated configuration structure.
    let name_conf = format_ident!("{}{CONF_OPTION}", ast.ident);
    // Name of original configuration structure.
    let name = &ast.ident;
    let help_heading = name.to_string().replace("Config", "");
    let fields = impl_conf_functions::get_fields(ast);

    let fields_vec = impl_conf_generate_fields(fields, &help_heading);
    let fields_merge = impl_conf_generate_fields_merge(fields);
    let fields_from_trait = impl_conf_generate_fields_from_trait(fields);

    let from_reader = impl_conf_functions::from_reader();
    let from_iter = impl_conf_functions::from_iter();
    let impl_conf_trait = impl_conf_functions::impl_conf_trait(name, &name_conf);

    // Construct new configuration object token stream
    quote! {
        #[derive(serde::Serialize, serde::Deserialize, clap::Parser, Debug, PartialEq, Eq, Clone)]
        pub struct #name_conf {
            #(#fields_vec)*
        }

        impl #name_conf {
            #from_reader

            #from_iter

            fn merge(source: #name, additional: #name_conf) -> #name {
                #name {
                    #(#fields_merge)*
                }
            }
        }

        impl From<#name> for #name_conf {
            fn from(value: #name) -> Self {
                #name_conf {
                    #(#fields_from_trait)*
                }
            }
        }

        #impl_conf_trait
    }
}

/// Generate vector of fields of generated configuration structure.
fn impl_conf_generate_fields(
    fields: &Punctuated<Field, Comma>,
    help_heading: &str,
) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;
            let attrs = &field.attrs;

            // Create environment variable that can set this field.
            let env_variable = format!(
                "TODOTXT_TUI_{}",
                field_name
                    .as_ref()
                    .expect("Field name have no identifier.")
                    .to_string()
                    .to_uppercase()
            );

            let new_type = if impl_conf_is_option(field) {
                quote! { #field_type }
            } else {
                quote! { Option<#field_type> }
            };

            // Mandatory derives: long flag, serialization, environment variable and heading
            let mandatory = quote! {
                #[arg(long)]
                #[serde()]
                #[clap(env = #env_variable, help_heading = #help_heading)]
                #(#attrs)*
                pub #field_name: #new_type,
            };

            match field_type {
                syn::Type::Path(path) => match impl_conf_functions::find_ident(path)
                    .expect("Field name have no identifier.")
                    .to_string()
                    .as_str()
                {
                    "Duration" => quote! {
                        #[arg(value_parser = self::parsers::parse_duration)]
                        #[arg(value_name = "DURATION")]
                        #mandatory
                    },
                    "usize" => quote! {
                        #[arg(value_name = "+NUM")]
                        #mandatory
                    },
                    "bool" => quote! {
                        #[arg(value_name = "BOOL")]
                        #mandatory
                    },
                    "TextStyleList" => quote! {
                        #[arg(value_name = "TEXT_STYLE_LIST")]
                        #mandatory
                    },
                    "TextStyle" => quote! {
                        #[arg(value_name = "TEXT_STYLE")]
                        #mandatory
                    },
                    "TaskSort" => quote! {
                        #[arg(value_name = "TASK_SORT")]
                        #mandatory
                    },
                    "WidgetType" => quote! {
                        #[arg(value_name = "WIDGET_TYPE")]
                        #mandatory
                    },
                    "Color" => quote! {
                        #[arg(value_name = "COLOR")]
                        #mandatory
                    },
                    "PathBuf" => quote! {
                        #[arg(value_name = "PATH")]
                        #mandatory
                    },
                    "LevelFilter" => quote! {
                        #[arg(value_name = "LOG_LEVEL")]
                        #mandatory
                    },
                    "String" => quote! {
                        #[arg(value_name = "STRING")]
                        #mandatory
                    },
                    "EventHandlerUI" => quote! {
                        #[arg(value_name = "KEYBINDS")]
                        #mandatory
                    },
                    _ => quote! {
                        #mandatory
                    },
                },
                _ => quote! {
                    #mandatory
                },
            }
        })
        .collect()
}

/// Way how to merge field from original structure with field
/// in generated configuration structure.
fn impl_conf_generate_fields_merge(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;
            let field_name = &field.ident;
            match field_type {
                syn::Type::Path(path) => match impl_conf_functions::find_ident(path)
                    .expect("Field name have no identifier.")
                    .to_string()
                    .as_str()
                {
                    "EventHandlerUI" => quote! {
                        #field_name: match additional.#field_name {
                            Some(a) => {
                                let mut s = source.#field_name;
                                s.combine(a);
                                s
                            }
                            None => source.#field_name,
                        },
                    },
                    _ if impl_conf_is_option(field) => quote! {
                        #field_name: additional.#field_name.or(source.#field_name),
                    },
                    _ => quote! {
                        #field_name: additional.#field_name.unwrap_or(source.#field_name),
                    },
                },
                _ => panic!("Unexpected type, cannot expand macro."),
            }
        })
        .collect()
}

/// Way how to convert field from original structure to field
/// in configuration structure.
fn impl_conf_generate_fields_from_trait(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            if impl_conf_is_option(field) {
                quote! {
                            #field_name: value.#field_name,
                }
            } else {
                quote! {
                            #field_name: Some(value.#field_name),
                }
            }
        })
        .collect()
}

fn impl_conf_is_option(field: &Field) -> bool {
    let field_type = &field.ty;
    matches!(field_type, syn::Type::Path(path) if path
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == "Option"))
}
