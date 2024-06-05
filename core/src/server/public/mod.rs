pub mod graphql;

mod config;

use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};

use axum_extra::headers::HeaderMap;
use serde_json::json;

use crate::app::LavaApp;

pub use config::*;

pub async fn run(config: PublicServerConfig, app: LavaApp) -> anyhow::Result<()> {
    let schema = graphql::schema(Some(app.clone()));

    let app = Router::new()
        .route(
            "/graphql",
            get(studio_explorer).post(axum::routing::post(graphql_handler)),
        )
        .layer(Extension(schema))
        .layer(Extension(config.clone()));

    println!("Starting public graphql server on port {}", config.port);
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

async fn studio_explorer(config: Extension<PublicServerConfig>) -> impl IntoResponse {
    let initial_state = json!({
            "document": r#"query SchemaDescription{
    __schema {
        description
    }
}
"#,
    });

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <body style="margin: 0; overflow-x: hidden; overflow-y: hidden">
        <div id="sandbox" style="height:100vh; width:100vw;"></div>
        <script src="https://embeddable-sandbox.cdn.apollographql.com/_latest/embeddable-sandbox.umd.production.min.js"></script>
        <script>
            new window.EmbeddedSandbox({{
            target: "\#sandbox",
            // Pass through your server href if you are embedding on an endpoint.
            // Otherwise, you can pass whatever endpoint you want Sandbox to start up with here.
            initialEndpoint: "{}:{}/graphql",
            endpointIsEditable: "false",
            initialState: {},
            }});
            // advanced options: https://www.apollographql.com/docs/studio/explorer/sandbox#embedding-sandbox
        </script>
    </body>
</html>
"#,
        config.endpoint, config.port, initial_state,
    );

    Html(html_content).into_response()
}
