#![recursion_limit = "512"]
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod graphql;
mod local_storage;
mod primitives;
mod sse;
mod webhooks;

use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension, Router, middleware,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};

use jwks_utils::{Claims, JwtDecoderState, RemoteJwksDecoder};
use lana_app::app::LanaApp;

pub use config::*;
use primitives::*;

use std::sync::Arc;

use std::future::Future;

#[instrument(name = "admin_server.run", skip_all)]
pub async fn run<S>(config: AdminServerConfig, app: LanaApp, signal: S) -> anyhow::Result<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    let port = config.port;
    let aud = config.aud.as_ref();

    let jwks_decoder = Arc::new(RemoteJwksDecoder::new(config.jwks_url.clone(), aud));
    let decoder = jwks_decoder.clone();
    tokio::spawn(async move {
        decoder.refresh_keys_periodically().await;
    });
    let schema = graphql::schema(Some(app.clone()));

    let cors = CorsLayer::permissive();

    let mut router = Router::new()
        .route("/health", get(health_check))
        .route("/graphql/stream", post(sse::graphql_sse_post))
        .route(
            "/graphql",
            get(health_check).post(axum::routing::post(graphql_handler)),
        )
        .merge(webhooks::custodians::routes())
        .merge(webhooks::reports::routes())
        .merge(webhooks::sumsub::routes());

    if app.storage().is_local() {
        router = router.merge(local_storage::routes());
    }

    let app = router
        .with_state(JwtDecoderState {
            decoder: jwks_decoder,
        })
        .layer(Extension(schema))
        .layer(Extension(config))
        .layer(Extension(app))
        .layer(middleware::from_fn(
            tracing_utils::http::trace_context_middleware,
        ))
        .layer(cors);

    info!("Starting admin server on port {port}");
    let listener =
        tokio::net::TcpListener::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], port))).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(signal)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminJwtClaims {
    pub subject: String,
}

#[instrument(
    name = "admin_server.graphql",
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
pub async fn graphql_handler(
    schema: Extension<Schema<graphql::Query, graphql::Mutation, graphql::Subscription>>,
    Claims(jwt_claims): Claims<AdminJwtClaims>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    tracing::Span::current().record("jwt.subject", &jwt_claims.subject);

    if let Some(op_name) = req.operation_name.as_ref() {
        tracing::Span::current().record("graphql.operation_name", op_name);
    }

    tracing::Span::current().record("graphql.query", &req.query);

    if let Some(query_type) = req.query.split_whitespace().next() {
        tracing::Span::current().record("graphql.operation_type", query_type);
    }

    // TODO: this should be behind a feature flag or env var
    let variables_str = format!("{:?}", req.variables);
    tracing::Span::current().record("graphql.variables", variables_str.as_str());

    let id = match uuid::Uuid::parse_str(&jwt_claims.subject) {
        Ok(id) => {
            tracing::Span::current().record("user.id", tracing::field::debug(&id));
            id
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                subject = &jwt_claims.subject,
                "Failed to parse UUID from JWT subject"
            );
            return async_graphql::Response::from_errors(vec![async_graphql::ServerError::new(
                e.to_string(),
                None,
            )])
            .into();
        }
    };

    let auth_context = AdminAuthContext::new(id);
    req = req.data(auth_context);

    let response = schema.execute(req).await;
    if !response.errors.is_empty() {
        for err in &response.errors {
            tracing::warn!(
                path = ?err.path,
                locations = ?err.locations,
                extensions = ?err.extensions,
                "{}", err.message
            );
        }
    }
    response.into()
}

async fn health_check() -> &'static str {
    "OK"
}
