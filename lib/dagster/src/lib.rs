#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod error;
pub mod graphql_client;

pub use config::DagsterConfig;
pub use error::DagsterError;
use graphql_client::GraphqlClient;

#[derive(Clone)]
pub struct Dagster {
    graphql: GraphqlClient,
}

impl Dagster {
    pub fn new(config: DagsterConfig) -> Self {
        Self {
            graphql: GraphqlClient::new(config),
        }
    }

    pub fn graphql(&self) -> &GraphqlClient {
        &self.graphql
    }
}
