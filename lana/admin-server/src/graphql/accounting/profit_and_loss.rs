use async_graphql::*;

use lana_app::{
    primitives::Currency as CalaCurrency,
    profit_and_loss::ProfitAndLossStatement as DomainProfitAndLossStatement,
};

use crate::primitives::*;

use super::LedgerAccountBalanceRange;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ProfitAndLossStatement {
    pub name: String,

    #[graphql(skip)]
    pub entity: Arc<DomainProfitAndLossStatement>,
}

impl From<DomainProfitAndLossStatement> for ProfitAndLossStatement {
    fn from(profit_and_loss: DomainProfitAndLossStatement) -> Self {
        ProfitAndLossStatement {
            name: profit_and_loss.name.to_string(),
            entity: Arc::new(profit_and_loss),
        }
    }
}

#[ComplexObject]
impl ProfitAndLossStatement {
    async fn net(&self) -> async_graphql::Result<LedgerAccountBalanceRange> {
        if let Some(balance) = self.entity.btc_balance_range.as_ref() {
            Ok(Some(balance).into())
        } else {
            Ok(self.entity.usd_balance_range.as_ref().into())
        }
    }
}
