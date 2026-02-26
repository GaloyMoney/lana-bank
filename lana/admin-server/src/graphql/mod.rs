mod access;
mod accounting;
mod approval_process;
mod audit;
mod committee;
mod credit_facility;
mod customer;
mod dashboard;
mod deposit;
mod deposit_account;
mod domain_config;
mod loader;
mod me;
mod policy;
mod price;
mod primitives;
mod prospect;
mod public_id;
mod reports;
mod schema;
mod withdrawal;

mod custody;

use async_graphql::*;

use loader::LanaLoader;
pub use schema::*;

use lana_app::app::LanaApp;

pub fn schema(app: Option<LanaApp>) -> Schema<Query, Mutation, Subscription> {
    let mut schema_builder = Schema::build(Query::default(), Mutation::default(), Subscription)
        .extension(extensions::Tracing);

    if let Some(app) = app {
        schema_builder = schema_builder.data(LanaLoader::new(&app)).data(app);
    }

    schema_builder.finish()
}
