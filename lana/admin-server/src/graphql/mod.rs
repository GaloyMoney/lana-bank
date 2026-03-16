mod accounting;
mod approval_process;
mod approval_rules;
mod audit;
mod build_info;
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
mod domain_config;
mod event_timeline;
mod loader;
mod me;
mod price;
mod price_provider;
mod primitives;
mod prospect;
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

pub use build_info::BuildInfo;
pub(crate) use loader::LanaLoader;
pub use schema::*;

use lana_app::app::LanaApp;

#[derive(Clone, Default)]
pub struct AppConfig(pub String);

pub fn schema(
    app: Option<LanaApp>,
    build_info: BuildInfo,
    app_config: AppConfig,
) -> Schema<Query, Mutation, Subscription> {
    let mut schema_builder = Schema::build(Query, Mutation, Subscription)
        .extension(extensions::Tracing)
        .data(build_info)
        .data(app_config);

    if let Some(app) = app {
        schema_builder = schema_builder.data(app);
    }

    schema_builder.finish()
}
