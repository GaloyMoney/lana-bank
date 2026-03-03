use async_graphql::dataloader::{DataLoader, Loader};
use tracing::instrument;

use std::collections::HashMap;

use domain_config::{DomainConfigError, DomainConfigId};
use lana_app::{
    access::{error::CoreAccessError, user::error::UserError},
    accounting::{
        Chart, FiscalYearId, LedgerAccountId, TransactionTemplateId, csv::AccountingCsvDocumentId,
        error::CoreAccountingError,
    },
    app::LanaApp,
    custody::error::CoreCustodyError,
    customer::{CustomerDocumentId, Party, PartyId},
    deposit::error::CoreDepositError,
    governance::error::GovernanceError,
    report::{ReportId, ReportRunId, error::ReportError},
};

use crate::primitives::*;

use super::{
    access::*, accounting::*, approval_process::*, committee::*, credit_facility::*, custody::*,
    customer::*, deposit::*, deposit_account::*, document::*, domain_config::*, policy::*,
    prospect::*, reports::*, terms_template::*, withdrawal::*,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChartRef(pub &'static str);
pub const CHART_REF: ChartRef = ChartRef(lana_app::accounting_init::constants::CHART_REF);

pub type LanaDataLoader = DataLoader<LanaLoader>;
pub struct LanaLoader {
    pub app: LanaApp,
    sub: Subject,
}

impl LanaLoader {
    pub fn new(app: &LanaApp, sub: &Subject) -> LanaDataLoader {
        DataLoader::new(
            Self {
                app: app.clone(),
                sub: sub.clone(),
            },
            async_graphql::runtime::TokioSpawner::current(),
            async_graphql::runtime::TokioTimer::default(),
        )
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
            .find_all_authorized(&self.sub, keys)
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
            .find_all_permission_sets_authorized(&self.sub, keys)
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
            .find_all_roles_authorized(&self.sub, keys)
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
            .find_all_custodians_authorized(&self.sub, keys)
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
            .find_all_committees_authorized(&self.sub, keys)
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
            .find_all_policies_authorized(&self.sub, keys)
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
            .find_all_approval_processes_authorized(&self.sub, keys)
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
            .find_all_documents_authorized(&self.sub, keys)
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
        self.app
            .customers()
            .find_all_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<ProspectId> for LanaLoader {
    type Value = Prospect;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.prospects", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[ProspectId],
    ) -> Result<HashMap<ProspectId, Prospect>, Self::Error> {
        self.app
            .customers()
            .find_all_prospects_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<PartyId> for LanaLoader {
    type Value = Arc<Party>;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.parties", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[PartyId]) -> Result<HashMap<PartyId, Arc<Party>>, Self::Error> {
        self.app
            .customers()
            .find_all_parties_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<ChartRef> for LanaLoader {
    type Value = Arc<Chart>;
    type Error = Arc<CoreAccountingError>;

    #[instrument(name = "loader.chart_refs", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[ChartRef]) -> Result<HashMap<ChartRef, Arc<Chart>>, Self::Error> {
        let refs: Vec<&str> = keys.iter().map(|k| k.0).collect();
        let mut charts = self
            .app
            .accounting()
            .find_all_charts_by_reference_authorized(&self.sub, &refs)
            .await
            .map_err(Arc::new)?;
        let mut res = HashMap::new();
        for key in keys {
            if let Some(chart) = charts.remove(key.0) {
                res.insert(key.clone(), Arc::new(chart));
            }
        }
        Ok(res)
    }
}

impl Loader<ChartId> for LanaLoader {
    type Value = ChartOfAccounts;
    type Error = Arc<CoreAccountingError>;

    #[instrument(name = "loader.charts", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[ChartId],
    ) -> Result<HashMap<ChartId, ChartOfAccounts>, Self::Error> {
        self.app
            .accounting()
            .find_all_charts_authorized(&self.sub, keys)
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
            .find_all_withdrawals_authorized(&self.sub, keys)
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
            .find_all_deposits_authorized(&self.sub, keys)
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
            .find_all_deposit_accounts_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<LedgerTransactionId> for LanaLoader {
    type Value = LedgerTransaction;
    type Error = Arc<CoreAccountingError>;

    #[instrument(name = "loader.ledger_transactions", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[LedgerTransactionId],
    ) -> Result<HashMap<LedgerTransactionId, Self::Value>, Self::Error> {
        self.app
            .accounting()
            .find_all_ledger_transactions_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<TransactionTemplateId> for LanaLoader {
    type Value = TransactionTemplate;
    type Error = Arc<CoreAccountingError>;

    #[instrument(name = "loader.transaction_templates", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[TransactionTemplateId],
    ) -> Result<HashMap<TransactionTemplateId, Self::Value>, Self::Error> {
        self.app
            .accounting()
            .find_all_transaction_templates_authorized(&self.sub, keys)
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
            .find_all_authorized(&self.sub, keys)
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
            .find_all_pending_credit_facilities_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_facilities_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_proposals_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_collaterals_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_disbursals_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_liquidations_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
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
            .find_all_ledger_accounts_authorized(&self.sub, CHART_REF.0, keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<AccountingCsvDocumentId> for LanaLoader {
    type Value = AccountingCsvDocument;
    type Error = Arc<CoreAccountingError>;

    #[instrument(name = "loader.accounting_csv_documents", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[AccountingCsvDocumentId],
    ) -> Result<HashMap<AccountingCsvDocumentId, AccountingCsvDocument>, Self::Error> {
        self.app
            .accounting()
            .find_all_csv_documents_authorized(&self.sub, keys)
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
            .find_all_reports_authorized(&self.sub, keys)
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
            .find_all_report_runs_authorized(&self.sub, keys)
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
    type Error = Arc<CoreAccountingError>;

    async fn load(
        &self,
        keys: &[FiscalYearId],
    ) -> Result<HashMap<FiscalYearId, FiscalYear>, Self::Error> {
        self.app
            .accounting()
            .find_all_fiscal_years_authorized(&self.sub, keys)
            .await
            .map_err(Arc::new)
    }
}
