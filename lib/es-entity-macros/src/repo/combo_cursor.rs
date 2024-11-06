use darling::ToTokens;
use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};

use super::{list_by_fn::CursorStruct, options::*};

pub struct ComboCursor<'a> {
    pub entity: &'a syn::Ident,
    pub cursors: Vec<CursorStruct<'a>>,
}

impl<'a> ComboCursor<'a> {
    pub fn new(opts: &'a RepositoryOptions, cursors: Vec<CursorStruct<'a>>) -> Self {
        Self {
            entity: &opts.entity(),
            cursors,
        }
    }

    pub fn ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}ComboCursor", self.entity), Span::call_site())
    }

    pub fn variants(&self) -> TokenStream {
        let variants = self
            .cursors
            .iter()
            .map(|cursor| {
                let tag =
                    syn::Ident::new(&format!("By{}", cursor.column.name()), Span::call_site());
                let ident = cursor.ident();
                quote! {
                    #tag(#ident),
                }
            })
            .collect::<TokenStream>();

        quote! {
            #variants
        }
    }

    pub fn from_impls(&self) -> TokenStream {
        let self_ident = self.ident();
        let from_impls = self
            .cursors
            .iter()
            .map(|cursor| {
                let tag =
                    syn::Ident::new(&format!("By{}", cursor.column.name()), Span::call_site());
                let ident = cursor.ident();
                quote! {
                    impl From<#ident> for #self_ident {
                        fn from(cursor: #ident) -> Self {
                            Self::#tag(cursor)
                        }
                    }

                    impl TryFrom<#self_ident> for #ident {
                        type Error = String;

                        fn try_from(cursor: #self_ident) -> Result<Self, Self::Error> {
                            match cursor {
                                #self_ident::#tag(cursor) => Ok(cursor),
                                _ => Err(format!("could not convert {} to {}", stringify!(#self_ident), stringify!(#ident))),
                            }
                        }
                    }
                }
            })
            .collect::<TokenStream>();

        quote! {
            #from_impls
        }
    }
}

impl<'a> ToTokens for ComboCursor<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident();
        let variants = self.variants();
        let from_impls = self.from_impls();

        tokens.append_all(quote! {
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            #[allow(clippy::enum_variant_names)]
            #[serde(tag = "type")]
            pub enum #ident {
                #variants
            }

            #from_impls

            impl es_entity::graphql::async_graphql::connection::CursorType for #ident {
                type Error = String;

                fn encode_cursor(&self) -> String {
                    use es_entity::graphql::base64::{engine::general_purpose, Engine as _};
                    let json = es_entity::prelude::serde_json::to_string(&self).expect("could not serialize token");
                    general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
                }

                fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
                    use es_entity::graphql::base64::{engine::general_purpose, Engine as _};
                    let bytes = general_purpose::STANDARD_NO_PAD
                        .decode(s.as_bytes())
                        .map_err(|e| e.to_string())?;
                    let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
                    es_entity::prelude::serde_json::from_str(&json).map_err(|e| e.to_string())
                }
            }
        });
    }
}
