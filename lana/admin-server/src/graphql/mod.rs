mod accounting;
mod approval_process;
mod approval_rules;
mod audit;
mod authenticated_subject;
mod balance_sheet_config;
mod committee;
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
mod loan_agreement;
mod loader;
mod price;
mod primitives;
mod profit_and_loss_config;
mod report;
mod sumsub;
mod terms;
mod terms_template;
mod withdrawal;
#[macro_use]
pub mod macros;
mod access;
mod policy;
mod schema;

use async_graphql::*;

use loader::LanaLoader;
pub use schema::*;
pub use loan_agreement::*;

use lana_app::app::LanaApp;

pub fn schema(app: Option<LanaApp>) -> Schema<Query, Mutation, EmptySubscription> {
    let mut schema_builder = Schema::build(Query, Mutation, EmptySubscription);

    if let Some(app) = app {
        schema_builder = schema_builder.data(LanaLoader::new(&app)).data(app);
    }

    schema_builder.finish()
}
