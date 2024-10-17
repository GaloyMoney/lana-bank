#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod event;
mod repo;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(EsEvent, attributes(es_event))]
pub fn es_event_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    event::derive(ast).into()
}

#[proc_macro_derive(EsRepo, attributes(es_repo))]
pub fn es_repo_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    repo::derive(ast).into()
}
