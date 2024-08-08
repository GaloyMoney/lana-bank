mod account;
mod account_set;
mod audit;
mod customer;
mod deposit;
mod loan;
mod schema;
mod shareholder_equity;
mod terms;
mod user;

use async_graphql::*;

pub use schema::*;

use crate::app::LavaApp;

pub fn schema(app: Option<LavaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let mut schema_builder = Schema::build(Query, Mutation, EmptySubscription);

    if let Some(app) = app {
        schema_builder = schema_builder.data(app);
    }

    schema_builder.finish()
}
