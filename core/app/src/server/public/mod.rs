pub mod graphql;

mod config;

use async_graphql::ServerError;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::get, Extension, Router};
use axum_extra::headers::HeaderMap;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{app::LavaApp, primitives::CustomerId};

use super::jwks::{Claims, JwtDecoderState, RemoteJwksDecoder};

pub use config::*;

use std::sync::Arc;

pub async fn run(config: PublicServerConfig, app: LavaApp) -> anyhow::Result<()> {
    let port = config.port;
    let aud = config.aud.as_ref();

    let jwks_decoder = Arc::new(RemoteJwksDecoder::new(config.jwks_url.clone(), aud));
    let decoder = jwks_decoder.clone();
    tokio::spawn(async move {
        decoder.refresh_keys_periodically().await;
    });

    let schema = graphql::schema(Some(app.clone()));

    let app = Router::new()
        .route(
            "/graphql",
            get(playground).post(axum::routing::post(graphql_handler)),
        )
        .with_state(JwtDecoderState {
            decoder: jwks_decoder,
        })
        .layer(Extension(schema));

    println!("Starting public server on port {}", port);
    let listener =
        tokio::net::TcpListener::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], port))).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

pub struct PublicAuthContext {
    pub customer_id: CustomerId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicJwtClaims {
    pub sub: String,
}

#[instrument(name = "lava.server.public.graphql", skip_all, fields(error, error.level, error.message))]
pub async fn graphql_handler(
    headers: HeaderMap,
    schema: Extension<Schema<graphql::Query, graphql::Mutation, EmptySubscription>>,
    Claims(jwt_claims): Claims<PublicJwtClaims>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    tracing_utils::http::extract_tracing(&headers);
    let mut req = req.into_inner();

    let customer_id: CustomerId = match jwt_claims.sub.parse() {
        Ok(user_id) => user_id,
        Err(_) => {
            let error = ServerError::new("Invalid user id", None);
            let response = async_graphql::Response::from_errors(vec![error]);
            return GraphQLResponse::from(response);
        }
    };

    let auth_context = PublicAuthContext { customer_id };
    req = req.data(auth_context);

    schema.execute(req).await.into()
}

async fn playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}
