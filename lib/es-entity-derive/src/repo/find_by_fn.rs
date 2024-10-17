use darling::ToTokens;
use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};

use super::options::*;

pub struct FindByFn<'a> {
    id: &'a syn::Ident,
    entity: &'a syn::Ident,
    column_name: syn::Ident,
    column_type: syn::Type,
    table_name: &'a str,
    events_table_name: &'a str,
    error: &'a syn::Type,
}

impl<'a> FindByFn<'a> {
    pub fn new(
        column_name: syn::Ident,
        column_type: syn::Type,
        opts: &'a RepositoryOptions,
    ) -> Self {
        Self {
            column_name,
            column_type,
            id: opts.id(),
            entity: opts.entity(),
            table_name: opts.table_name(),
            events_table_name: opts.events_table_name(),
            error: opts.err(),
        }
    }
}

impl<'a> ToTokens for FindByFn<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entity = self.entity;
        let column_name = &self.column_name;
        let column_type = &self.column_type;
        let error = self.error;

        let fn_name = syn::Ident::new(&format!("find_by_{}", column_name), Span::call_site());
        let fn_via = syn::Ident::new(&format!("find_by_{}_via", column_name), Span::call_site());
        let fn_in_tx =
            syn::Ident::new(&format!("find_by_{}_in_tx", column_name), Span::call_site());

        let query = format!(
            r#"SELECT i.id AS "id: {}", e.sequence, e.event, e.recorded_at FROM {} i JOIN {} e ON i.id = e.id WHERE i.{} = $1 ORDER BY e.sequence"#,
            self.id, self.table_name, self.events_table_name, column_name
        );

        tokens.append_all(quote! {
            pub async fn #fn_name(
                &self,
                #column_name: #column_type
            ) -> Result<#entity, #error> {
                self.#fn_via(self.pool(), #column_name).await
            }

            pub async fn #fn_in_tx(
                &self,
                db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
                #column_name: #column_type
            ) -> Result<#entity, #error> {
                self.#fn_via(&mut **db, #column_name).await
            }

            async fn #fn_via(
                &self,
                executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
                #column_name: #column_type
            ) -> Result<#entity, #error> {
                let rows = sqlx::query!(
                    #query,
                    #column_name as #column_type,
                )
                    .fetch_all(executor)
                    .await?;
                Ok(es_entity::EntityEvents::load_first(rows.into_iter().map(|r|
                    es_entity::GenericEvent {
                        id: r.id,
                        sequence: r.sequence,
                        event: r.event,
                        recorded_at: r.recorded_at,
                }))?)
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::Ident;

    #[test]
    fn find_by_fn() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let column_type = syn::parse_str("EntityId").unwrap();
        let entity = Ident::new("Entity", Span::call_site());
        let error = syn::parse_str("es_entity::EsRepoError").unwrap();

        let persist_fn = FindByFn {
            id: &id_type,
            column_name: Ident::new("id", Span::call_site()),
            column_type,
            entity: &entity,
            table_name: "entities",
            events_table_name: "entity_events",
            error: &error,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            pub async fn find_by_id(
                &self,
                id: EntityId
            ) -> Result<Entity, es_entity::EsRepoError> {
                self.find_by_id_via(self.pool(), id).await
            }

            pub async fn find_by_id_in_tx(
                &self,
                db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
                id: EntityId
            ) -> Result<Entity, es_entity::EsRepoError> {
                self.find_by_id_via(&mut **db, id).await
            }

            async fn find_by_id_via(
                &self,
                executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
                id: EntityId
            ) -> Result<Entity, es_entity::EsRepoError> {
                let rows = sqlx::query!(
                    "SELECT i.id AS \"id: EntityId\", e.sequence, e.event, e.recorded_at FROM entities i JOIN entity_events e ON i.id = e.id WHERE i.id = $1 ORDER BY e.sequence",
                    id as EntityId,
                )
                    .fetch_all(executor)
                    .await?;
                Ok(es_entity::EntityEvents::load_first(rows.into_iter().map(|r|
                    es_entity::GenericEvent {
                        id: r.id,
                        sequence: r.sequence,
                        event: r.event,
                        recorded_at: r.recorded_at,
                }))?)
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
