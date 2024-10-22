use convert_case::{Case, Casing};
use darling::ToTokens;
use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};

use super::options::*;

pub struct CursorStruct<'a> {
    pub id: &'a syn::Ident,
    pub entity: &'a syn::Ident,
    pub column: &'a Column,
}

impl<'a> CursorStruct<'a> {
    fn name(&self) -> String {
        format!(
            "{}By{}Cursor",
            self.entity,
            self.column.name().to_string().to_case(Case::UpperCamel)
        )
    }

    pub fn ident(&self) -> syn::Ident {
        syn::Ident::new(&self.name(), Span::call_site())
    }

    pub fn select_columns(&self) -> String {
        if self.column.is_id() {
            format!("id")
        } else {
            format!("{}, id", self.column.name())
        }
    }

    pub fn condition(&self, offset: u32) -> String {
        if self.column.is_id() {
            format!("(id > ${}) OR ${} IS NULL", offset + 2, offset + 2)
        } else {
            format!(
                "(({}, id) > (${}, ${})) OR ${} IS NULL",
                self.column.name(),
                offset + 3,
                offset + 2,
                offset + 2
            )
        }
    }

    pub fn query_arg_tokens(&self) -> TokenStream {
        let id = self.id;

        if self.column.is_id() {
            quote! {
                (first + 1) as i64,
                id as Option<#id>,
            }
        } else {
            let column_name = self.column.name();
            let column_type = self.column.ty();
            quote! {
                (first + 1) as i64,
                id as Option<#id>,
                #column_name as Option<#column_type>,
            }
        }
    }

    pub fn destructure_tokens(&self) -> TokenStream {
        let column_name = self.column.name();

        let mut after_args = quote! {
            (id, #column_name)
        };
        let mut after_destruction = quote! {
            (Some(after.id), Some(after.#column_name))
        };
        let mut after_default = quote! {
            (None, None)
        };

        if self.column.is_id() {
            after_args = quote! {
                id
            };
            after_destruction = quote! {
                Some(after.id)
            };
            after_default = quote! {
                None
            };
        }

        quote! {
            let es_entity::PaginatedQueryArgs { first, after } = cursor;
            let #after_args = if let Some(after) = after {
                #after_destruction
            } else {
                #after_default
            };
        }
    }
}

impl<'a> ToTokens for CursorStruct<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entity = self.entity;
        let accessor = &self.column.accessor();
        let ident = self.ident();
        let id = &self.id;

        let (field, from_impl) = if self.column.is_id() {
            (quote! {}, quote! {})
        } else {
            let column_name = self.column.name();
            let column_type = self.column.ty();
            (
                quote! {
                    pub #column_name: #column_type,
                },
                quote! {
                    #column_name: entity.#accessor.clone(),
                },
            )
        };

        tokens.append_all(quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct #ident {
                pub id: #id,
                #field
            }

            impl From<&#entity> for #ident {
                fn from(entity: &#entity) -> Self {
                    Self {
                        id: entity.id.clone(),
                        #from_impl
                    }
                }
            }
        });
    }
}

pub struct ListByFn<'a> {
    id: &'a syn::Ident,
    entity: &'a syn::Ident,
    column: &'a Column,
    table_name: &'a str,
    error: &'a syn::Type,
}

impl<'a> ListByFn<'a> {
    pub fn new(column: &'a Column, opts: &'a RepositoryOptions) -> Self {
        Self {
            column,
            id: opts.id(),
            entity: opts.entity(),
            table_name: opts.table_name(),
            error: opts.err(),
        }
    }

    pub fn cursor(&'a self) -> CursorStruct<'a> {
        CursorStruct {
            column: self.column,
            id: self.id,
            entity: self.entity,
        }
    }
}

impl<'a> ToTokens for ListByFn<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = self.id;
        let entity = self.entity;
        let column_name = self.column.name();
        let column_type = self.column.ty();
        let cursor = self.cursor();
        let cursor_ident = cursor.ident();
        let error = self.error;

        let fn_name = syn::Ident::new(&format!("list_by_{}", column_name), Span::call_site());
        let name = column_name.to_string();
        let destructure_tokens = self.cursor().destructure_tokens();
        let select_columns = cursor.select_columns();
        let condition = cursor.condition(0);
        let arg_tokens = cursor.query_arg_tokens();

        let query = format!(
            r#"SELECT {} FROM {} WHERE {} ORDER BY {} LIMIT $1"#,
            select_columns, self.table_name, condition, select_columns
        );

        tokens.append_all(quote! {
            pub async fn #fn_name(
                &self,
                cursor: es_entity::PaginatedQueryArgs<cursor::#cursor_ident>,
            ) -> Result<es_entity::PaginatedQueryRet<#entity, cursor::#cursor_ident>, #error> {
                #destructure_tokens

                let (entities, has_next_page) = es_entity::es_query!(
                    self.pool(),
                    #query,
                    #arg_tokens
                )
                    .fetch_n(first)
                    .await?;

                let end_cursor = entities.last().map(cursor::#cursor_ident::from);

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
        let by_column = Column::for_id(syn::parse_str("EntityId").unwrap());

        let cursor = CursorStruct {
            column: &by_column,
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

            impl From<&Entity> for EntityByIdCursor {
                fn from(entity: &Entity) -> Self {
                    Self {
                        id: entity.id.clone(),
                    }
                }
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn cursor_struct_by_created_at() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let entity = Ident::new("Entity", Span::call_site());
        let by_column = Column::for_created_at();

        let cursor = CursorStruct {
            column: &by_column,
            id: &id_type,
            entity: &entity,
        };

        let mut tokens = TokenStream::new();
        cursor.to_tokens(&mut tokens);

        let expected = quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct EntityByCreatedAtCursor {
                pub id: EntityId,
                pub created_at: chrono::DateTime<chrono::Utc>,
            }

            impl From<&Entity> for EntityByCreatedAtCursor {
                fn from(entity: &Entity) -> Self {
                    Self {
                        id: entity.id.clone(),
                        created_at: entity.events()
                            .entity_first_persisted_at()
                            .expect("entity not persisted")
                            .clone(),
                    }
                }
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn list_by_fn() {
        let id_type = Ident::new("EntityId", Span::call_site());
        let entity = Ident::new("Entity", Span::call_site());
        let error = syn::parse_str("es_entity::EsRepoError").unwrap();
        let column = Column::for_id(syn::parse_str("EntityId").unwrap());

        let persist_fn = ListByFn {
            column: &column,
            id: &id_type,
            entity: &entity,
            table_name: "entities",
            error: &error,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            pub async fn list_by_id(
                &self,
                cursor: es_entity::PaginatedQueryArgs<cursor::EntityByIdCursor>,
            ) -> Result<es_entity::PaginatedQueryRet<Entity, cursor::EntityByIdCursor>, es_entity::EsRepoError> {
                let es_entity::PaginatedQueryArgs { first, after } = cursor;
                let id = if let Some(after) = after {
                    Some(after.id)
                } else {
                    None
                };
                let (entities, has_next_page) = es_entity::es_query!(
                    self.pool(),
                    "SELECT id FROM entities WHERE (id > $2) OR $2 IS NULL ORDER BY id LIMIT $1",
                    (first + 1) as i64,
                    id as Option<EntityId>,
                )
                    .fetch_n(first)
                    .await?;
                let end_cursor = entities.last().map(cursor::EntityByIdCursor::from);
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
        let entity = Ident::new("Entity", Span::call_site());
        let error = syn::parse_str("es_entity::EsRepoError").unwrap();
        let column = Column::new(
            syn::Ident::new("name", proc_macro2::Span::call_site()),
            syn::parse_str("String").unwrap(),
        );

        let persist_fn = ListByFn {
            column: &column,
            id: &id_type,
            entity: &entity,
            table_name: "entities",
            error: &error,
        };

        let mut tokens = TokenStream::new();
        persist_fn.to_tokens(&mut tokens);

        let expected = quote! {
            pub async fn list_by_name(
                &self,
                cursor: es_entity::PaginatedQueryArgs<cursor::EntityByNameCursor>,
            ) -> Result<es_entity::PaginatedQueryRet<Entity, cursor::EntityByNameCursor>, es_entity::EsRepoError> {
                let es_entity::PaginatedQueryArgs { first, after } = cursor;
                let (id, name) = if let Some(after) = after {
                    (Some(after.id), Some(after.name))
                } else {
                    (None, None)
                };

                let (entities, has_next_page) = es_entity::es_query!(
                        self.pool(),
                        "SELECT name, id FROM entities WHERE ((name, id) > ($3, $2)) OR $2 IS NULL ORDER BY name, id LIMIT $1",
                        (first + 1) as i64,
                        id as Option<EntityId>,
                        name as Option<String>,
                )
                    .fetch_n(first)
                    .await?;

                let end_cursor = entities.last().map(cursor::EntityByNameCursor::from);

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
