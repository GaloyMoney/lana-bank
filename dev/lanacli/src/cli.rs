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
    /// Authenticate and cache session token
    Login,
    /// Clear cached session token
    Logout,
    /// Interactive TUI mode (coming soon)
    Tui,
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
    /// Initiate a disbursal
    DisbursalInitiate {
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
