pub mod graphql;

mod config;

use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{http::HeaderValue, response::IntoResponse, routing::get, Extension, Router};
use axum_extra::headers::HeaderMap;

use crate::app::LavaApp;

pub use config::*;

pub async fn run(config: AdminServerConfig, app: LavaApp) -> anyhow::Result<()> {
    let schema = graphql::schema(Some(app.clone()));

    let app = Router::new()
        .route(
            "/graphql",
            get(playground)
                .post(axum::routing::post(graphql_handler))
                .options(cors_preflight),
        )
        .layer(Extension(schema));

    println!("Starting graphql server on port {}", config.port);
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
) -> impl IntoResponse {
    lava_tracing::http::extract_tracing(&headers);
    let req = req.into_inner();
    let response = schema.execute(req).await;
    let mut res = GraphQLResponse::from(response).into_response();

    res.headers_mut().insert(
        "access-control-allow-origin",
        HeaderValue::from_static("https://studio.apollographql.com"),
    );
    res.headers_mut().insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );

    res
}

async fn playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

async fn cors_preflight() -> impl IntoResponse {
    axum::response::Response::builder()
        .header(
            "access-control-allow-origin",
            "https://studio.apollographql.com",
        )
        .header("access-control-allow-methods", "POST, OPTIONS")
        .header("access-control-allow-credentials", "true")
        .header(
            "access-control-allow-headers",
            "Content-Type, Authorization",
        )
        .status(204)
        .body(axum::body::Body::empty())
        .unwrap()
}
