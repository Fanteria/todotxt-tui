use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Field;

pub fn from_reader() -> TokenStream {
    quote! {
            fn from_reader<R>(mut reader: R) -> crate::Result<Self>
            where
                R: std::io::Read
            {
                let mut buf = String::default();
                reader.read_to_string(&mut buf)?;
                Ok(toml::from_str(buf.as_str())?)
            }
    }
}

pub fn from_iter() -> TokenStream {
    quote! {
            fn from_iter<Iter, T>(iter: Iter) -> Self
            where
                Self: Sized,
                Iter: IntoIterator<Item = T>,
                T: Into<std::ffi::OsString> + Clone
            {
                use clap::Parser;
                return Self::from_iter(iter)
            }
    }
}

pub fn impl_conf_trait(name: &Ident, name_conf: &Ident) -> TokenStream {
    quote! {
        impl crate::config::Conf for #name {
            fn from_reader<R>(mut reader: R) -> crate::Result<Self>
            where
                R: std::io::Read
            {
                Ok(#name_conf :: merge(
                    Self::default(),
                    #name_conf :: from_reader(reader)?
                ))
            }

            fn parse<Iter, T, R>(iter: Iter, reader: R) -> crate::Result<Self>
            where
                Iter: IntoIterator<Item = T>,
                T: Into<std::ffi::OsString> + Clone,
                R: std::io::Read
            {
                Ok(#name_conf :: merge(
                    Self::from_reader(reader)?,
                    #name_conf :: from_iter(iter)
                ))
            }
        }
    }
}

pub fn find_ident(path: &syn::TypePath) -> Option<&syn::Ident> {
    let last = path.path.segments.last()?;
    if last.ident == "Option" {
        match &last.arguments {
            syn::PathArguments::AngleBracketed(angle) => match angle.args.last()? {
                syn::GenericArgument::Type(syn::Type::Path(p)) => find_ident(p),
                _ => None,
            },
            _ => None,
        }
    } else {
        Some(&last.ident)
    }
}

pub fn get_fields(ast: &syn::DeriveInput) -> &Punctuated<Field, Comma> {
    match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => &named.named,
            _ => panic!("Must have all fields named"),
        },
        _ => panic!("Can be derived only for structs"),
    }
}
