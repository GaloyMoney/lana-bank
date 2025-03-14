mod config;
mod error;

use sqlx::PgPool;
use tracing::instrument;

use authz::PermissionCheck;

use crate::{
    accounting_init::{ChartsInit, JournalInit, StatementsInit},
    applicant::Applicants,
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{init as init_authz, AppAction, AppObject, AuditAction, Authorization},
    balance_sheet::BalanceSheets,
    cash_flow::CashFlowStatements,
    chart_of_accounts::ChartOfAccounts,
    credit_facility::CreditFacilities,
    customer::Customers,
    customer_onboarding::CustomerOnboarding,
    dashboard::Dashboard,
    deposit::Deposits,
    deposit::Withdrawal,
    document::Documents,
    governance::Governance,
    job::Jobs,
    ledger_account::LedgerAccounts,
    outbox::Outbox,
    price::Price,
    primitives::{DepositAccountId, Subject, UsdCents, WithdrawalId},
    profit_and_loss::ProfitAndLossStatements,
    report::Reports,
    storage::Storage,
    terms_template::TermsTemplates,
    trial_balance::TrialBalances,
    user::Users,
    user_onboarding::UserOnboarding,
};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LanaApp {
    _pool: PgPool,
    _jobs: Jobs,
    audit: Audit,
    authz: Authorization,
    chart_of_accounts: ChartOfAccounts,
    customers: Customers,
    deposits: Deposits,
    applicants: Applicants,
    users: Users,
    credit_facilities: CreditFacilities,
    trial_balances: TrialBalances,
    profit_and_loss_statements: ProfitAndLossStatements,
    balance_sheets: BalanceSheets,
    cash_flow_statements: CashFlowStatements,
    ledger_accounts: LedgerAccounts,
    price: Price,
    report: Reports,
    terms_templates: TermsTemplates,
    documents: Documents,
    outbox: Outbox,
    governance: Governance,
    dashboard: Dashboard,
    _user_onboarding: UserOnboarding,
    _customer_onboarding: CustomerOnboarding,
}

impl LanaApp {
    pub async fn run(pool: PgPool, config: AppConfig) -> Result<Self, ApplicationError> {
        sqlx::migrate!().run(&pool).await?;

        let mut jobs = Jobs::new(&pool, config.job_execution);
        let audit = Audit::new(&pool);
        let authz = init_authz(&pool, &audit).await?;
        let outbox = Outbox::init(&pool).await?;
        let dashboard = Dashboard::init(&pool, &authz, &jobs, &outbox).await?;
        let governance = Governance::new(&pool, &authz, &outbox);
        let price = Price::new();
        let storage = Storage::init(&config.storage).await?;
        let documents = Documents::new(&pool, &storage, &authz);
        let report = Reports::init(&pool, &config.report, &authz, &jobs, &storage).await?;
        let users = Users::init(&pool, &authz, &outbox, config.user.superuser_email).await?;
        let user_onboarding =
            UserOnboarding::init(&jobs, &outbox, &users, config.user_onboarding).await?;

        let cala_config = cala_ledger::CalaLedgerConfig::builder()
            .pool(pool.clone())
            .exec_migrations(false)
            .build()
            .expect("cala config");
        let cala = cala_ledger::CalaLedger::init(cala_config).await?;
        let journal_init = JournalInit::journal(&cala).await?;
        let trial_balances =
            TrialBalances::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let pl_statements =
            ProfitAndLossStatements::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let balance_sheets =
            BalanceSheets::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        let cash_flow_statements =
            CashFlowStatements::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        StatementsInit::statements(
            &trial_balances,
            &pl_statements,
            &balance_sheets,
            &cash_flow_statements,
        )
        .await?;
        let chart_of_accounts =
            ChartOfAccounts::init(&pool, &authz, &cala, journal_init.journal_id).await?;
        ChartsInit::charts_of_accounts(&chart_of_accounts).await?;
        let ledger_accounts = LedgerAccounts::init(&authz, &cala, journal_init.journal_id);
        let customers = Customers::new(&pool, &authz, &outbox);
        let deposits = Deposits::init(
            &pool,
            &authz,
            &outbox,
            &governance,
            &customers,
            &jobs,
            &cala,
            journal_init.journal_id,
        )
        .await?;
        let customer_onboarding = CustomerOnboarding::init(
            &jobs,
            &outbox,
            &customers,
            &deposits,
            config.customer_onboarding,
        )
        .await?;
        let applicants = Applicants::new(&pool, &config.sumsub, &customers, &jobs);

        let credit_facilities = CreditFacilities::init(
            &pool,
            config.credit_facility,
            &governance,
            &jobs,
            &authz,
            &customers,
            &price,
            &outbox,
            &cala,
            journal_init.journal_id,
        )
        .await?;
        let terms_templates = TermsTemplates::new(&pool, &authz);
        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            _jobs: jobs,
            audit,
            authz,
            chart_of_accounts,
            customers,
            deposits,
            applicants,
            users,
            price,
            report,
            credit_facilities,
            trial_balances,
            profit_and_loss_statements: pl_statements,
            balance_sheets,
            cash_flow_statements,
            ledger_accounts,
            terms_templates,
            documents,
            outbox,
            governance,
            dashboard,
            _user_onboarding: user_onboarding,
            _customer_onboarding: customer_onboarding,
        })
    }

    pub fn dashboard(&self) -> &Dashboard {
        &self.dashboard
    }

    pub fn governance(&self) -> &Governance {
        &self.governance
    }

    pub fn customers(&self) -> &Customers {
        &self.customers
    }

    pub fn audit(&self) -> &Audit {
        &self.audit
    }

    pub fn reports(&self) -> &Reports {
        &self.report
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    pub fn outbox(&self) -> &Outbox {
        &self.outbox
    }

    #[instrument(name = "lana.audit.list_audit", skip(self), err)]
    pub async fn list_audit(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<AuditCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<AuditEntry, AuditCursor>, ApplicationError> {
        use crate::audit::AuditSvc;

        self.authz
            .enforce_permission(sub, AppObject::Audit, AppAction::Audit(AuditAction::List))
            .await?;

        self.audit.list(query).await.map_err(ApplicationError::from)
    }

    pub fn chart_of_accounts(&self) -> &ChartOfAccounts {
        &self.chart_of_accounts
    }

    pub fn deposits(&self) -> &Deposits {
        &self.deposits
    }

    pub fn applicants(&self) -> &Applicants {
        &self.applicants
    }

    pub fn credit_facilities(&self) -> &CreditFacilities {
        &self.credit_facilities
    }

    pub fn trial_balances(&self) -> &TrialBalances {
        &self.trial_balances
    }

    pub fn profit_and_loss_statements(&self) -> &ProfitAndLossStatements {
        &self.profit_and_loss_statements
    }

    pub fn balance_sheets(&self) -> &BalanceSheets {
        &self.balance_sheets
    }

    pub fn cash_flow_statements(&self) -> &CashFlowStatements {
        &self.cash_flow_statements
    }

    pub fn ledger_accounts(&self) -> &LedgerAccounts {
        &self.ledger_accounts
    }

    pub fn users(&self) -> &Users {
        &self.users
    }

    pub fn terms_templates(&self) -> &TermsTemplates {
        &self.terms_templates
    }

    pub fn documents(&self) -> &Documents {
        &self.documents
    }

    pub async fn get_visible_nav_items(
        &self,
        sub: &Subject,
    ) -> Result<
        crate::authorization::VisibleNavigationItems,
        crate::authorization::error::AuthorizationError,
    > {
        crate::authorization::get_visible_navigation_items(&self.authz, sub).await
    }

    #[instrument(name = "lana.confirm_withdrawal_with_sumsub", skip(self), err)]
    pub async fn confirm_withdrawal_with_sumsub(
        &self,
        sub: &Subject,
        withdrawal_id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Withdrawal, ApplicationError> {
        let withdrawal_id = withdrawal_id.into();

        // First, get the withdrawal to find the associated deposit account
        let withdrawal = match self
            .deposits()
            .find_withdrawal_by_id(sub, withdrawal_id)
            .await?
        {
            Some(withdrawal) => withdrawal,
            None => {
                tracing::error!("Withdrawal not found: {}", withdrawal_id);
                return Err(ApplicationError::DepositError(
                    crate::deposit::error::CoreDepositError::DepositAccountNotFound,
                ));
            }
        };

        // Use find_all_deposit_accounts to get the account holder ID
        let account_id = withdrawal.deposit_account_id;
        let accounts = self
            .deposits()
            .find_all_deposit_accounts::<crate::deposit::DepositAccount>(&[account_id])
            .await?;

        let deposit_account = match accounts.get(&account_id) {
            Some(account) => account,
            None => {
                tracing::error!("Deposit account not found: {}", account_id);
                return Err(ApplicationError::DepositError(
                    crate::deposit::error::CoreDepositError::DepositAccountNotFound,
                ));
            }
        };

        // Confirm the withdrawal using the CoreDeposit implementation
        // This is the critical step that should not be affected by Sumsub
        let withdrawal = self
            .deposits()
            .confirm_withdrawal(sub, withdrawal_id)
            .await?;

        // Get customer ID for Sumsub
        let customer_id = deposit_account.account_holder_id.into();

        // Clone the data we need for Sumsub submission
        let withdrawal_id_for_sumsub = withdrawal.id;
        let amount_for_sumsub = withdrawal.amount;
        let applicants = self.applicants().clone();

        // Submit transaction to Sumsub in a separate task to not block confirmation
        // This ensures the withdrawal confirmation completes regardless of Sumsub status
        tokio::spawn(async move {
            match applicants
                .submit_withdrawal_transaction(
                    withdrawal_id_for_sumsub,
                    customer_id,
                    amount_for_sumsub,
                )
                .await
            {
                Ok(_) => {
                    tracing::info!("Successfully submitted withdrawal to Sumsub");
                }
                Err(e) => {
                    // Just log the error, but don't affect the withdrawal confirmation
                    tracing::warn!("Failed to submit transaction to Sumsub: {:?}", e);
                }
            }
        });

        // Return the confirmed withdrawal immediately
        Ok(withdrawal)
    }

    #[instrument(name = "lana.confirm_deposit_with_sumsub", skip(self), err)]
    pub async fn confirm_deposit_with_sumsub(
        &self,
        sub: &Subject,
        deposit_account_id: impl Into<DepositAccountId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<crate::deposit::Deposit, ApplicationError> {
        let deposit_account_id = deposit_account_id.into();

        // First, record the deposit using the CoreDeposit implementation
        let deposit = self
            .deposits()
            .record_deposit(sub, deposit_account_id, amount, reference)
            .await?;

        // Get the account information using find_all_deposit_accounts
        let accounts = self
            .deposits()
            .find_all_deposit_accounts::<crate::deposit::DepositAccount>(&[deposit_account_id])
            .await?;

        let deposit_account = match accounts.get(&deposit_account_id) {
            Some(account) => account,
            None => {
                tracing::error!("Deposit account not found: {}", deposit_account_id);
                return Err(ApplicationError::DepositError(
                    crate::deposit::error::CoreDepositError::DepositAccountNotFound,
                ));
            }
        };

        // Get customer ID for Sumsub
        let customer_id = deposit_account.account_holder_id.into();

        // Clone the data we need for Sumsub submission
        let deposit_id_for_sumsub = deposit.id;
        let amount_for_sumsub = deposit.amount;
        let applicants = self.applicants().clone();

        // Submit transaction to Sumsub in a separate task to not block
        tokio::spawn(async move {
            match applicants
                .submit_deposit_transaction(deposit_id_for_sumsub, customer_id, amount_for_sumsub)
                .await
            {
                Ok(_) => {
                    tracing::info!("Successfully submitted deposit to Sumsub");
                }
                Err(e) => {
                    // Just log the error, but don't affect the deposit processing
                    tracing::warn!("Failed to submit transaction to Sumsub: {:?}", e);
                }
            }
        });

        // Return the deposit
        Ok(deposit)
    }
}
