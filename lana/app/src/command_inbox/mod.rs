use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use tracing_macros::record_error_severity;

use obix::inbox::{Inbox, InboxConfig, InboxEvent, InboxHandler, InboxResult};

use crate::{customer::Customers, deposit::Deposits, job::Jobs, primitives::Subject};

mod error;

use core_customer::{CustomerId, CustomerType};
use core_deposit::{DepositAccountId, DepositId, UsdCents, WithdrawalId};
pub use error::CommandInboxError;

const COMMAND_INBOX_JOB: job::JobType = job::JobType::new("command-inbox");

/// Unified payload type for async commands processed via the inbox.
///
/// This enum centralizes all async commands from different modules,
/// making it easy to see all available commands in one place.
///
/// # Available Commands (for GraphQL generation)
///
/// | Command            | GraphQL Mutation           | Module    | Description                        |
/// |--------------------|----------------------------|-----------|------------------------------------|
/// | `CreateCustomer`   | `customerCreateAsync`      | customer  | Create a new customer              |
/// | `RecordDeposit`    | `depositRecordAsync`       | deposit   | Record a deposit to an account     |
/// | `InitiateWithdrawal` | `withdrawalInitiateAsync` | deposit   | Initiate a withdrawal from account |
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandInboxPayload {
    /// Create a new customer
    CreateCustomer {
        customer_id: CustomerId,
        email: String,
        telegram_id: String,
        customer_type: CustomerType,
    },
    /// Record a deposit to a deposit account
    RecordDeposit {
        deposit_id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    },
    /// Initiate a withdrawal from a deposit account
    InitiateWithdrawal {
        withdrawal_id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    },
}

/// Handler that processes command inbox events
struct CommandInboxHandler {
    customers: Customers,
    deposits: Deposits,
}

impl Clone for CommandInboxHandler {
    fn clone(&self) -> Self {
        Self {
            customers: self.customers.clone(),
            deposits: self.deposits.clone(),
        }
    }
}

impl InboxHandler for CommandInboxHandler {
    async fn handle(
        &self,
        event: &InboxEvent,
    ) -> Result<InboxResult, Box<dyn std::error::Error + Send + Sync>> {
        let payload: CommandInboxPayload = event.payload()?;
        match payload {
            CommandInboxPayload::CreateCustomer {
                customer_id,
                email,
                telegram_id,
                customer_type,
            } => {
                self.create_customer_internal(customer_id, email, telegram_id, customer_type)
                    .await?;
            }
            CommandInboxPayload::RecordDeposit {
                deposit_id,
                deposit_account_id,
                amount,
                reference,
            } => {
                self.record_deposit_internal(deposit_id, deposit_account_id, amount, reference)
                    .await?;
            }
            CommandInboxPayload::InitiateWithdrawal {
                withdrawal_id,
                deposit_account_id,
                amount,
                reference,
            } => {
                self.initiate_withdrawal_internal(
                    withdrawal_id,
                    deposit_account_id,
                    amount,
                    reference,
                )
                .await?;
            }
        }
        Ok(InboxResult::Complete)
    }
}

impl CommandInboxHandler {
    fn new(customers: &Customers, deposits: &Deposits) -> Self {
        Self {
            customers: customers.clone(),
            deposits: deposits.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "command_inbox.create_customer_internal", skip(self))]
    async fn create_customer_internal(
        &self,
        customer_id: CustomerId,
        email: String,
        telegram_id: String,
        customer_type: CustomerType,
    ) -> Result<core_customer::Customer, CommandInboxError> {
        let customer = self
            .customers
            .create_with_id(customer_id, email, telegram_id, customer_type)
            .await?;
        Ok(customer)
    }

    #[record_error_severity]
    #[instrument(name = "command_inbox.record_deposit_internal", skip(self))]
    async fn record_deposit_internal(
        &self,
        deposit_id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<core_deposit::Deposit, CommandInboxError> {
        let deposit = self
            .deposits
            .record_deposit_with_id(deposit_id, deposit_account_id, amount, reference)
            .await?;
        Ok(deposit)
    }

    #[record_error_severity]
    #[instrument(name = "command_inbox.initiate_withdrawal_internal", skip(self))]
    async fn initiate_withdrawal_internal(
        &self,
        withdrawal_id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<core_deposit::Withdrawal, CommandInboxError> {
        let withdrawal = self
            .deposits
            .initiate_withdrawal_with_id(withdrawal_id, deposit_account_id, amount, reference)
            .await?;
        Ok(withdrawal)
    }
}

/// Unified command inbox for async command processing across modules
pub struct CommandInbox {
    inbox: Inbox,
    customers: Customers,
    deposits: Deposits,
}

impl Clone for CommandInbox {
    fn clone(&self) -> Self {
        Self {
            inbox: self.inbox.clone(),
            customers: self.customers.clone(),
            deposits: self.deposits.clone(),
        }
    }
}

impl CommandInbox {
    pub async fn init(
        pool: &PgPool,
        jobs: &mut Jobs,
        customers: &Customers,
        deposits: &Deposits,
    ) -> Result<Self, CommandInboxError> {
        let handler = CommandInboxHandler::new(customers, deposits);
        let inbox_config = InboxConfig::new(COMMAND_INBOX_JOB);
        let inbox = Inbox::new(pool, jobs, inbox_config, handler);

        Ok(Self {
            inbox,
            customers: customers.clone(),
            deposits: deposits.clone(),
        })
    }

    /// Create a customer asynchronously using the inbox pattern.
    /// Waits for the inbox job to complete and returns the created customer.
    #[record_error_severity]
    #[instrument(name = "command_inbox.create_customer_async", skip(self))]
    pub async fn create_customer_async(
        &self,
        sub: &Subject,
        email: String,
        telegram_id: String,
        customer_type: CustomerType,
    ) -> Result<core_customer::Customer, CommandInboxError> {
        // Auth check happens before persisting to inbox
        self.customers
            .subject_can_create_customer(sub, false)
            .await?;

        // Generate customer ID upfront so we can poll for it after job completes
        let customer_id = CustomerId::new();

        // Generate idempotency key using the customer ID for uniqueness
        let idempotency_key = format!("customer:{}", customer_id);

        let payload = CommandInboxPayload::CreateCustomer {
            customer_id,
            email,
            telegram_id,
            customer_type,
        };

        let result = self
            .inbox
            .persist_and_process(&idempotency_key, payload)
            .await?;

        match result {
            es_entity::Idempotent::Executed(_) => {}
            es_entity::Idempotent::AlreadyApplied => {
                return Err(CommandInboxError::DuplicateIdempotencyKey);
            }
        };

        // Poll for the customer to be created by the inbox job
        let customer = self
            .poll_for_customer(customer_id, std::time::Duration::from_millis(100), 50)
            .await?;

        Ok(customer)
    }

    /// Record a deposit asynchronously using the inbox pattern.
    /// Waits for the inbox job to complete and returns the created deposit.
    #[record_error_severity]
    #[instrument(name = "command_inbox.record_deposit_async", skip(self))]
    pub async fn record_deposit_async(
        &self,
        sub: &Subject,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<core_deposit::Deposit, CommandInboxError> {
        // Auth check happens before persisting to inbox
        self.deposits.subject_can_record_deposit(sub, false).await?;

        // Generate deposit ID upfront so we can poll for it after job completes
        let deposit_id = DepositId::new();

        // Generate idempotency key using the deposit ID for uniqueness
        let idempotency_key = format!("deposit:{}", deposit_id);

        let payload = CommandInboxPayload::RecordDeposit {
            deposit_id,
            deposit_account_id,
            amount,
            reference,
        };

        let result = self
            .inbox
            .persist_and_process(&idempotency_key, payload)
            .await?;

        match result {
            es_entity::Idempotent::Executed(_) => {}
            es_entity::Idempotent::AlreadyApplied => {
                return Err(CommandInboxError::DuplicateIdempotencyKey);
            }
        };

        // Poll for the deposit to be created by the inbox job
        let deposit = self
            .poll_for_deposit(deposit_id, std::time::Duration::from_millis(100), 50)
            .await?;

        Ok(deposit)
    }

    /// Initiate a withdrawal asynchronously using the inbox pattern.
    /// Waits for the inbox job to complete and returns the created withdrawal.
    #[record_error_severity]
    #[instrument(name = "command_inbox.initiate_withdrawal_async", skip(self))]
    pub async fn initiate_withdrawal_async(
        &self,
        sub: &Subject,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<core_deposit::Withdrawal, CommandInboxError> {
        // Auth check happens before persisting to inbox
        self.deposits
            .subject_can_initiate_withdrawal(sub, false)
            .await?;

        // Generate withdrawal ID upfront so we can poll for it after job completes
        let withdrawal_id = WithdrawalId::new();

        // Generate idempotency key using the withdrawal ID for uniqueness
        let idempotency_key = format!("withdrawal:{}", withdrawal_id);

        let payload = CommandInboxPayload::InitiateWithdrawal {
            withdrawal_id,
            deposit_account_id,
            amount,
            reference,
        };

        let result = self
            .inbox
            .persist_and_process(&idempotency_key, payload)
            .await?;

        match result {
            es_entity::Idempotent::Executed(_) => {}
            es_entity::Idempotent::AlreadyApplied => {
                return Err(CommandInboxError::DuplicateIdempotencyKey);
            }
        };

        // Poll for the withdrawal to be created by the inbox job
        let withdrawal = self
            .poll_for_withdrawal(withdrawal_id, std::time::Duration::from_millis(100), 50)
            .await?;

        Ok(withdrawal)
    }

    /// Poll for a customer to exist, with configurable interval and max attempts
    async fn poll_for_customer(
        &self,
        customer_id: CustomerId,
        interval: std::time::Duration,
        max_attempts: u32,
    ) -> Result<core_customer::Customer, CommandInboxError> {
        for _ in 0..max_attempts {
            match self.customers.find_by_id_internal(customer_id).await {
                Ok(customer) => return Ok(customer),
                Err(_) => {
                    tokio::time::sleep(interval).await;
                }
            }
        }
        Err(CommandInboxError::CustomerNotFoundAfterProcessing)
    }

    /// Poll for a deposit to exist, with configurable interval and max attempts
    async fn poll_for_deposit(
        &self,
        deposit_id: DepositId,
        interval: std::time::Duration,
        max_attempts: u32,
    ) -> Result<core_deposit::Deposit, CommandInboxError> {
        for _ in 0..max_attempts {
            match self.deposits.find_deposit_by_id_internal(deposit_id).await {
                Ok(deposit) => return Ok(deposit),
                Err(_) => {
                    tokio::time::sleep(interval).await;
                }
            }
        }
        Err(CommandInboxError::DepositNotFoundAfterProcessing)
    }

    /// Poll for a withdrawal to exist, with configurable interval and max attempts
    async fn poll_for_withdrawal(
        &self,
        withdrawal_id: WithdrawalId,
        interval: std::time::Duration,
        max_attempts: u32,
    ) -> Result<core_deposit::Withdrawal, CommandInboxError> {
        for _ in 0..max_attempts {
            match self
                .deposits
                .find_withdrawal_by_id_internal(withdrawal_id)
                .await
            {
                Ok(withdrawal) => return Ok(withdrawal),
                Err(_) => {
                    tokio::time::sleep(interval).await;
                }
            }
        }
        Err(CommandInboxError::WithdrawalNotFoundAfterProcessing)
    }
}
