use std::convert::Infallible;

use async_graphql::{Executor, Request, Response, ServerError};
use axum::{
    Extension,
    extract::{Json, rejection::JsonRejection},
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use futures::stream::{self, StreamExt};
use tracing::instrument;

use crate::{
    AdminJwtClaims,
    graphql::{Mutation, Query, Subscription},
    primitives::AdminAuthContext,
};
use jwks_utils::Claims;

type Schema = async_graphql::Schema<Query, Mutation, Subscription>;

#[instrument(
    name = "admin_server.graphql_sse",
    skip_all,
    fields(
        graphql.operation_name,
        graphql.operation_type,
        graphql.query,
        graphql.variables,
        jwt.subject,
        user.id,
        error,
        error.level,
        error.message,
    )
)]
#[es_entity::es_event_context]
pub async fn graphql_sse_post(
    schema: Extension<Schema>,
    Claims(jwt_claims): Claims<AdminJwtClaims>,
    body: Result<Json<Request>, JsonRejection>,
) -> impl IntoResponse {
    tracing::Span::current().record("jwt.subject", &jwt_claims.subject);

    let responses = match body {
        Ok(Json(req)) => {
            if let Some(op_name) = req.operation_name.as_ref() {
                tracing::Span::current().record("graphql.operation_name", op_name);
            }

            tracing::Span::current().record("graphql.query", &req.query);
            if let Some(query_type) = req.query.split_whitespace().next() {
                tracing::Span::current().record("graphql.operation_type", query_type);
            }

            let variables_str = format!("{:?}", req.variables);
            tracing::Span::current().record("graphql.variables", variables_str.as_str());

            match uuid::Uuid::parse_str(&jwt_claims.subject) {
                Ok(id) => {
                    tracing::Span::current().record("user.id", tracing::field::debug(&id));
                    Executor::execute_stream(&*schema, req.data(AdminAuthContext::new(id)), None)
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        subject = &jwt_claims.subject,
                        "Failed to parse UUID from JWT subject"
                    );
                    error_stream(format!("Invalid UUID in JWT subject: {e}"))
                }
            }
        }
        Err(e) => error_stream(e.to_string()),
    };

    let events = responses.map(|r| {
        Ok::<_, Infallible>(
            Event::default()
                .event("next")
                .json_data(&r)
                .unwrap_or_else(|_| {
                    Event::default()
                        .event("next")
                        .data(r#"{"errors":[{"message":"Serialization error"}]}"#)
                }),
        )
    });

    let complete = stream::iter([Ok::<_, Infallible>(
        Event::default().event("complete").data(""),
    )]);

    Sse::new(events.chain(complete))
}

fn error_stream(message: String) -> futures::stream::BoxStream<'static, Response> {
    stream::iter([Response::from_errors(vec![ServerError::new(message, None)])]).boxed()
}
