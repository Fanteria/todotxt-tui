use super::CONF_OPTION;
use crate::impl_conf_functions;
use core::panic;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Meta;

pub fn impl_conf_merge(ast: &syn::DeriveInput) -> TokenStream {
    let name_conf = format_ident!("{}{CONF_OPTION}", ast.ident);
    let name = &ast.ident;
    // Structure used for exporting auto complete or configuration.
    let export_struct = ast
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("export_option") {
                Some(match &attr.meta {
                    Meta::List(meta_list) => meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .find_map(|f| match f {
                            proc_macro2::TokenTree::Ident(ident) => Some(ident),
                            _ => None,
                        })
                        .expect("Struct implementing Export trait must be set."),
                    _ => panic!("Struct implementing Export trait must be set."),
                })
            } else {
                None
            }
        })
        .expect("Struct implementing Export trait must be set.");
    // Remove `export_option` from attributes
    let attrs: Vec<_> = ast
        .attrs
        .iter()
        .filter(|attr| {
            if attr.path().is_ident("export_option") {
                false
            } else {
                true
            }
        })
        .collect();
    let fields = impl_conf_functions::get_fields(ast);

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

            #[serde(skip)]
            #[command(flatten)]
            export: #export_struct,

            /// Path to configuration file.
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
                    #(#fields_from_trait)*
                    export: #export_struct::default(),
                    config_path: None,
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
                let file = std::fs::File::open(&path).map_err(|e| crate::ToDoError::io_operation_failed(&path, e))?;
                let from_matches = #name_conf::from_arg_matches(&matches).unwrap();

                from_matches.export.export(path.as_path(), &matches)?;

                let from_reader = #name_conf::merge(
                    Self::default(),
                    #name_conf::from_reader(file)?,
                );
                Ok(#name_conf::merge(from_reader, from_matches))
            }

            fn configured_toml(path: impl AsRef<Path>, matches: &clap::ArgMatches) -> Result<String> {
                let file = std::fs::File::open(path.as_ref()).map_err(|e| crate::ToDoError::io_operation_failed(path.as_ref(), e))?;
                let from_reader = #name_conf::merge(
                    Self::default(),
                    #name_conf::from_reader(file)?,
                );
                let from_matches = #name_conf::from_arg_matches(&matches).unwrap();
                let conf = #name_conf::merge(from_reader, from_matches);
                let mut conf: #name_conf = conf.into();
                Ok(toml::to_string_pretty(&conf)?)
            }

            fn default_toml() -> Result<String> {
                let mut default: #name_conf = Self::default().into();
                Ok(toml::to_string_pretty(&default)?)
            }

            fn autocomplete(writer: &mut impl std::io::Write) -> crate::error::Result<()> {
                use clap::CommandFactory;
                clap_complete::generate(
                    clap_complete::shells::Bash,
                    &mut #name_conf::command(),
                    env!("CARGO_PKG_NAME"),
                    writer,
                );
                Ok(())
            }

        }

        #impl_conf_trait
    }
}
