use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use core_customer::CustomerId;
use core_money::UsdCents;

use super::{error::ApplicantError, sumsub_auth::SumsubClient};

/// Converts USD cents to dollars for Sumsub API
pub fn usd_cents_to_dollars(cents: UsdCents) -> f64 {
    (cents.into_inner() as f64) / 100.0
}

/// Direction of the transaction from Sumsub's perspective
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SumsubTransactionDirection {
    /// Money coming into the customer's account (deposit)
    #[serde(rename = "in")]
    In,
    /// Money going out of the customer's account (withdrawal)
    #[serde(rename = "out")]
    Out,
}

impl std::fmt::Display for SumsubTransactionDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SumsubTransactionDirection::In => write!(f, "in"),
            SumsubTransactionDirection::Out => write!(f, "out"),
        }
    }
}

/// Core transaction export service for Sumsub compliance
#[derive(Clone)]
pub struct TransactionExporter {
    sumsub_client: SumsubClient,
}

impl TransactionExporter {
    pub fn new(sumsub_client: SumsubClient) -> Self {
        Self { sumsub_client }
    }

    /// Submit a deposit transaction to Sumsub for monitoring
    #[instrument(name = "applicant.submit_deposit_transaction", skip(self), err)]
    pub async fn submit_deposit_transaction(
        &self,
        transaction_id: impl Into<String> + std::fmt::Debug,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), ApplicantError> {
        let transaction_id = transaction_id.into();

        self.sumsub_client
            .submit_finance_transaction(
                customer_id,
                transaction_id,
                "Deposit",
                &SumsubTransactionDirection::In.to_string(),
                usd_cents_to_dollars(amount),
                "USD",
            )
            .await
    }

    /// Submit a withdrawal transaction to Sumsub for monitoring
    #[instrument(name = "applicant.submit_withdrawal_transaction", skip(self), err)]
    pub async fn submit_withdrawal_transaction(
        &self,
        transaction_id: impl Into<String> + std::fmt::Debug,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), ApplicantError> {
        let transaction_id = transaction_id.into();

        self.sumsub_client
            .submit_finance_transaction(
                customer_id,
                transaction_id,
                "Withdrawal",
                &SumsubTransactionDirection::Out.to_string(),
                usd_cents_to_dollars(amount),
                "USD",
            )
            .await
    }
}

/// Transaction data for export to Sumsub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub transaction_id: String,
    pub customer_id: CustomerId,
    pub amount: UsdCents,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
}

impl TransactionData {
    pub fn new_deposit(
        transaction_id: impl Into<String>,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            customer_id,
            amount,
            transaction_type: TransactionType::Deposit,
        }
    }

    pub fn new_withdrawal(
        transaction_id: impl Into<String>,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            customer_id,
            amount,
            transaction_type: TransactionType::Withdrawal,
        }
    }
}

/// Trait for processing transaction exports
#[async_trait]
pub trait TransactionProcessor {
    async fn process_transaction(&self, transaction: TransactionData)
        -> Result<(), ApplicantError>;
}

#[async_trait]
impl TransactionProcessor for TransactionExporter {
    async fn process_transaction(
        &self,
        transaction: TransactionData,
    ) -> Result<(), ApplicantError> {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                self.submit_deposit_transaction(
                    transaction.transaction_id,
                    transaction.customer_id,
                    transaction.amount,
                )
                .await
            }
            TransactionType::Withdrawal => {
                self.submit_withdrawal_transaction(
                    transaction.transaction_id,
                    transaction.customer_id,
                    transaction.amount,
                )
                .await
            }
        }
    }
}
