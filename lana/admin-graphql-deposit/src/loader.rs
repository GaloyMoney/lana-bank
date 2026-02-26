use async_graphql::dataloader::{DataLoader, Loader};
use tracing::instrument;

use std::collections::HashMap;

use admin_graphql_governance::ApprovalProcess;
use lana_app::{
    app::LanaApp, deposit::error::CoreDepositError, governance::error::GovernanceError,
};

use crate::{Deposit, DepositAccount, Withdrawal, primitives::*};

pub type LanaDataLoader = DataLoader<LanaLoader>;
pub struct LanaLoader {
    pub app: LanaApp,
}

impl LanaLoader {
    pub fn new(app: &LanaApp) -> LanaDataLoader {
        DataLoader::new(
            Self { app: app.clone() },
            async_graphql::runtime::TokioSpawner::current(),
            async_graphql::runtime::TokioTimer::default(),
        )
        // Set delay to 0 as per https://github.com/async-graphql/async-graphql/issues/1306
        .delay(std::time::Duration::from_millis(5))
    }
}

impl Loader<ApprovalProcessId> for LanaLoader {
    type Value = ApprovalProcess;
    type Error = Arc<GovernanceError>;

    #[instrument(name = "loader.approval_processes", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[ApprovalProcessId],
    ) -> Result<HashMap<ApprovalProcessId, ApprovalProcess>, Self::Error> {
        self.app
            .governance()
            .find_all_approval_processes(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<WithdrawalId> for LanaLoader {
    type Value = Withdrawal;
    type Error = Arc<CoreDepositError>;

    #[instrument(name = "loader.withdrawals", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[WithdrawalId],
    ) -> Result<HashMap<WithdrawalId, Withdrawal>, Self::Error> {
        self.app
            .deposits()
            .find_all_withdrawals(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<DepositId> for LanaLoader {
    type Value = Deposit;
    type Error = Arc<CoreDepositError>;

    #[instrument(name = "loader.deposits", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[DepositId]) -> Result<HashMap<DepositId, Deposit>, Self::Error> {
        self.app
            .deposits()
            .find_all_deposits(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<DepositAccountId> for LanaLoader {
    type Value = DepositAccount;
    type Error = Arc<CoreDepositError>;

    #[instrument(name = "loader.deposit_accounts", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[DepositAccountId],
    ) -> Result<HashMap<DepositAccountId, DepositAccount>, Self::Error> {
        self.app
            .deposits()
            .find_all_deposit_accounts(keys)
            .await
            .map_err(Arc::new)
    }
}
