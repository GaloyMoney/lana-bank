mod account;
mod approval_process;
mod approval_rules;
mod audit;
mod authenticated_subject;
mod committee;
mod credit_facility;
mod customer;
mod dashboard;
mod deposit;
mod document;
mod financials;
mod loader;
mod price;
mod primitives;
mod report;
mod sumsub;
mod terms;
mod terms_template;
mod withdrawal;
#[macro_use]
pub mod macros;
mod policy;
mod schema;
mod user;

use async_graphql::*;

use loader::LavaLoader;
pub use schema::*;

use lava_app::app::LavaApp;

pub fn schema(app: Option<LavaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let mut schema_builder = Schema::build(Query, Mutation, EmptySubscription);

    if let Some(app) = app {
        schema_builder = schema_builder.data(LavaLoader::new(&app)).data(app);
    }

    schema_builder.finish()
}
