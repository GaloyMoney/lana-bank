use async_graphql::dataloader::{DataLoader, Loader};

use std::collections::HashMap;

use lana_app::{
    accounting::{
        chart_of_accounts::error::ChartOfAccountsError,
        ledger_transaction::error::LedgerTransactionError,
        transaction_templates::error::TransactionTemplateError, Chart, LedgerAccountId,
        TransactionTemplateId,
    },
    app::LanaApp,
    deposit::error::CoreDepositError,
    user::error::UserError,
};

use crate::primitives::*;

use super::{
    accounting::*, approval_process::*, chart_of_accounts::*, committee::*, credit_facility::*,
    customer::*, deposit::*, deposit_account::*, document::*, policy::*, terms_template::*,
    user::*, withdrawal::*,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChartRef(pub &'static str);
pub const CHART_REF: ChartRef = ChartRef(lana_app::accounting_init::constants::CHART_REF);

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

impl Loader<ChartRef> for LanaLoader {
    type Value = Arc<Chart>;
    type Error = Arc<ChartOfAccountsError>;

    async fn load(&self, keys: &[ChartRef]) -> Result<HashMap<ChartRef, Arc<Chart>>, Self::Error> {
        let mut res = HashMap::new();
        for key in keys {
            if let Some(chart) = self
                .app
                .accounting()
                .chart_of_accounts()
                .find_by_reference(key.0)
                .await
                .map_err(Arc::new)?
            {
                res.insert(key.clone(), Arc::new(chart));
            }
        }
        Ok(res)
    }
}

impl Loader<ChartId> for LanaLoader {
    type Value = ChartOfAccounts;
    type Error = Arc<ChartOfAccountsError>;

    async fn load(
        &self,
        keys: &[ChartId],
    ) -> Result<HashMap<ChartId, ChartOfAccounts>, Self::Error> {
        self.app
            .accounting()
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

impl Loader<LedgerTransactionId> for LanaLoader {
    type Value = LedgerTransaction;
    type Error = Arc<LedgerTransactionError>;

    async fn load(
        &self,
        keys: &[LedgerTransactionId],
    ) -> Result<HashMap<LedgerTransactionId, Self::Value>, Self::Error> {
        self.app
            .accounting()
            .ledger_transactions()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<TransactionTemplateId> for LanaLoader {
    type Value = TransactionTemplate;
    type Error = Arc<TransactionTemplateError>;

    async fn load(
        &self,
        keys: &[TransactionTemplateId],
    ) -> Result<HashMap<TransactionTemplateId, Self::Value>, Self::Error> {
        self.app
            .accounting()
            .transaction_templates()
            .find_all(keys)
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
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

    async fn load(
        &self,
        keys: &[CreditFacilityId],
    ) -> Result<HashMap<CreditFacilityId, CreditFacility>, Self::Error> {
        self.app.credit().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<DisbursalId> for LanaLoader {
    type Value = CreditFacilityDisbursal;
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

    async fn load(
        &self,
        keys: &[DisbursalId],
    ) -> Result<HashMap<DisbursalId, CreditFacilityDisbursal>, Self::Error> {
        self.app
            .credit()
            .find_all_disbursals(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<LedgerAccountId> for LanaLoader {
    type Value = LedgerAccount;
    type Error = Arc<lana_app::accounting::error::CoreAccountingError>;

    async fn load(
        &self,
        keys: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, LedgerAccount>, Self::Error> {
        self.app
            .accounting()
            .find_all_ledger_accounts(CHART_REF.0, keys)
            .await
            .map_err(Arc::new)
    }
}
