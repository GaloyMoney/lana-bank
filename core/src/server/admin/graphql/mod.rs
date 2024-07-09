mod account;
mod account_set;
mod convert;
mod loan;
mod schema;
mod shareholder_equity;
mod terms;
mod user;

use async_graphql::*;

pub use schema::*;

use crate::app::LavaApp;

pub fn schema(app: Option<LavaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let schema = Schema::build(Query, Mutation, EmptySubscription);
    if let Some(app) = app {
        schema.data(app).finish()
    } else {
        schema.finish()
    }
}
