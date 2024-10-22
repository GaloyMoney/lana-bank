mod create_fn;
mod find_all_fn;
mod find_by_fn;
mod list_by_fn;
mod options;
mod persist_events_fn;
mod post_persist_hook;
mod update_fn;

use darling::{FromDeriveInput, ToTokens};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

use options::RepositoryOptions;

pub fn derive(ast: syn::DeriveInput) -> darling::Result<proc_macro2::TokenStream> {
    let opts = RepositoryOptions::from_derive_input(&ast)?;
    let repo = EsRepo::from(&opts);
    Ok(quote!(#repo))
}
pub struct EsRepo<'a> {
    repo: &'a syn::Ident,
    persist_events_fn: persist_events_fn::PersistEventsFn<'a>,
    update_fn: update_fn::UpdateFn<'a>,
    create_fn: create_fn::CreateFn<'a>,
    find_by_fns: Vec<find_by_fn::FindByFn<'a>>,
    find_all_fn: find_all_fn::FindAllFn<'a>,
    post_persist_hook: post_persist_hook::PostPersistHook<'a>,
    list_by_fns: Vec<list_by_fn::ListByFn<'a>>,
    opts: &'a RepositoryOptions,
}

impl<'a> From<&'a RepositoryOptions> for EsRepo<'a> {
    fn from(opts: &'a RepositoryOptions) -> Self {
        let find_by_fns = opts
            .columns
            .all_find_by()
            .map(|c| find_by_fn::FindByFn::new(c.name(), c.ty(), opts))
            .collect();
        let list_by_fns = opts
            .columns
            .all_list_by()
            .map(|c| list_by_fn::ListByFn::new(c, opts))
            .collect();

        Self {
            repo: &opts.ident,
            persist_events_fn: persist_events_fn::PersistEventsFn::from(opts),
            update_fn: update_fn::UpdateFn::from(opts),
            create_fn: create_fn::CreateFn::from(opts),
            find_by_fns,
            find_all_fn: find_all_fn::FindAllFn::from(opts),
            post_persist_hook: post_persist_hook::PostPersistHook::from(opts),
            list_by_fns,
            opts,
        }
    }
}

impl<'a> ToTokens for EsRepo<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repo = &self.repo;
        let persist_events_fn = &self.persist_events_fn;
        let update_fn = &self.update_fn;
        let create_fn = &self.create_fn;
        let find_by_fns = &self.find_by_fns;
        let find_all_fn = &self.find_all_fn;
        let post_persist_hook = &self.post_persist_hook;
        let cursors = self.list_by_fns.iter().map(|l| l.cursor());
        let list_by_fns = &self.list_by_fns;

        let entity = self.opts.entity();
        let event = self.opts.event();
        let id = self.opts.id();
        let error = self.opts.err();

        tokens.append_all(quote! {
            pub mod cursor {
                use super::*;

                #(#cursors)*
            }

            mod repo_types {

                use super::*;

                #[allow(non_camel_case_types)]
                pub(super) type Repo__Id = #id;
                #[allow(non_camel_case_types)]
                pub(super) type Repo__Event = #event;
                #[allow(non_camel_case_types)]
                pub(super) type Repo__Entity = #entity;
                #[allow(non_camel_case_types)]
                pub(super) type Repo__Error = #error;
                #[allow(non_camel_case_types)]
                pub(super) type Repo__DbEvent = es_entity::GenericEvent<#id>;

                pub(super) struct QueryRes {
                    pub(super) rows: Vec<Repo__DbEvent>,
                }

                impl QueryRes {
                    pub(super) async fn fetch_one(
                        self,
                    ) -> Result<Repo__Entity, Repo__Error>
                    {
                        Ok(es_entity::EntityEvents::load_first(self.rows.into_iter())?)
                    }

                    pub(super) async fn fetch_n(
                        self,
                        first: usize,
                    ) -> Result<(Vec<Repo__Entity>, bool), Repo__Error>
                    {
                        Ok(es_entity::EntityEvents::load_n(self.rows.into_iter(), first)?)
                    }
                }
            }

            impl #repo {
                #[inline(always)]
                fn pool(&self) -> &sqlx::PgPool {
                    &self.pool
                }

                #post_persist_hook
                #persist_events_fn
                #create_fn
                #update_fn
                #(#find_by_fns)*
                #find_all_fn
                #(#list_by_fns)*
            }
        });
    }
}
