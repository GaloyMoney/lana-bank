use std::convert::Infallible;

use async_graphql::parser::types::OperationType;
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

pub async fn graphql_sse(
    schema: Extension<Schema>,
    Claims(jwt_claims): Claims<AdminJwtClaims>,
    body: Result<Json<Request>, JsonRejection>,
) -> impl IntoResponse {
    let responses = match body {
        Ok(Json(req)) => stream_for_request(&schema, &jwt_claims, req),
        Err(e) => error_stream(ServerError::new(e.to_string(), None)),
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

fn stream_for_request(
    schema: &Schema,
    jwt_claims: &AdminJwtClaims,
    mut req: Request,
) -> futures::stream::BoxStream<'static, Response> {
    match operation_type_for_request(&mut req) {
        Ok(OperationType::Subscription) => match uuid::Uuid::parse_str(&jwt_claims.subject) {
            Ok(id) => Executor::execute_stream(schema, req.data(AdminAuthContext::new(id)), None),
            Err(e) => error_stream(ServerError::new(
                format!("Invalid UUID in JWT subject: {e}"),
                None,
            )),
        },
        Ok(OperationType::Query | OperationType::Mutation) => error_stream(ServerError::new(
            "Queries and mutations are not supported on this transport.",
            None,
        )),
        Err(err) => error_stream(err),
    }
}

fn operation_type_for_request(req: &mut Request) -> Result<OperationType, ServerError> {
    let op_name = req.operation_name.take();
    let result = (|| {
        let doc = req.parsed_query()?;
        match op_name.as_deref() {
            Some(name) => doc
                .operations
                .iter()
                .find_map(|(doc_name, op)| {
                    (doc_name.map(|n| n.as_str()) == Some(name)).then_some(op.node.ty)
                })
                .ok_or_else(|| {
                    ServerError::new(format!(r#"Unknown operation named "{}""#, name), None)
                }),
            None => {
                let mut iter = doc.operations.iter();
                let (_, op) = iter
                    .next()
                    .ok_or_else(|| ServerError::new("No operations found", None))?;
                if iter.next().is_some() {
                    Err(ServerError::new(
                        "Operation name is required for documents with multiple operations",
                        None,
                    ))
                } else {
                    Ok(op.node.ty)
                }
            }
        }
    })();
    req.operation_name = op_name;
    result
}

fn error_stream(error: ServerError) -> futures::stream::BoxStream<'static, Response> {
    stream::iter([Response::from_errors(vec![error])]).boxed()
}
