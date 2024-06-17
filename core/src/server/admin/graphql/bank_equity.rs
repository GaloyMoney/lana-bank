use async_graphql::*;

use crate::app::LavaApp;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ShareholderEquityAddress {
    dummy: String,
}

impl Default for ShareholderEquityAddress {
    fn default() -> Self {
        Self {
            dummy: "value".to_string(),
        }
    }
}

#[ComplexObject]
impl ShareholderEquityAddress {
    async fn btc(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<String>> {
        let app = ctx.data_unchecked::<LavaApp>();
        Ok(app
            .ledger()
            .btc_equity_address_for_address_backed_account_by_id()
            .await?)
    }
}
