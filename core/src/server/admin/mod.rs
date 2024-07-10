pub mod graphql;

mod auth;
mod config;
mod sumsub;

use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use auth::auth_routes;
use axum::{routing::get, Extension, Router};
use axum_extra::headers::HeaderMap;
use sumsub::sumsub_routes;
use tower_http::cors::CorsLayer;

use crate::app::LavaApp;

pub use config::*;

use super::jwks::{Claims, JwtClaims, JwtDecoderState, RemoteJwksDecoder};

use std::sync::Arc;

pub async fn run(config: AdminServerConfig, app: LavaApp) -> anyhow::Result<()> {
    let port = config.port;
    let aud = config.aud.as_ref();

    let jwks_decoder = Arc::new(RemoteJwksDecoder::new(config.jwks_url.clone(), aud));
    let decoder = jwks_decoder.clone();
    tokio::spawn(async move {
        decoder.refresh_keys_periodically().await;
    });

    let schema = graphql::schema(Some(app.clone()));

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route(
            "/graphql",
            get(playground).post(axum::routing::post(graphql_handler)),
        )
        .with_state(JwtDecoderState {
            decoder: jwks_decoder,
        })
        .layer(Extension(schema))
        .merge(auth_routes())
        .merge(sumsub_routes())
        .layer(Extension(config))
        .layer(Extension(app))
        .layer(cors);

    println!("Starting admin server on port {}", port);
    let listener =
        tokio::net::TcpListener::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], port))).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

pub struct AdminAuthContext {
    pub sub: String,
}

pub async fn graphql_handler(
    headers: HeaderMap,
    schema: Extension<Schema<graphql::Query, graphql::Mutation, EmptySubscription>>,
    Claims(jwt_claims): Claims<JwtClaims>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    lava_tracing::http::extract_tracing(&headers);
    let mut req = req.into_inner();

    let sub: String = jwt_claims.sub;

    let auth_context = AdminAuthContext { sub };
    req = req.data(auth_context);

    schema.execute(req).await.into()
}

async fn playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/admin/graphql")
            .with_setting("request.credentials", "include"),
    ))
}
