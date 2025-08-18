mod accounting;
mod approval_process;
mod approval_rules;
mod audit;
mod balance_sheet_config;
mod committee;
mod contract_creation;
mod credit_config;
mod credit_facility;
mod custody;
mod customer;
mod dashboard;
mod deposit;
mod deposit_account;
mod deposit_account_history;
mod deposit_config;
mod document;
mod loader;
mod me;
mod price;
mod primitives;
mod profit_and_loss_config;
mod public_id;
mod reports;
mod sumsub;
mod terms;
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

pub fn schema(app: Option<LanaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let mut schema_builder = Schema::build(Query, Mutation, EmptySubscription);

    if let Some(app) = app {
        schema_builder = schema_builder.data(LanaLoader::new(&app)).data(app);
    }

    schema_builder.finish()
}
