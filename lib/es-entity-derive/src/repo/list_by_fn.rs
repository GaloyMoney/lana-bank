use convert_case::{Case, Casing};
use darling::ToTokens;
use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};

use super::options::*;

pub struct CursorStruct<'a> {
    id: &'a syn::Ident,
    entity: &'a syn::Ident,
    column_name: &'a syn::Ident,
    column_type: &'a syn::Type,
}

impl<'a> CursorStruct<'a> {
    fn name(&self) -> String {
        format!(
            "{}By{}Cursor",
            self.entity,
            self.column_name.to_string().to_case(Case::UpperCamel)
        )
    }
}

impl<'a> ToTokens for CursorStruct<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_ident = syn::Ident::new(
            &format!(
                "{}By{}Cursor",
                self.entity,
                self.column_name.to_string().to_case(Case::UpperCamel)
            ),
            Span::call_site(),
        );
        let id = &self.id;

        let field = if &self.column_name.to_string() != "id" {
            let column_name = &self.column_name;
            let column_type = &self.column_type;
            quote! {
                pub #column_name: #column_type,
            }
        } else {
            quote! {}
        };

        tokens.append_all(quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct #struct_ident {
                pub id: #id,
                #field
            }
        });
    }
}

pub struct ListByFn<'a> {
    id: &'a syn::Ident,
    entity: &'a syn::Ident,
    column_name: syn::Ident,
    column_type: syn::Type,
    table_name: &'a str,
    events_table_name: &'a str,
    error: &'a syn::Ident,
}

impl<'a> ListByFn<'a> {
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

    pub fn cursor(&'a self) -> CursorStruct<'a> {
        CursorStruct {
            column_name: &self.column_name,
            column_type: &self.column_type,
            id: self.id,
            entity: self.entity,
        }
    }
}

impl<'a> ToTokens for ListByFn<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = self.id;
        let entity = self.entity;
        let column_name = &self.column_name;
        let column_type = &self.column_type;
        let cursor = syn::Ident::new(&self.cursor().name(), Span::call_site());
        let error = self.error;

        let fn_name = syn::Ident::new(&format!("list_by_{}", column_name), Span::call_site());
        let name = column_name.to_string();
        let mut column = format!("{}, ", name);
        let mut where_pt1 = format!("({}, id) > ($3, $2)", name);
        let mut order_by = format!("{}, ", name);
        let mut arg_tokens = quote! {
            #column_name as Option<#column_type>,
        };
        let mut cursor_arg = quote! {
            #column_name: last.#column_name.clone(),
        };
        let mut after_args = quote! {
            (id, #column_name)
        };
        let mut after_destruction = quote! {
            (Some(after.id), Some(after.#column_name))
        };
        let mut after_default = quote! {
            (None, None)
        };

        if &name == "id" {
            column = String::new();
            where_pt1 = "id > $2".to_string();
            order_by = String::new();
            arg_tokens = quote! {};
            cursor_arg = quote! {};
            after_args = quote! {
                id
            };
            after_destruction = quote! {
                Some(after.id)
            };
            after_default = quote! {
                None
            };
        };

        let query = format!(
            r#"WITH entities AS (SELECT {}id FROM {} WHERE ({}) OR $2 IS NULL ORDER BY {}id LIMIT $1) SELECT i.id AS "id: {}", e.sequence, e.event, e.recorded_at FROM entities i JOIN {} e ON i.id = e.id ORDER BY {}i.id, e.sequence"#,
            column, self.table_name, where_pt1, order_by, self.id, self.events_table_name, column
        );

        tokens.append_all(quote! {
            pub async fn #fn_name(
                &self,
                es_entity::PaginatedQueryArgs { first, after }: es_entity::PaginatedQueryArgs<cursor::#cursor>,
            ) -> Result<es_entity::PaginatedQueryRet<#entity, cursor::#cursor>, #error> {
                let #after_args = if let Some(after) = after {
                    #after_destruction
                } else {
                    #after_default
                };

                let rows = sqlx::query!(
                    #query,
                    (first + 1) as i64,
                    id as Option<#id>,
                    #arg_tokens
                )
                    .fetch_all(self.pool())
                    .await?;

                let (entities, has_next_page) = EntityEvents::load_n::<#entity>(rows.into_iter().map(|r|
                        es_entity::GenericEvent {
                            id: r.id,
                            sequence: r.sequence,
                            event: r.event,
                            recorded_at: r.recorded_at,
                        }), first)?;
                let mut end_cursor = None;
                if let Some(last) = entities.last() {
                    end_cursor = Some(cursor::#cursor {
                        id: last.id,
                        #cursor_arg
                    });
                }

                Ok(es_entity::PaginatedQueryRet {
                    entities,
                    has_next_page,
                    end_cursor,
                })
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
    fn cursor_struct_by_id() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let entity = Ident::new("Entity", Span::call_site());
        let column_name = Ident::new("id", Span::call_site());
        let column_type = syn::parse_str(&id_type.to_string()).unwrap();

        let cursor = CursorStruct {
            column_name: &column_name,
            column_type: &column_type,
            id: &id_type,
            entity: &entity,
        };

        let mut tokens = TokenStream::new();
        cursor.to_tokens(&mut tokens);

        let expected = quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct EntityByIdCursor {
                pub id: EntityId,
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn cursor_struct_by_created_at() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let entity = Ident::new("Entity", Span::call_site());
        let column_name = Ident::new("created_at", Span::call_site());
        let column_type = syn::parse_str("DateTime<Utc>").unwrap();

        let cursor = CursorStruct {
            column_name: &column_name,
            column_type: &column_type,
            id: &id_type,
            entity: &entity,
        };

        let mut tokens = TokenStream::new();
        cursor.to_tokens(&mut tokens);

        let expected = quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct EntityByCreatedAtCursor {
                pub id: EntityId,
                pub created_at: DateTime<Utc>,
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn list_by_fn() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let column_type = syn::parse_str(&id_type.to_string()).unwrap();
        let entity = Ident::new("Entity", Span::call_site());
        let error = Ident::new("EsRepoError", Span::call_site());

        let persist_fn = ListByFn {
            column_name: Ident::new("id", Span::call_site()),
            column_type,
            id: &id_type,
            entity: &entity,
            table_name: "entities",
            events_table_name: "entity_events",
            error: &error,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            pub async fn list_by_id(
                &self,
                es_entity::PaginatedQueryArgs { first, after }: es_entity::PaginatedQueryArgs<cursor::EntityByIdCursor>,
            ) -> Result<es_entity::PaginatedQueryRet<Entity, cursor::EntityByIdCursor>, EsRepoError> {
                let id = if let Some(after) = after {
                    Some(after.id)
                } else {
                    None
                };
                let rows = sqlx::query!(
                    "WITH entities AS (SELECT id FROM entities WHERE (id > $2) OR $2 IS NULL ORDER BY id LIMIT $1) SELECT i.id AS \"id: EntityId\", e.sequence, e.event, e.recorded_at FROM entities i JOIN entity_events e ON i.id = e.id ORDER BY i.id, e.sequence",
                    (first + 1) as i64,
                    id as Option<EntityId>,
                )
                    .fetch_all(self.pool())
                    .await?;

                let (entities, has_next_page) = EntityEvents::load_n::<Entity>(rows.into_iter().map(|r|
                        es_entity::GenericEvent {
                            id: r.id,
                            sequence: r.sequence,
                            event: r.event,
                            recorded_at: r.recorded_at,
                        }), first)?;
                let mut end_cursor = None;
                if let Some(last) = entities.last() {
                    end_cursor = Some(cursor::EntityByIdCursor {
                        id: last.id,
                    });
                }

                Ok(es_entity::PaginatedQueryRet {
                    entities,
                    has_next_page,
                    end_cursor,
                })
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn list_by_fn_name() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let column_type = syn::parse_str("String").unwrap();
        let entity = Ident::new("Entity", Span::call_site());
        let error = Ident::new("EsRepoError", Span::call_site());

        let persist_fn = ListByFn {
            column_name: Ident::new("name", Span::call_site()),
            column_type,
            id: &id_type,
            entity: &entity,
            table_name: "entities",
            events_table_name: "entity_events",
            error: &error,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            pub async fn list_by_name(
                &self,
                es_entity::PaginatedQueryArgs { first, after }: es_entity::PaginatedQueryArgs<cursor::EntityByNameCursor>,
            ) -> Result<es_entity::PaginatedQueryRet<Entity, cursor::EntityByNameCursor>, EsRepoError> {
                let (id, name) = if let Some(after) = after {
                    (Some(after.id), Some(after.name))
                } else {
                    (None, None)
                };
                let rows = sqlx::query!(
                    "WITH entities AS (SELECT name, id FROM entities WHERE ((name, id) > ($3, $2)) OR $2 IS NULL ORDER BY name, id LIMIT $1) SELECT i.id AS \"id: EntityId\", e.sequence, e.event, e.recorded_at FROM entities i JOIN entity_events e ON i.id = e.id ORDER BY name, i.id, e.sequence",
                    (first + 1) as i64,
                    id as Option<EntityId>,
                    name as Option<String>,
                )
                    .fetch_all(self.pool())
                    .await?;

                let (entities, has_next_page) = EntityEvents::load_n::<Entity>(rows.into_iter().map(|r|
                        es_entity::GenericEvent {
                            id: r.id,
                            sequence: r.sequence,
                            event: r.event,
                            recorded_at: r.recorded_at,
                        }), first)?;
                let mut end_cursor = None;
                if let Some(last) = entities.last() {
                    end_cursor = Some(cursor::EntityByNameCursor {
                        id: last.id,
                        name: last.name.clone(),
                    });
                }

                Ok(es_entity::PaginatedQueryRet {
                    entities,
                    has_next_page,
                    end_cursor,
                })
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
