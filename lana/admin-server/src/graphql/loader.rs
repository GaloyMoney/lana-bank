use async_graphql::dataloader::{DataLoader, Loader};
use tracing::instrument;

use std::collections::HashMap;

use domain_config::{DomainConfigError, DomainConfigId};
use lana_app::{
    access::{error::CoreAccessError, user::error::UserError},
    accounting::{
        Chart, FiscalYearId, LedgerAccountId, TransactionTemplateId,
        chart_of_accounts::error::ChartOfAccountsError,
        csv::{AccountingCsvDocumentId, error::AccountingCsvExportError},
        fiscal_year::error::FiscalYearError,
        ledger_transaction::error::LedgerTransactionError,
        transaction_templates::error::TransactionTemplateError,
    },
    app::LanaApp,
    custody::error::CoreCustodyError,
    customer::CustomerDocumentId,
    deposit::error::CoreDepositError,
    governance::error::GovernanceError,
    report::{ReportId, ReportRunId, error::ReportError},
};

use crate::primitives::*;

use super::{
    access::*, accounting::*, approval_process::*, committee::*, credit_facility::*, custody::*,
    customer::*, deposit::*, deposit_account::*, document::*, domain_config::*, policy::*,
    reports::*, terms_template::*, withdrawal::*,
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
            .delay(std::time::Duration::from_millis(5))
    }
}
impl Loader<UserId> for LanaLoader {
    type Value = User;
    type Error = Arc<UserError>;

    #[instrument(name = "loader.users", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, User>, Self::Error> {
        self.app
            .access()
            .users()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}
impl Loader<PermissionSetId> for LanaLoader {
    type Value = PermissionSet;
    type Error = Arc<CoreAccessError>;

    #[instrument(name = "loader.permission_sets", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[PermissionSetId],
    ) -> Result<HashMap<PermissionSetId, PermissionSet>, Self::Error> {
        self.app
            .access()
            .find_all_permission_sets(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<RoleId> for LanaLoader {
    type Value = Role;
    type Error = Arc<CoreAccessError>;

    #[instrument(name = "loader.roles", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[RoleId]) -> Result<HashMap<RoleId, Role>, Self::Error> {
        self.app
            .access()
            .find_all_roles(keys)
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

impl Loader<CommitteeId> for LanaLoader {
    type Value = Committee;
    type Error = Arc<GovernanceError>;

    #[instrument(name = "loader.committees", skip(self), fields(count = keys.len()), err)]
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

impl Loader<PolicyId> for LanaLoader {
    type Value = Policy;
    type Error = Arc<GovernanceError>;

    #[instrument(name = "loader.policies", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[PolicyId]) -> Result<HashMap<PolicyId, Policy>, Self::Error> {
        self.app
            .governance()
            .find_all_policies(keys)
            .await
            .map_err(Arc::new)
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

impl Loader<CustomerDocumentId> for LanaLoader {
    type Value = CustomerDocument;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.customer_documents", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CustomerDocumentId],
    ) -> Result<HashMap<CustomerDocumentId, CustomerDocument>, Self::Error> {
        self.app
            .customers()
            .find_all_documents(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<CustomerId> for LanaLoader {
    type Value = Customer;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.customers", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CustomerId],
    ) -> Result<HashMap<CustomerId, Customer>, Self::Error> {
        self.app.customers().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<DomainConfigId> for LanaLoader {
    type Value = DomainConfig;
    type Error = Arc<DomainConfigError>;

    #[instrument(name = "loader.domain_configs", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[DomainConfigId],
    ) -> Result<HashMap<DomainConfigId, DomainConfig>, Self::Error> {
        self.app
            .exposed_domain_configs()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<ChartRef> for LanaLoader {
    type Value = Arc<Chart>;
    type Error = Arc<ChartOfAccountsError>;

    #[instrument(name = "loader.chart_refs", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[ChartRef]) -> Result<HashMap<ChartRef, Arc<Chart>>, Self::Error> {
        let mut res = HashMap::new();
        for key in keys {
            if let Some(chart) = self
                .app
                .accounting()
                .chart_of_accounts()
                .maybe_find_by_reference(key.0)
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

    #[instrument(name = "loader.charts", skip(self), fields(count = keys.len()), err)]
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

impl Loader<LedgerTransactionId> for LanaLoader {
    type Value = LedgerTransaction;
    type Error = Arc<LedgerTransactionError>;

    #[instrument(name = "loader.ledger_transactions", skip(self), fields(count = keys.len()), err)]
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

    #[instrument(name = "loader.transaction_templates", skip(self), fields(count = keys.len()), err)]
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
    type Error = Arc<lana_app::terms_template::terms_template_error::TermsTemplateError>;

    #[instrument(name = "loader.terms_templates", skip(self), fields(count = keys.len()), err)]
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

impl Loader<PendingCreditFacilityId> for LanaLoader {
    type Value = PendingCreditFacility;
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

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
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

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
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

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
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

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

impl Loader<WalletId> for LanaLoader {
    type Value = Wallet;
    type Error = Arc<lana_app::custody::error::CoreCustodyError>;

    #[instrument(name = "loader.wallets", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[WalletId]) -> Result<HashMap<WalletId, Wallet>, Self::Error> {
        self.app
            .custody()
            .find_all_wallets(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<DisbursalId> for LanaLoader {
    type Value = CreditFacilityDisbursal;
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

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
    type Error = Arc<lana_app::credit::error::CoreCreditError>;

    #[instrument(name = "loader.liquidations", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[LiquidationId],
    ) -> Result<HashMap<LiquidationId, Liquidation>, Self::Error> {
        self.app
            .credit()
            .liquidations()
            .find_all(keys)
            .await
            .map_err(|e| Arc::new(e.into()))
    }
}

impl Loader<LedgerAccountId> for LanaLoader {
    type Value = LedgerAccount;
    type Error = Arc<lana_app::accounting::error::CoreAccountingError>;

    #[instrument(name = "loader.ledger_accounts", skip(self), fields(count = keys.len()), err)]
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

impl Loader<AccountingCsvDocumentId> for LanaLoader {
    type Value = AccountingCsvDocument;
    type Error = Arc<AccountingCsvExportError>;

    #[instrument(name = "loader.accounting_csv_documents", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[AccountingCsvDocumentId],
    ) -> Result<HashMap<AccountingCsvDocumentId, AccountingCsvDocument>, Self::Error> {
        self.app
            .accounting()
            .csvs()
            .find_all_documents::<AccountingCsvDocument>(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<ReportId> for LanaLoader {
    type Value = Report;
    type Error = Arc<ReportError>;

    #[instrument(name = "loader.reports", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[ReportId]) -> Result<HashMap<ReportId, Report>, Self::Error> {
        let reports = self
            .app
            .reports()
            .find_all_reports(keys)
            .await
            .map_err(Arc::new)?;
        Ok(reports.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl Loader<ReportRunId> for LanaLoader {
    type Value = ReportRun;
    type Error = Arc<ReportError>;

    #[instrument(name = "loader.report_runs", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[ReportRunId],
    ) -> Result<HashMap<ReportRunId, ReportRun>, Self::Error> {
        let report_runs = self
            .app
            .reports()
            .find_all_report_runs(keys)
            .await
            .map_err(Arc::new)?;
        Ok(report_runs
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect())
    }
}

impl Loader<FiscalYearId> for LanaLoader {
    type Value = FiscalYear;
    type Error = Arc<FiscalYearError>;

    async fn load(
        &self,
        keys: &[FiscalYearId],
    ) -> Result<HashMap<FiscalYearId, FiscalYear>, Self::Error> {
        self.app
            .accounting()
            .fiscal_year()
            .find_all(keys)
            .await
            .map_err(Arc::new)
    }
}
