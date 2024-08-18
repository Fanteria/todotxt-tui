use proc_macro::TokenStream;
use quote::{format_ident, quote};
use crate::impl_conf_functions;

use super::CONF_OPTION;

pub fn impl_conf(ast: &syn::DeriveInput) -> TokenStream {
    let name_conf = format_ident!("{}{CONF_OPTION}", ast.ident);
    let name = &ast.ident;
    let help_heading = name.to_string().replace("Config", "");

            // #[clap(long, group = "export", help_heading = "Export")]
    let fields = match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => &named.named,
            _ => panic!("Conf can only be derived for structs"),
        },
        _ => panic!("Conf can only be derived for structs"),
    };
    let mut fields_vec = Vec::new();
    let mut fields_merge = Vec::new();
    let mut fields_from_trait = Vec::new();
    for field in fields {
        let field_name = &field.ident;
        let ty = &field.ty;
        let attrs = &field.attrs;

        let mandatory = quote! {
            #[arg(long)]
            #[serde()]
            #[clap(help_heading = #help_heading)]
            #(#attrs)*
            pub #field_name: Option<#ty>,
        };

        fields_vec.push(match ty {
            syn::Type::Path(syn::TypePath { qself: _, path }) => {
                let last = path.segments.last().expect("TODO");
                match last.ident.to_string().as_str() {
                    "Duration" => quote! {
                        #[arg(value_parser = super::parsers::parse_duration)]
                        #[arg(value_name = "DURATION")]
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
                    // "Option" => quote! {
                    //     #[arg(long)]
                    //     #[serde()]
                    //     pub #field_name: #ty,
                    // },
                    _ => quote! {
                        #mandatory
                    },
                }
            }
            _ => panic!("TODO"),
        });
        fields_merge.push(quote! {
            #field_name: additional.#field_name.unwrap_or(source.#field_name),
        });
        fields_from_trait.push(quote!{
            #field_name: Some(value.#field_name),
        })
    }

    let from_reader = impl_conf_functions::from_reader();
    let from_iter = impl_conf_functions::from_iter();
    let impl_conf_trait = impl_conf_functions::impl_conf_trait(name, &name_conf);
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
    .into()
}
