use core::panic;

use super::CONF_OPTION;
use crate::impl_conf_functions;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

pub fn impl_conf_merge(ast: &syn::DeriveInput) -> TokenStream {
    let name_conf = format_ident!("{}{CONF_OPTION}", ast.ident);
    let name = &ast.ident;
    let attrs = &ast.attrs;
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
        let ty_conf = match &field.ty {
            syn::Type::Path(type_path) => {
                if let Some(last_segment) = type_path.path.segments.last() {
                    format_ident!("{}{CONF_OPTION}", &last_segment.ident)
                } else {
                    panic!("Type path have not a single segment.")
                }
            }
            _ => panic!("Cannot create conf type."),
        };
        fields_vec.push(quote! {
            #[clap(flatten)]
            #[serde(flatten)]
            pub #field_name: #ty_conf,
        });
        fields_merge.push(quote! {
            #field_name: #ty_conf::merge(source.#field_name, additional.#field_name),
        });
        fields_from_trait.push(quote! {
            #field_name: value.#field_name.into(),
        })
    }

    let from_reader = impl_conf_functions::from_reader();
    let from_iter = impl_conf_functions::from_iter();
    let impl_conf_trait = impl_conf_functions::impl_conf_trait(name, &name_conf);
    quote! {
        #[derive(serde::Serialize, serde::Deserialize, clap::Parser, Debug, PartialEq, Eq, Clone)]
        #[command(styles = #name::help_colors())]
        #(#attrs)*
        struct #name_conf {

            #(#fields_vec)*

            /// Generate autocomplete script to given file path.
            #[serde(skip)]
            #[clap(long, group = "export", help_heading = "Export")]
            #[arg(value_name = "PATH")]
            pub export_autocomplete: Option<std::path::PathBuf>,

            /// Generate full configuration file for actual session
            /// so present configuration file and command lines
            /// options are taken in account.
            #[serde(skip)]
            #[clap(long, group = "export", help_heading = "Export")]
            #[arg(value_name = "PATH")]
            pub export_config: Option<std::path::PathBuf>,

            /// Generate configuration file with default values
            /// to given file path.
            #[serde(skip)]
            #[clap(long, group = "export", help_heading = "Export")]
            #[arg(value_name = "PATH")]
            pub export_default_config: Option<std::path::PathBuf>,

            /// Path to configuration file.
            #[serde(skip)]
            #[clap(short, long)]
            #[arg(value_name = "PATH")]
            config_path: Option<std::path::PathBuf>,
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
                    export_autocomplete: None,
                    export_config: None,
                    export_default_config: None,
                    config_path: None,
                    #(#fields_from_trait)*
                }
            }
        }

        impl crate::config::ConfMerge for #name {
            fn from_args<Iter, T>(iter: Iter) -> crate::error::Result<Self>
            where
                Iter: IntoIterator<Item = T>,
                T: Into<std::ffi::OsString> + Clone,
            {
                use clap::CommandFactory;
                let matches = #name_conf::command().get_matches_from(iter);
                let path = match matches.get_one::<std::path::PathBuf>("config_path") {
                    Some(config_path) => config_path.to_owned(),
                    None => Self::config_path(),
                };
                let file = std::fs::File::open(&path).map_err(|e| crate::ToDoIoError::new(path, e))?;
                let from_matches = #name_conf::from_arg_matches(&matches).unwrap();
                if let Some(path) = &from_matches.export_autocomplete {
                    clap_complete::generate(
                        clap_complete::shells::Bash,
                        &mut #name_conf::command(),
                        env!("CARGO_PKG_NAME"),
                        &mut std::fs::File::create(path).map_err(|e| crate::ToDoIoError::new(path, e))?,
                    );
                    std::process::exit(0);
                } else if let Some(path) = &from_matches.export_config {
                    use std::io::Write;
                    let from_reader = #name_conf::merge(
                        Self::default(),
                        #name_conf::from_reader(file)?,
                    );
                    let from_matches = #name_conf::from_arg_matches(&matches).unwrap();
                    let conf = #name_conf::merge(from_reader, from_matches);
                    let mut conf: #name_conf = conf.into();

                    // TODO ugly hack
                    conf.ui_config.save_state_path = None;
                    conf.file_worker_config.archive_path = None;

                    let mut output = std::fs::File::create(path).map_err(|e| crate::ToDoIoError::new(path, e))?;
                    write!(output, "{}", toml::to_string_pretty(&conf)?).map_err(|e| crate::IOError(e))?;
                    std::process::exit(0);
                } else if let Some(path) = &from_matches.export_default_config {
                    use std::io::Write;
                    let mut default: #name_conf = Self::default().into();
                    let mut output = std::fs::File::create(path).map_err(|e| crate::ToDoIoError::new(path, e))?;
                    // TODO ugly hack
                    default.ui_config.save_state_path = None;
                    default.file_worker_config.archive_path = None;
                    write!(output, "{}", toml::to_string_pretty(&default)?)
                        .map_err(|e| crate::IOError(e))?;
                    std::process::exit(0);
                }

                let from_reader = #name_conf::merge(
                    Self::default(),
                    #name_conf::from_reader(file)?,
                );
                Ok(#name_conf::merge(from_reader, from_matches))
            }
        }

        #impl_conf_trait
    }
    .into()
}
