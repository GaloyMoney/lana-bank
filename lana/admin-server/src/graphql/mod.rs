mod accounting;
mod approval_process;
mod audit;
mod committee;
mod credit_facility;
mod custody;
mod customer;
mod deposit;
mod deposit_account;
mod domain_config;
mod loader;
mod primitives;
mod prospect;
mod public_id;
mod reports;
mod terms_template;
mod withdrawal;
#[macro_use]
pub(crate) mod macros;
mod access;
mod policy;
mod schema;

use async_graphql::*;

use loader::LanaLoader;
pub use schema::*;

use lana_app::app::LanaApp;

pub fn schema(app: Option<LanaApp>) -> Schema<Query, Mutation, Subscription> {
    let mut schema_builder = Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
    .extension(extensions::Tracing);

    if let Some(app) = app {
        schema_builder = schema_builder
            .data(LanaLoader::new(&app))
            .data(admin_graphql_customer::LanaLoader::new(&app))
            .data(admin_graphql_credit::LanaLoader::new(&app))
            .data(admin_graphql_deposit::LanaLoader::new(&app))
            .data(app);
    }

    schema_builder.finish()
}
