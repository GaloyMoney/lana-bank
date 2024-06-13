pub mod graphql;

mod config;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::get, Extension, Router};
use axum_extra::headers::HeaderMap;
use uuid::Uuid;

use crate::{app::LavaApp, primitives::UserId};

pub use config::*;

pub async fn run(config: PublicServerConfig, app: LavaApp) -> anyhow::Result<()> {
    let schema = graphql::schema(Some(app.clone()));

    let app = Router::new()
        .route(
            "/graphql",
            get(playground).post(axum::routing::post(graphql_handler)),
        )
        .layer(Extension(schema));

    println!("Starting public graphql server on port {}", config.port);
    let listener =
        tokio::net::TcpListener::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], config.port)))
            .await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

pub struct UserContext {
    pub user_id: Option<UserId>,
}

impl UserContext {
    pub fn new(user_id: Option<UserId>) -> Self {
        Self { user_id }
    }
}

pub async fn graphql_handler(
    headers: HeaderMap,
    schema: Extension<Schema<graphql::Query, graphql::Mutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    lava_tracing::http::extract_tracing(&headers);
    let req = req.into_inner();

    let user_id = headers
        .get("X-USER-ID")
        .and_then(|header| header.to_str().ok())
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
        .map(UserId::from);

    let context = UserContext::new(user_id);

    schema.execute(req.data(context)).await.into()
}

async fn playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}
