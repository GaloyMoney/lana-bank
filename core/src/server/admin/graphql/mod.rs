mod account;
mod account_set;
mod convert;
mod loader;
mod loan;
mod schema;
mod shareholder_equity;
mod terms;
mod user;

use async_graphql::{dataloader::*, *};

pub use schema::*;

use crate::app::LavaApp;
use loader::ChartOfAccountsLoader;

pub fn schema(app: Option<LavaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let schema = Schema::build(Query, Mutation, EmptySubscription);
    if let Some(app) = app {
        schema
            .data(
                DataLoader::new(
                    ChartOfAccountsLoader {
                        cala: app.ledger.clone(),
                    },
                    tokio::task::spawn,
                )
                // Set delay to 0 as per https://github.com/async-graphql/async-graphql/issues/1306
                .delay(std::time::Duration::from_secs(0)),
            )
            .data(app)
            .finish()
    } else {
        schema.finish()
    }
}
