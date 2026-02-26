use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lanacli", about = "LANA Bank Admin CLI")]
pub struct Cli {
    /// Admin GraphQL endpoint URL
    #[arg(
        long,
        env = "LANA_ADMIN_URL",
        default_value = "http://admin.localhost:4455/graphql"
    )]
    pub admin_url: String,

    /// Keycloak URL
    #[arg(
        long,
        env = "LANA_KEYCLOAK_URL",
        default_value = "http://localhost:8081"
    )]
    pub keycloak_url: String,

    /// Admin username (Keycloak email)
    #[arg(long, env = "LANA_USERNAME", default_value = "admin@galoy.io")]
    pub username: String,

    /// Admin password (empty for local dev)
    #[arg(long, env = "LANA_PASSWORD", default_value = "")]
    pub password: String,

    /// Output as JSON instead of tables
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage prospects
    Prospect {
        #[command(subcommand)]
        action: ProspectAction,
    },
    /// Manage customers
    Customer {
        #[command(subcommand)]
        action: CustomerAction,
    },
    /// Manage deposit accounts
    DepositAccount {
        #[command(subcommand)]
        action: DepositAccountAction,
    },
    /// Manage terms templates
    TermsTemplate {
        #[command(subcommand)]
        action: TermsTemplateAction,
    },
    /// Manage credit facilities
    CreditFacility {
        #[command(subcommand)]
        action: CreditFacilityAction,
    },
    /// Manage approval processes
    ApprovalProcess {
        #[command(subcommand)]
        action: ApprovalProcessAction,
    },
    /// Manage collateral
    Collateral {
        #[command(subcommand)]
        action: CollateralAction,
    },
    /// Manage liquidations
    Liquidation {
        #[command(subcommand)]
        action: LiquidationAction,
    },
    /// View dashboard
    Dashboard {
        #[command(subcommand)]
        action: DashboardAction,
    },
    /// Manage accounting
    Accounting {
        #[command(subcommand)]
        action: AccountingAction,
    },
    /// Manage fiscal years
    FiscalYear {
        #[command(subcommand)]
        action: FiscalYearAction,
    },
    /// Manage CSV exports
    CsvExport {
        #[command(subcommand)]
        action: CsvExportAction,
    },
    /// Manage custodians
    Custodian {
        #[command(subcommand)]
        action: CustodianAction,
    },
    /// Manage customer documents
    Document {
        #[command(subcommand)]
        action: DocumentAction,
    },
    /// Manage domain configs
    DomainConfig {
        #[command(subcommand)]
        action: DomainConfigAction,
    },
    /// View audit logs
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
    /// View financial statements
    FinancialStatement {
        #[command(subcommand)]
        action: FinancialStatementAction,
    },
    /// Manage Sumsub integration
    Sumsub {
        #[command(subcommand)]
        action: SumsubAction,
    },
    /// Manage loan agreements
    LoanAgreement {
        #[command(subcommand)]
        action: LoanAgreementAction,
    },
    /// Manage users and roles
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    /// Manage reports
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },
    /// Find a withdrawal by ID
    Withdrawal {
        #[command(subcommand)]
        action: WithdrawalAction,
    },
    /// Authenticate and cache session token
    Login,
    /// Clear cached session token
    Logout,
}

#[derive(Subcommand)]
pub enum ProspectAction {
    /// Create a new prospect
    Create {
        #[arg(long)]
        email: String,
        #[arg(long)]
        telegram_handle: String,
        #[arg(long, default_value = "INDIVIDUAL")]
        customer_type: String,
    },
    /// List prospects
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get a prospect by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// Convert a prospect to a customer
    Convert {
        #[arg(long)]
        prospect_id: String,
    },
    /// Close a prospect
    Close {
        #[arg(long)]
        prospect_id: String,
    },
}

#[derive(Subcommand)]
pub enum CustomerAction {
    /// List customers
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get a customer by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// Get a customer by email
    GetByEmail {
        #[arg(long)]
        email: String,
    },
}

#[derive(Subcommand)]
pub enum DepositAccountAction {
    /// Create a deposit account
    Create {
        #[arg(long)]
        customer_id: String,
    },
    /// List deposit accounts
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get a deposit account by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// Record a deposit
    RecordDeposit {
        #[arg(long)]
        deposit_account_id: String,
        #[arg(long)]
        amount: String,
    },
    /// Initiate a withdrawal
    InitiateWithdrawal {
        #[arg(long)]
        deposit_account_id: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        reference: String,
    },
    /// Confirm a withdrawal
    ConfirmWithdrawal {
        #[arg(long)]
        withdrawal_id: String,
    },
    /// Cancel a withdrawal
    CancelWithdrawal {
        #[arg(long)]
        withdrawal_id: String,
    },
    /// Revert a withdrawal
    RevertWithdrawal {
        #[arg(long)]
        withdrawal_id: String,
    },
    /// Freeze a deposit account
    Freeze {
        #[arg(long)]
        deposit_account_id: String,
    },
    /// Unfreeze a deposit account
    Unfreeze {
        #[arg(long)]
        deposit_account_id: String,
    },
    /// Close a deposit account
    Close {
        #[arg(long)]
        deposit_account_id: String,
    },
}

#[derive(Subcommand)]
pub enum TermsTemplateAction {
    /// Create a terms template
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        annual_rate: String,
        #[arg(long, default_value = "END_OF_MONTH")]
        accrual_interval: String,
        #[arg(long, default_value = "END_OF_MONTH")]
        accrual_cycle_interval: String,
        #[arg(long, default_value = "0")]
        one_time_fee_rate: String,
        #[arg(long, default_value = "SINGLE_DISBURSAL")]
        disbursal_policy: String,
        #[arg(long)]
        duration_months: i64,
        #[arg(long)]
        initial_cvl: String,
        #[arg(long)]
        margin_call_cvl: String,
        #[arg(long)]
        liquidation_cvl: String,
        #[arg(long, default_value = "30")]
        interest_due_days: i64,
        #[arg(long, default_value = "30")]
        overdue_days: i64,
        #[arg(long, default_value = "90")]
        liquidation_days: i64,
    },
    /// List all terms templates
    List,
    /// Get a terms template by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// Update a terms template
    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        annual_rate: String,
        #[arg(long, default_value = "END_OF_MONTH")]
        accrual_interval: String,
        #[arg(long, default_value = "END_OF_MONTH")]
        accrual_cycle_interval: String,
        #[arg(long, default_value = "0")]
        one_time_fee_rate: String,
        #[arg(long, default_value = "SINGLE_DISBURSAL")]
        disbursal_policy: String,
        #[arg(long)]
        duration_months: i64,
        #[arg(long)]
        initial_cvl: String,
        #[arg(long)]
        margin_call_cvl: String,
        #[arg(long)]
        liquidation_cvl: String,
        #[arg(long, default_value = "30")]
        interest_due_days: i64,
        #[arg(long, default_value = "30")]
        overdue_days: i64,
        #[arg(long, default_value = "90")]
        liquidation_days: i64,
    },
}

#[derive(Subcommand)]
pub enum CreditFacilityAction {
    /// Create a credit facility proposal
    ProposalCreate {
        #[arg(long)]
        customer_id: String,
        #[arg(long)]
        facility_amount: String,
        #[arg(long)]
        custodian_id: Option<String>,
        #[arg(long)]
        annual_rate: String,
        #[arg(long, default_value = "END_OF_MONTH")]
        accrual_interval: String,
        #[arg(long, default_value = "END_OF_MONTH")]
        accrual_cycle_interval: String,
        #[arg(long, default_value = "0")]
        one_time_fee_rate: String,
        #[arg(long, default_value = "SINGLE_DISBURSAL")]
        disbursal_policy: String,
        #[arg(long)]
        duration_months: i64,
        #[arg(long)]
        initial_cvl: String,
        #[arg(long)]
        margin_call_cvl: String,
        #[arg(long)]
        liquidation_cvl: String,
        #[arg(long, default_value = "30")]
        interest_due_days: i64,
        #[arg(long, default_value = "30")]
        overdue_days: i64,
        #[arg(long, default_value = "90")]
        liquidation_days: i64,
    },
    /// Get a credit facility proposal by ID
    ProposalGet {
        #[arg(long)]
        id: String,
    },
    /// Conclude customer approval for a proposal
    ProposalConclude {
        #[arg(long)]
        id: String,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        approved: bool,
    },
    /// Get a pending credit facility by ID
    PendingGet {
        #[arg(long)]
        id: String,
    },
    /// List credit facility proposals
    ProposalList {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// List active credit facilities
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get a credit facility by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// Find a credit facility by ID (detailed view)
    Find {
        #[arg(long)]
        id: String,
    },
    /// Initiate a disbursal
    DisbursalInitiate {
        #[arg(long)]
        credit_facility_id: String,
        #[arg(long)]
        amount: String,
    },
    /// Record a partial payment
    PartialPaymentRecord {
        #[arg(long)]
        credit_facility_id: String,
        #[arg(long)]
        amount: String,
    },
}

#[derive(Subcommand)]
pub enum ApprovalProcessAction {
    /// Approve a process
    Approve {
        #[arg(long)]
        process_id: String,
    },
    /// Deny a process
    Deny {
        #[arg(long)]
        process_id: String,
        #[arg(long)]
        reason: String,
    },
    /// List approval processes
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get an approval process by ID
    Get {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum CollateralAction {
    /// Update collateral
    Update {
        #[arg(long)]
        collateral_id: String,
        #[arg(long)]
        collateral: String,
        #[arg(long)]
        effective: String,
    },
}

#[derive(Subcommand)]
pub enum LiquidationAction {
    /// Find a liquidation by ID
    Find {
        #[arg(long)]
        id: String,
    },
    /// Record collateral sent to liquidation
    RecordCollateralSent {
        #[arg(long)]
        collateral_id: String,
        #[arg(long)]
        amount: String,
    },
    /// Record payment received from liquidation
    RecordPaymentReceived {
        #[arg(long)]
        collateral_id: String,
        #[arg(long)]
        amount: String,
    },
}

#[derive(Subcommand)]
pub enum DashboardAction {
    /// Get dashboard summary
    Get,
}

#[derive(Subcommand)]
pub enum AccountingAction {
    /// Get chart of accounts
    ChartOfAccounts,
    /// Add a root node to the chart of accounts
    AddRootNode {
        #[arg(long)]
        code: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        normal_balance_type: String,
    },
    /// Add a child node to the chart of accounts
    AddChildNode {
        #[arg(long)]
        parent: String,
        #[arg(long)]
        code: String,
        #[arg(long)]
        name: String,
    },
    /// Import chart of accounts from CSV file
    CsvImport {
        #[arg(long)]
        file: String,
    },
    /// Get accounting base config
    BaseConfig,
    /// Get credit config
    CreditConfig,
    /// Get deposit config
    DepositConfig,
    /// Get account sets by category
    AccountSets {
        #[arg(long)]
        category: String,
    },
    /// Execute a manual transaction
    ManualTransaction {
        #[arg(long)]
        description: String,
        #[arg(long)]
        reference: Option<String>,
        #[arg(long)]
        effective: Option<String>,
        /// JSON array of entries: [{"accountRef":"...","amount":"...","currency":"...","direction":"...","description":"..."}]
        #[arg(long)]
        entries_json: String,
    },
    /// Get ledger account by code
    LedgerAccount {
        #[arg(long)]
        code: String,
    },
}

#[derive(Subcommand)]
pub enum FiscalYearAction {
    /// List fiscal years
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Close a month in a fiscal year
    CloseMonth {
        #[arg(long)]
        fiscal_year_id: String,
    },
    /// Close a fiscal year
    Close {
        #[arg(long)]
        fiscal_year_id: String,
    },
}

#[derive(Subcommand)]
pub enum CsvExportAction {
    /// Get account entry CSV status
    AccountEntry {
        #[arg(long)]
        ledger_account_id: String,
    },
    /// Create a ledger account CSV export
    CreateLedgerCsv {
        #[arg(long)]
        ledger_account_id: String,
    },
    /// Generate download link for CSV
    DownloadLink {
        #[arg(long)]
        document_id: String,
    },
}

#[derive(Subcommand)]
pub enum CustodianAction {
    /// Create a custodian (pass full input as JSON)
    Create {
        /// JSON input, e.g. {"komainu":{"name":"...","apiKey":"...","apiSecret":"...","testingInstance":true,"secretKey":"..."}}
        #[arg(long)]
        input_json: String,
    },
    /// Update custodian config
    ConfigUpdate {
        #[arg(long)]
        custodian_id: String,
        /// JSON config, e.g. {"komainu":{"name":"...","apiKey":"...","apiSecret":"...","testingInstance":true,"secretKey":"..."}}
        #[arg(long)]
        config_json: String,
    },
}

#[derive(Subcommand)]
pub enum DocumentAction {
    /// Attach a document to a customer
    Attach {
        #[arg(long)]
        customer_id: String,
        #[arg(long)]
        file: String,
    },
    /// Get a document by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// List documents for a customer
    List {
        #[arg(long)]
        customer_id: String,
    },
    /// Generate download link for a document
    DownloadLink {
        #[arg(long)]
        document_id: String,
    },
    /// Archive a document
    Archive {
        #[arg(long)]
        document_id: String,
    },
    /// Delete a document
    Delete {
        #[arg(long)]
        document_id: String,
    },
}

#[derive(Subcommand)]
pub enum DomainConfigAction {
    /// List all domain configs
    List,
    /// Update a domain config
    Update {
        #[arg(long)]
        domain_config_id: String,
        /// JSON value to set
        #[arg(long)]
        value_json: String,
    },
}

#[derive(Subcommand)]
pub enum AuditAction {
    /// List audit logs
    List {
        #[arg(long, default_value = "25")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get audit log for a customer
    Customer {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum FinancialStatementAction {
    /// Get balance sheet
    BalanceSheet {
        #[arg(long)]
        from: String,
        #[arg(long)]
        until: Option<String>,
    },
    /// Get trial balance
    TrialBalance {
        #[arg(long)]
        from: String,
        #[arg(long)]
        until: String,
    },
    /// Get profit and loss statement
    ProfitAndLoss {
        #[arg(long)]
        from: String,
        #[arg(long)]
        until: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SumsubAction {
    /// Create Sumsub permalink for a prospect
    PermalinkCreate {
        #[arg(long)]
        prospect_id: String,
    },
    /// Create a full Sumsub test applicant for a prospect
    TestApplicantCreate {
        #[arg(long)]
        prospect_id: String,
    },
}

#[derive(Subcommand)]
pub enum LoanAgreementAction {
    /// Find a loan agreement by ID
    Find {
        #[arg(long)]
        id: String,
    },
    /// Generate a loan agreement
    Generate {
        #[arg(long)]
        customer_id: String,
    },
    /// Generate download link for a loan agreement
    DownloadLink {
        #[arg(long)]
        loan_agreement_id: String,
    },
}

#[derive(Subcommand)]
pub enum UserAction {
    /// List roles
    RolesList,
    /// Create a user
    Create {
        #[arg(long)]
        email: String,
        #[arg(long)]
        role_id: String,
    },
}

#[derive(Subcommand)]
pub enum ReportAction {
    /// Find a report run by ID
    Find {
        #[arg(long)]
        id: String,
    },
    /// List report runs
    List {
        #[arg(long, default_value = "25")]
        first: i64,
    },
    /// Generate download link for a report file
    DownloadLink {
        #[arg(long)]
        report_id: String,
        #[arg(long)]
        extension: String,
    },
    /// Trigger a report run
    Trigger,
}

#[derive(Subcommand)]
pub enum WithdrawalAction {
    /// Find a withdrawal by ID
    Find {
        #[arg(long)]
        id: String,
    },
}
