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

use crate::{
    AdminJwtClaims,
    graphql::{Mutation, Query, Subscription},
    primitives::AdminAuthContext,
};
use jwks_utils::Claims;

type Schema = async_graphql::Schema<Query, Mutation, Subscription>;

pub async fn graphql_sse_post(
    schema: Extension<Schema>,
    Claims(jwt_claims): Claims<AdminJwtClaims>,
    body: Result<Json<Request>, JsonRejection>,
) -> impl IntoResponse {
    let responses = match body {
        Ok(Json(req)) => match uuid::Uuid::parse_str(&jwt_claims.subject) {
            Ok(id) => Executor::execute_stream(&*schema, req.data(AdminAuthContext::new(id)), None),
            Err(e) => error_stream(format!("Invalid UUID in JWT subject: {e}")),
        },
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
