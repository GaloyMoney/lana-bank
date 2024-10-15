use darling::{FromDeriveInput, FromMeta};
use quote::quote;
use syn::{parse::Parse, Expr, Ident, Token};

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(es_repo), map = "Self::update_defaults")]
pub struct RepositoryOptions {
    pub ident: syn::Ident,
    #[darling(default)]
    pub indexes: Indexes,
    #[darling(rename = "entity")]
    entity_ident: syn::Ident,
    #[darling(default, rename = "new")]
    new_entity_ident: Option<syn::Ident>,
    #[darling(default, rename = "event")]
    event_ident: Option<syn::Ident>,
    #[darling(default, rename = "id")]
    id_ident: Option<syn::Ident>,
    #[darling(default, rename = "err")]
    err_ident: Option<syn::Ident>,
    #[darling(default, rename = "tbl")]
    table_name: Option<String>,
    #[darling(default, rename = "events_tbl")]
    events_table_name: Option<String>,
}

impl RepositoryOptions {
    fn update_defaults(mut self) -> Self {
        let entity_name = self.entity_ident.to_string();
        if self.new_entity_ident.is_none() {
            self.new_entity_ident = Some(syn::Ident::new(
                &format!("New{}", entity_name),
                proc_macro2::Span::call_site(),
            ));
        }
        if self.event_ident.is_none() {
            self.event_ident = Some(syn::Ident::new(
                &format!("{}Event", entity_name),
                proc_macro2::Span::call_site(),
            ));
        }
        if self.id_ident.is_none() {
            self.id_ident = Some(syn::Ident::new(
                &format!("{}Id", entity_name),
                proc_macro2::Span::call_site(),
            ));
        }
        if self.err_ident.is_none() {
            self.err_ident = Some(syn::Ident::new(
                "EsRepoError",
                proc_macro2::Span::call_site(),
            ));
        }
        if self.table_name.is_none() {
            self.table_name = Some(pluralizer::pluralize(&entity_name.to_lowercase(), 2, false));
        }
        if self.events_table_name.is_none() {
            self.events_table_name = Some(format!("{}_events", entity_name.to_lowercase()));
        }
        self
    }

    pub fn entity(&self) -> &syn::Ident {
        &self.entity_ident
    }

    pub fn table_name(&self) -> &str {
        self.table_name.as_ref().expect("Table name is not set")
    }

    pub fn id(&self) -> &syn::Ident {
        self.id_ident.as_ref().expect("ID identifier is not set")
    }

    pub fn event(&self) -> &syn::Ident {
        self.event_ident
            .as_ref()
            .expect("Event identifier is not set")
    }

    pub fn events_table_name(&self) -> &str {
        self.events_table_name
            .as_ref()
            .expect("Events table name is not set")
    }

    pub fn new_entity(&self) -> &syn::Ident {
        self.new_entity_ident
            .as_ref()
            .expect("New entity identifier is not set")
    }

    pub fn err(&self) -> &syn::Ident {
        self.err_ident
            .as_ref()
            .expect("Error identifier is not set")
    }
}

#[derive(Debug, Clone)]
pub struct Indexes {
    pub columns: Vec<IndexColumn>,
}

#[derive(Debug, Clone)]
pub struct IndexColumn {
    pub name: Ident,
    pub ty: Option<Ident>,
}

impl Default for Indexes {
    fn default() -> Self {
        Self {
            columns: vec![IndexColumn {
                name: syn::Ident::new("id", proc_macro2::Span::call_site()),
                ty: None,
            }],
        }
    }
}

impl Indexes {
    pub fn query_args(&self) -> Vec<proc_macro2::TokenStream> {
        self.columns
            .iter()
            .map(|column| {
                let ident = &column.name;
                match &column.ty {
                    Some(ty) => quote! {
                        #ident as &#ty
                    },
                    None => quote! {
                        #ident
                    },
                }
            })
            .collect()
    }
}

impl Parse for IndexColumn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let ty = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(IndexColumn { name, ty })
    }
}

impl FromMeta for Indexes {
    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let columns = items
            .iter()
            .map(|item| match item {
                darling::ast::NestedMeta::Meta(syn::Meta::Path(path)) => Ok(IndexColumn {
                    name: path.get_ident().cloned().ok_or_else(|| {
                        darling::Error::custom("Expected identifier").with_span(path)
                    })?,
                    ty: None,
                }),
                darling::ast::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                    let name = name_value.path.get_ident().cloned().ok_or_else(|| {
                        darling::Error::custom("Expected identifier").with_span(&name_value.path)
                    })?;
                    let ty = match &name_value.value {
                        Expr::Path(expr_path) => {
                            expr_path.path.get_ident().cloned().ok_or_else(|| {
                                darling::Error::custom("Expected identifier for type")
                                    .with_span(&expr_path.path)
                            })
                        }
                        Expr::Lit(expr_lit) => {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                Ok(Ident::new(&lit_str.value(), lit_str.span()))
                            } else {
                                Err(darling::Error::custom("Expected string literal for type")
                                    .with_span(&expr_lit.lit))
                            }
                        }
                        _ => Err(darling::Error::custom(
                            "Expected identifier or string literal for type",
                        )
                        .with_span(&name_value.value)),
                    }?;
                    Ok(IndexColumn { name, ty: Some(ty) })
                }
                _ => Err(
                    darling::Error::custom("Expected identifier or name-value pair")
                        .with_span(item),
                ),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Indexes { columns })
    }
}
