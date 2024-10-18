use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

use super::options::*;

pub struct PersistFn<'a> {
    id: &'a syn::Ident,
    entity: &'a syn::Ident,
    table_name: &'a str,
    columns: &'a Indexes,
    error: &'a syn::Type,
}

impl<'a> From<&'a RepositoryOptions> for PersistFn<'a> {
    fn from(opts: &'a RepositoryOptions) -> Self {
        Self {
            id: opts.id(),
            entity: opts.entity(),
            error: opts.err(),
            columns: &opts.columns,
            table_name: opts.table_name(),
        }
    }
}

impl<'a> ToTokens for PersistFn<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entity = self.entity;
        let error = self.error;

        let update_tokens = if !self.columns.columns.is_empty() {
            let index_tokens = self.columns.columns.iter().map(|column| {
                let ident = &column.name;
                quote! {
                    let #ident = &entity.#ident;
                }
            });
            let column_updates = self
                .columns
                .columns
                .iter()
                .enumerate()
                .map(|(idx, column)| format!("{} = ${}", column.name, idx + 2))
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!(
                "UPDATE {} SET {} WHERE id = $1",
                self.table_name, column_updates,
            );
            let args = self.columns.query_args();
            let id = &self.id;
            Some(quote! {
            let id = &entity.id;
            #(#index_tokens)*
            sqlx::query!(
                #query,
                id as &#id,
                #(#args),*
            )
                .execute(&mut **db)
                .await?;
            })
        } else {
            None
        };

        tokens.append_all(quote! {
            #[inline(always)]
            fn extract_events<T, E>(entity: &mut T) -> &mut es_entity::EntityEvents<E>
            where
                T: es_entity::EsEntity<E>,
                E: es_entity::EsEvent,
            {
                entity.events_mut()
            }

            pub async fn persist(
                &self,
                entity: &mut #entity
            ) -> Result<(), #error> {
                let mut db = self.pool().begin().await?;
                let res = self.persist_in_tx(&mut db, entity).await?;
                db.commit().await?;
                Ok(res)
            }

            pub async fn persist_in_tx(
                &self,
                db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
                entity: &mut #entity
            ) -> Result<(), #error> {
                if !Self::extract_events(entity).any_new() {
                    return Ok(());
                }

                #update_tokens
                let events = Self::extract_events(entity);
                let n_events = self.persist_events(db, events).await?;

                self.execute_post_persist_hook(db, events.last_persisted(n_events)).await?;

                Ok(())
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
    fn persist_fn() {
        let id = syn::parse_str("EntityId").unwrap();
        let entity = Ident::new("Entity", Span::call_site());
        let error = syn::parse_str("es_entity::EsRepoError").unwrap();

        let columns = Indexes {
            columns: vec![IndexColumn {
                name: Ident::new("name", Span::call_site()),
                ty: syn::parse_str("String").unwrap(),
            }],
        };

        let persist_fn = PersistFn {
            entity: &entity,
            table_name: "entities",
            id: &id,
            error: &error,
            columns: &columns,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            #[inline(always)]
            fn extract_events<T, E>(entity: &mut T) -> &mut es_entity::EntityEvents<E>
            where
                T: es_entity::EsEntity<E>,
                E: es_entity::EsEvent,
            {
                entity.events_mut()
            }

            pub async fn persist(
                &self,
                entity: &mut Entity
            ) -> Result<(), es_entity::EsRepoError> {
                let mut db = self.pool().begin().await?;
                let res = self.persist_in_tx(&mut db, entity).await?;
                db.commit().await?;
                Ok(res)
            }

            pub async fn persist_in_tx(
                &self,
                db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
                entity: &mut Entity
            ) -> Result<(), es_entity::EsRepoError> {
                if !Self::extract_events(entity).any_new() {
                    return Ok(());
                }

                let id = &entity.id;
                let name = &entity.name;
                sqlx::query!(
                    "UPDATE entities SET name = $2 WHERE id = $1",
                    id as &EntityId,
                    name as &String
                )
                    .execute(&mut **db)
                    .await?;

                let events = Self::extract_events(entity);
                let n_events = self.persist_events(db, events).await?;

                self.execute_post_persist_hook(db, events.last_persisted(n_events)).await?;

                Ok(())
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn persist_fn_no_columns() {
        let id = syn::parse_str("EntityId").unwrap();
        let entity = Ident::new("Entity", Span::call_site());
        let error = syn::parse_str("es_entity::EsRepoError").unwrap();

        let columns = Indexes { columns: vec![] };

        let persist_fn = PersistFn {
            entity: &entity,
            table_name: "entities",
            id: &id,
            error: &error,
            columns: &columns,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            #[inline(always)]
            fn extract_events<T, E>(entity: &mut T) -> &mut es_entity::EntityEvents<E>
            where
                T: es_entity::EsEntity<E>,
                E: es_entity::EsEvent,
            {
                entity.events_mut()
            }

            pub async fn persist(
                &self,
                entity: &mut Entity
            ) -> Result<(), es_entity::EsRepoError> {
                let mut db = self.pool().begin().await?;
                let res = self.persist_in_tx(&mut db, entity).await?;
                db.commit().await?;
                Ok(res)
            }

            pub async fn persist_in_tx(
                &self,
                db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
                entity: &mut Entity
            ) -> Result<(), es_entity::EsRepoError> {
                if !Self::extract_events(entity).any_new() {
                    return Ok(());
                }

                let events = Self::extract_events(entity);
                let n_events = self.persist_events(db, events).await?;

                self.execute_post_persist_hook(db, events.last_persisted(n_events)).await?;

                Ok(())
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
