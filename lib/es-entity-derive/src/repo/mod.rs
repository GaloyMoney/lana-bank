mod create_fn;
mod find_by_fn;
mod options;
mod persist_events_fn;
mod persist_fn;

use darling::{FromDeriveInput, ToTokens};
use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};

use options::RepositoryOptions;

pub fn derive(ast: syn::DeriveInput) -> proc_macro2::TokenStream {
    let opts = match RepositoryOptions::from_derive_input(&ast) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors();
        }
    };

    let repo = EsRepo::from(&opts);
    quote!(#repo)
}
pub struct EsRepo<'a> {
    repo: &'a syn::Ident,
    persist_events_fn: persist_events_fn::PersistEventsFn<'a>,
    persist_fn: persist_fn::PersistFn<'a>,
    create_fn: create_fn::CreateFn<'a>,
    find_by_fns: Vec<find_by_fn::FindByFn<'a>>,
}

impl<'a> From<&'a RepositoryOptions> for EsRepo<'a> {
    fn from(opts: &'a RepositoryOptions) -> Self {
        let mut find_by_fns = vec![find_by_fn::FindByFn::new(
            syn::Ident::new("id", Span::call_site()),
            opts.id(),
            opts,
        )];
        for i in opts.indexes.columns.iter() {
            find_by_fns.push(find_by_fn::FindByFn::new(
                i.name.clone(),
                &i.ty.as_ref().unwrap(),
                opts,
            ));
        }

        Self {
            repo: &opts.ident,
            persist_events_fn: persist_events_fn::PersistEventsFn::from(opts),
            persist_fn: persist_fn::PersistFn::from(opts),
            create_fn: create_fn::CreateFn::from(opts),
            find_by_fns,
        }
    }
}

impl<'a> ToTokens for EsRepo<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repo = &self.repo;
        let persist_events_fn = &self.persist_events_fn;
        let persist_fn = &self.persist_fn;
        let create_fn = &self.create_fn;
        let find_by_fns = &self.find_by_fns;

        tokens.append_all(quote! {
            impl #repo {
                #persist_events_fn
                #persist_fn
                #create_fn
                #(#find_by_fns)*
            }
        });
    }
}
