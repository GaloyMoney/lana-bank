use std::{fmt::Display, sync::Arc};

use async_graphql::ErrorExtensions;

use crate::primitives::*;

pub struct GqlError(pub async_graphql::Error);

impl From<GqlError> for async_graphql::Error {
    fn from(e: GqlError) -> Self {
        e.0
    }
}

impl<T> From<T> for GqlError
where
    T: tracing_utils::ErrorSeverity + Display + Send + Sync + 'static,
{
    fn from(e: T) -> Self {
        let level = e.severity().as_str();
        let gql_error = async_graphql::Error {
            message: e.to_string(),
            source: Some(Arc::new(e)),
            extensions: None,
        }
        .extend_with(|_, ext| {
            ext.set(ERROR_EXTENSIONS_LEVEL_KEY, level);
        });

        GqlError(gql_error)
    }
}
