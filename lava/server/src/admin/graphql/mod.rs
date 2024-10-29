mod account;
mod account_set;
mod approval_process;
pub mod audit;
mod committee;
pub mod credit_facility;
mod customer;
mod deposit;
pub mod loader;
mod loan;
mod policy;
mod price;
mod report;
mod schema;
mod shareholder_equity;
mod terms_template;
pub mod user;
mod withdraw;

use async_graphql::{dataloader::*, *};

use loader::LavaDataLoader;
pub use schema::*;

use lava_app::app::LavaApp;

pub fn schema(app: Option<LavaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let mut schema_builder = Schema::build(Query, Mutation, EmptySubscription);

    if let Some(app) = app {
        schema_builder = schema_builder
            .data(
                DataLoader::new(LavaDataLoader { app: app.clone() }, tokio::task::spawn)
                    // Set delay to 0 as per https://github.com/async-graphql/async-graphql/issues/1306
                    .delay(std::time::Duration::from_secs(0)),
            )
            .data(app);
    }

    schema_builder.finish()
}
