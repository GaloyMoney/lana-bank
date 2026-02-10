use async_graphql::*;

#[derive(SimpleObject)]
pub struct AccountSetDag {
    pub d2: String,
}

impl From<lana_app::accounting::chart_of_accounts::dag::AccountDag> for AccountSetDag {
    fn from(dag: lana_app::accounting::chart_of_accounts::dag::AccountDag) -> Self {
        Self { d2: dag.to_d2() }
    }
}
