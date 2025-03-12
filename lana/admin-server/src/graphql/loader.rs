use async_graphql::dataloader::{DataLoader, Loader};

use std::collections::HashMap;

use lana_app::{
    app::LanaApp, chart_of_accounts::error::CoreChartOfAccountsError,
    deposit::error::CoreDepositError, user::error::UserError,
};

use crate::primitives::*;

use super::{
    approval_process::*, chart_of_accounts::*, committee::*, credit_facility::*, customer::*,
    deposit::*, deposit_account::*, document::*, policy::*, terms_template::*, user::*,
    withdrawal::*,
};

pub type LanaDataLoader = DataLoader<LanaLoader>;
pub struct LanaLoader {
    pub app: LanaApp,
}

impl LanaLoader {
    pub fn new(app: &LanaApp) -> LanaDataLoader {
        DataLoader::new(Self { app: app.clone() }, tokio::task::spawn)
            // Set delay to 0 as per https://github.com/async-graphql/async-graphql/issues/1306
            .delay(std::time::Duration::from_secs(0))
    }
}

impl Loader<UserId> for LanaLoader {
    type Value = User;
    type Error = Arc<UserError>;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, User>, Self::Error> {
        self.app.users().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<governance::CommitteeId> for LanaLoader {
    type Value = Committee;
    type Error = Arc<governance::error::GovernanceError>;

    async fn load(
        &self,
        keys: &[CommitteeId],
    ) -> Result<HashMap<CommitteeId, Committee>, Self::Error> {
        self.app
            .governance()
            .find_all_committees(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<governance::PolicyId> for LanaLoader {
    type Value = Policy;
    type Error = Arc<governance::error::GovernanceError>;

    async fn load(&self, keys: &[PolicyId]) -> Result<HashMap<PolicyId, Policy>, Self::Error> {
        self.app
            .governance()
            .find_all_policies(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<governance::ApprovalProcessId> for LanaLoader {
    type Value = ApprovalProcess;
    type Error = Arc<governance::error::GovernanceError>;

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

impl Loader<DocumentId> for LanaLoader {
    type Value = Document;
    type Error = Arc<lana_app::document::error::DocumentError>;

    async fn load(
        &self,
        keys: &[DocumentId],
    ) -> Result<HashMap<DocumentId, Document>, Self::Error> {
        self.app.documents().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<CustomerId> for LanaLoader {
    type Value = Customer;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    async fn load(
        &self,
        keys: &[CustomerId],
    ) -> Result<HashMap<CustomerId, Customer>, Self::Error> {
        self.app.customers().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<ChartId> for LanaLoader {
    type Value = ChartOfAccounts;
    type Error = Arc<CoreChartOfAccountsError>;

    async fn load(
        &self,
        keys: &[ChartId],
    ) -> Result<HashMap<ChartId, ChartOfAccounts>, Self::Error> {
        self.app
            .chart_of_accounts()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<WithdrawalId> for LanaLoader {
    type Value = Withdrawal;
    type Error = Arc<CoreDepositError>;

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

impl Loader<TermsTemplateId> for LanaLoader {
    type Value = TermsTemplate;
    type Error = Arc<lana_app::terms_template::error::TermsTemplateError>;

    async fn load(
        &self,
        keys: &[TermsTemplateId],
    ) -> Result<HashMap<TermsTemplateId, TermsTemplate>, Self::Error> {
        self.app
            .terms_templates()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<CreditFacilityId> for LanaLoader {
    type Value = CreditFacility;
    type Error = Arc<lana_app::credit_facility::error::CoreCreditError>;

    async fn load(
        &self,
        keys: &[CreditFacilityId],
    ) -> Result<HashMap<CreditFacilityId, CreditFacility>, Self::Error> {
        self.app
            .credit_facilities()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<DisbursalId> for LanaLoader {
    type Value = CreditFacilityDisbursal;
    type Error = Arc<lana_app::credit_facility::error::CoreCreditError>;

    async fn load(
        &self,
        keys: &[DisbursalId],
    ) -> Result<HashMap<DisbursalId, CreditFacilityDisbursal>, Self::Error> {
        self.app
            .credit_facilities()
            .find_all_disbursals(keys)
            .await
            .map_err(Arc::new)
    }
}
