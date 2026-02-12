use async_graphql::*;

use crate::{graphql::loader::CHART_REF, primitives::*};

use super::{
    BtcLedgerAccountBalanceRange, LedgerAccount, LedgerAccountBalanceRangeByCurrency,
    UsdLedgerAccountBalanceRange,
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct TrialBalance {
    name: String,

    #[graphql(skip)]
    from: Date,
    #[graphql(skip)]
    until: Date,
    #[graphql(skip)]
    entity: Arc<lana_app::trial_balance::TrialBalanceRoot>,
}

#[ComplexObject]
impl TrialBalance {
    async fn total(&self) -> async_graphql::Result<LedgerAccountBalanceRangeByCurrency> {
        Ok(LedgerAccountBalanceRangeByCurrency {
            usd: self
                .entity
                .usd_balance_range
                .as_ref()
                .map(UsdLedgerAccountBalanceRange::from)
                .unwrap_or_default(),
            btc: self
                .entity
                .btc_balance_range
                .as_ref()
                .map(BtcLedgerAccountBalanceRange::from)
                .unwrap_or_default(),
        })
    }

    pub(crate) async fn accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<LedgerAccount>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let accounts = app
            .accounting()
            .list_all_account_flattened(
                sub,
                CHART_REF.0,
                self.from.into_inner(),
                Some(self.until.into_inner()),
            )
            .await?;
        Ok(accounts.into_iter().map(LedgerAccount::from).collect())
    }
}

impl From<lana_app::trial_balance::TrialBalanceRoot> for TrialBalance {
    fn from(trial_balance: lana_app::trial_balance::TrialBalanceRoot) -> Self {
        TrialBalance {
            name: trial_balance.name.to_string(),
            from: trial_balance.from.into(),
            until: trial_balance
                .until
                .expect("Mandatory 'until' value missing")
                .into(),
            entity: Arc::new(trial_balance),
        }
    }
}
