use async_graphql::dataloader::{DataLoader, Loader};
use tracing::instrument;

use std::collections::HashMap;

use admin_graphql_custody::{Custodian, Wallet};
use admin_graphql_governance::ApprovalProcess;
use admin_graphql_shared::primitives::*;
use lana_app::{
    app::LanaApp, credit::error::CoreCreditError, custody::error::CoreCustodyError,
    governance::error::GovernanceError,
};

use crate::{
    Collateral, CreditFacility, CreditFacilityDisbursal, CreditFacilityProposal, Liquidation,
    PendingCreditFacility,
};

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

impl Loader<CustodianId> for LanaLoader {
    type Value = Custodian;
    type Error = Arc<CoreCustodyError>;

    #[instrument(name = "loader.custodians", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CustodianId],
    ) -> Result<HashMap<CustodianId, Custodian>, Self::Error> {
        self.app
            .custody()
            .find_all_custodians(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<WalletId> for LanaLoader {
    type Value = Wallet;
    type Error = Arc<CoreCustodyError>;

    #[instrument(name = "loader.wallets", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[WalletId]) -> Result<HashMap<WalletId, Wallet>, Self::Error> {
        self.app
            .custody()
            .find_all_wallets(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<PendingCreditFacilityId> for LanaLoader {
    type Value = PendingCreditFacility;
    type Error = Arc<CoreCreditError>;

    #[instrument(name = "loader.pending_credit_facilities", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[PendingCreditFacilityId],
    ) -> Result<HashMap<PendingCreditFacilityId, PendingCreditFacility>, Self::Error> {
        self.app
            .credit()
            .pending_credit_facilities()
            .find_all(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}

impl Loader<CreditFacilityId> for LanaLoader {
    type Value = CreditFacility;
    type Error = Arc<CoreCreditError>;

    #[instrument(name = "loader.credit_facilities", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CreditFacilityId],
    ) -> Result<HashMap<CreditFacilityId, CreditFacility>, Self::Error> {
        self.app
            .credit()
            .facilities()
            .find_all(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}

impl Loader<CreditFacilityProposalId> for LanaLoader {
    type Value = CreditFacilityProposal;
    type Error = Arc<CoreCreditError>;

    #[instrument(name = "loader.credit_facility_proposals", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CreditFacilityProposalId],
    ) -> Result<HashMap<CreditFacilityProposalId, CreditFacilityProposal>, Self::Error> {
        self.app
            .credit()
            .proposals()
            .find_all(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}

impl Loader<CollateralId> for LanaLoader {
    type Value = Collateral;
    type Error = Arc<CoreCreditError>;

    #[instrument(name = "loader.collaterals", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CollateralId],
    ) -> Result<HashMap<CollateralId, Collateral>, Self::Error> {
        self.app
            .credit()
            .collaterals()
            .find_all(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}

impl Loader<DisbursalId> for LanaLoader {
    type Value = CreditFacilityDisbursal;
    type Error = Arc<CoreCreditError>;

    #[instrument(name = "loader.disbursals", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[DisbursalId],
    ) -> Result<HashMap<DisbursalId, CreditFacilityDisbursal>, Self::Error> {
        self.app
            .credit()
            .disbursals()
            .find_all(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}

impl Loader<LiquidationId> for LanaLoader {
    type Value = Liquidation;
    type Error = Arc<CoreCreditError>;

    #[instrument(name = "loader.liquidations", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[LiquidationId],
    ) -> Result<HashMap<LiquidationId, Liquidation>, Self::Error> {
        self.app
            .credit()
            .collaterals()
            .find_all_liquidations(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}
