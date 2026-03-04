use clap::{ArgAction, Parser, Subcommand};

const CLI_AFTER_HELP: &str = r#"Examples:
  lana-admin login \
    --admin-url https://admin.qa.lana.galoy.io/graphql \
    --keycloak-url https://auth.qa.lana.galoy.io \
    --keycloak-client-id admin-panel \
    --username galoysuperuser@mailinator.com

  lana-admin customer list --first 5 --json

  lana-admin accounting manual-transaction \
    --description "QA test entry" \
    --entries-json '[{"accountRef":"11.01.0101","amount":"1","currency":"USD","direction":"CREDIT","description":"Entry 1"},{"accountRef":"61.01","amount":"1","currency":"USD","direction":"DEBIT","description":"Entry 2"}]' \
    --json
"#;

const MANUAL_TRANSACTION_ENTRIES_HELP: &str = r#"JSON array of transaction entries.
Example:
[
  {"accountRef":"11.01.0101","amount":"1","currency":"USD","direction":"CREDIT","description":"Entry 1"},
  {"accountRef":"61.01","amount":"1","currency":"USD","direction":"DEBIT","description":"Entry 2"}
]
Accepted direction values: DEBIT, CREDIT"#;

const CUSTODIAN_CREATE_INPUT_HELP: &str = r#"CustodianCreateInput as JSON.
Provider-specific object shape is required (do not pass a generic object).
Minimal Komainu example:
{
  "komainu": {
    "name": "test-komainu",
    "apiKey": "test-api-key",
    "apiSecret": "test-api-secret",
    "testingInstance": true,
    "secretKey": "test-secret-key",
    "webhookSecret": "test-webhook-secret"
  }
  }
}

If your environment uses another provider key (for example `bitgo`), pass that
provider object with its required fields instead."#;

const CUSTODIAN_CONFIG_HELP: &str = r#"CustodianConfigInput as JSON.
Provider-specific object shape is required (do not pass generic payloads such as {"enabled":true}).
Minimal Komainu example:
{
  "komainu": {
    "name": "updated-komainu",
    "apiKey": "updated-api-key",
    "apiSecret": "updated-api-secret",
    "testingInstance": false,
    "secretKey": "updated-secret-key",
    "webhookSecret": "updated-webhook-secret"
  }
  }
}

Use the same provider key and schema family that was used when creating the custodian."#;

const DOMAIN_CONFIG_VALUE_HELP: &str = r#"JSON value for the target config key.
Examples:
  --value-json '"notifications@example.com"'
  --value-json '123'
  --value-json '{"enabled":true}'"#;

#[derive(Parser)]
#[command(
    name = "lana-admin",
    about = "LANA Bank Admin CLI",
    long_about = "Admin CLI for Lana Bank backoffice actions. Use `login` to cache credentials, then run entity commands (customer, credit-facility, accounting, etc).",
    after_long_help = CLI_AFTER_HELP,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Output as JSON instead of tables
    #[arg(long, global = true)]
    pub json: bool,

    /// Increase verbosity for GraphQL debug output (`-v`: operation + variables, `-vv`: also raw response)
    #[arg(short = 'v', long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

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
    /// Manage loan agreements
    LoanAgreement {
        #[command(subcommand)]
        action: LoanAgreementAction,
    },
    /// Manage users
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    /// Manage roles
    Role {
        #[command(subcommand)]
        action: RoleAction,
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
    /// Authenticate and cache login profile + session token for subsequent commands
    Login {
        /// Admin GraphQL endpoint URL
        #[arg(
            long,
            env = "LANA_ADMIN_URL",
            default_value = "http://admin.localhost:4455/graphql"
        )]
        admin_url: String,

        /// Keycloak URL
        #[arg(
            long,
            env = "LANA_KEYCLOAK_URL",
            default_value = "http://localhost:8081"
        )]
        keycloak_url: String,

        /// Keycloak client ID for login flow (QA/staging default: admin-panel)
        #[arg(long, env = "LANA_KEYCLOAK_CLIENT_ID", default_value = "admin-panel")]
        keycloak_client_id: String,

        /// Admin username (Keycloak email)
        #[arg(long, env = "LANA_USERNAME", default_value = "admin@galoy.io")]
        username: String,

        /// Admin password (empty for passwordless/browser-based flows)
        #[arg(long, env = "LANA_PASSWORD", default_value = "")]
        password: String,
    },
    /// Clear cached session token
    Logout,
    /// Show CLI version and (if logged in) server build info
    Version,
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
        #[arg(long, help = "External/operator reference string for this withdrawal")]
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
        #[arg(
            long,
            value_name = "DATE",
            help = "Effective date (YYYY-MM-DD); RFC3339 datetime is accepted and normalized"
        )]
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
        #[arg(
            long,
            value_parser = ["DEBIT", "CREDIT"],
            help = "Normal balance type (accepted: DEBIT, CREDIT)"
        )]
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
        #[arg(
            long,
            value_parser = [
                "ASSET",
                "LIABILITY",
                "EQUITY",
                "REVENUE",
                "COST_OF_REVENUE",
                "EXPENSES",
                "OFF_BALANCE_SHEET"
            ],
            help = "Account category"
        )]
        category: String,
    },
    /// Execute a manual transaction
    ManualTransaction {
        #[arg(long)]
        description: String,
        #[arg(long)]
        reference: Option<String>,
        #[arg(
            long,
            value_name = "DATE",
            help = "Effective date (YYYY-MM-DD); RFC3339 datetime is accepted and normalized"
        )]
        effective: Option<String>,
        #[arg(long, long_help = MANUAL_TRANSACTION_ENTRIES_HELP)]
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
        #[arg(long, long_help = CUSTODIAN_CREATE_INPUT_HELP)]
        input_json: String,
    },
    /// Update custodian config
    ConfigUpdate {
        #[arg(long)]
        custodian_id: String,
        #[arg(long, long_help = CUSTODIAN_CONFIG_HELP)]
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
        #[arg(long, long_help = DOMAIN_CONFIG_VALUE_HELP)]
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
        #[arg(long, help = "Start date in YYYY-MM-DD format")]
        from: String,
        #[arg(long, help = "Optional end date in YYYY-MM-DD format")]
        until: Option<String>,
    },
    /// Get trial balance
    TrialBalance {
        #[arg(long, help = "Start date in YYYY-MM-DD format")]
        from: String,
        #[arg(long, help = "End date in YYYY-MM-DD format")]
        until: String,
    },
    /// Get profit and loss statement
    ProfitAndLoss {
        #[arg(long, help = "Start date in YYYY-MM-DD format")]
        from: String,
        #[arg(long, help = "Optional end date in YYYY-MM-DD format")]
        until: Option<String>,
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
    /// Create a user
    Create {
        #[arg(long)]
        email: String,
        #[arg(long)]
        role_id: String,
    },
    /// Update a user's role
    UpdateRole {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        role_id: String,
    },
}

#[derive(Subcommand)]
pub enum RoleAction {
    /// List roles
    List {
        #[arg(long, default_value = "100")]
        first: i64,
        #[arg(long)]
        after: Option<String>,
    },
    /// Get a role by ID, including its permission sets
    Get {
        #[arg(long)]
        id: String,
    },
    /// Add permission sets to a role
    AddPermissionSets {
        #[arg(long)]
        role_id: String,
        #[arg(
            long = "permission-set-id",
            required = true,
            num_args = 1..,
            value_delimiter = ',',
            help = "Permission set ID(s); repeat the flag or pass comma-separated values"
        )]
        permission_set_ids: Vec<String>,
    },
    /// Remove permission sets from a role
    RemovePermissionSets {
        #[arg(long)]
        role_id: String,
        #[arg(
            long = "permission-set-id",
            required = true,
            num_args = 1..,
            value_delimiter = ',',
            help = "Permission set ID(s); repeat the flag or pass comma-separated values"
        )]
        permission_set_ids: Vec<String>,
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
        #[arg(
            long,
            help = "File extension for the report file (use values returned by report list/find, e.g. pdf)"
        )]
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
