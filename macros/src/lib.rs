extern crate proc_macro;

mod impl_conf;
mod impl_conf_merge;
mod impl_conf_functions;

use proc_macro::TokenStream;

const CONF_OPTION: &str = "ConfOption";

#[proc_macro_derive(Conf, attributes(arg))]
pub fn conf_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("TODO some meaningfull");
    impl_conf::impl_conf(&ast)
}


#[proc_macro_derive(ConfMerge, attributes(command))]
pub fn conf_merge_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("TODO some meaningfull");
    impl_conf_merge::impl_conf_merge(&ast)
}

