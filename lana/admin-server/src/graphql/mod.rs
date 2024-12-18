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
mod deposit_account;
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

use loader::LanaLoader;
pub use schema::*;

use lana_app::app::LanaApp;

pub fn schema(app: Option<LanaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let mut schema_builder = Schema::build(Query, Mutation, EmptySubscription);

    if let Some(app) = app {
        schema_builder = schema_builder.data(LanaLoader::new(&app)).data(app);
    }

    schema_builder.finish()
}
