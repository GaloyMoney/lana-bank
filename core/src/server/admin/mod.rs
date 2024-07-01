pub mod graphql;

mod auth;
mod config;

use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use auth::auth_routes;
use axum::{routing::get, Extension, Router};
use axum_extra::headers::HeaderMap;
use tower_http::cors::CorsLayer;

use crate::app::LavaApp;

pub use config::*;

pub async fn run(config: AdminServerConfig, app: LavaApp) -> anyhow::Result<()> {
    let schema = graphql::schema(Some(app.clone()));

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route(
            "/graphql",
            get(playground).post(axum::routing::post(graphql_handler)),
        )
        .layer(Extension(schema))
        .merge(auth_routes())
        .layer(Extension(config.clone()))
        .layer(Extension(app))
        .layer(cors);

    println!("Starting admin server on port {}", config.port);
    let listener =
        tokio::net::TcpListener::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], config.port)))
            .await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

pub async fn graphql_handler(
    headers: HeaderMap,
    schema: Extension<Schema<graphql::Query, graphql::Mutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    lava_tracing::http::extract_tracing(&headers);
    let req = req.into_inner();
    schema.execute(req).await.into()
}

async fn playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")
            .with_setting("request.credentials", "include"),
    ))
}
