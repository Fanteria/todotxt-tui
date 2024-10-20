extern crate proc_macro;

mod impl_conf;
mod impl_conf_functions;
mod impl_conf_merge;

use proc_macro::TokenStream;

const CONF_OPTION: &str = "ConfOption";

#[proc_macro_derive(Conf, attributes(arg))]
pub fn conf_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("Syn cannot parse Conf macro input");
    impl_conf::impl_conf(&ast)
}

#[proc_macro_derive(ConfMerge, attributes(command))]
pub fn conf_merge_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("Syn cannot parse ConfMerge macro input");
    impl_conf_merge::impl_conf_merge(&ast)
}
