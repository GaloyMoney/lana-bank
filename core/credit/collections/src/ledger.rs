pub mod error {
    use thiserror::Error;
    use tracing::Level;
    use tracing_utils::ErrorSeverity;

    #[derive(Error, Debug)]
    pub enum CollectionsLedgerError {
        #[error("CollectionsLedgerError - Sqlx: {0}")]
        Sqlx(#[from] sqlx::Error),
        #[error("CollectionsLedgerError - CalaLedger: {0}")]
        CalaLedger(#[from] cala_ledger::error::LedgerError),
        #[error("CollectionsLedgerError - PaymentAmountGreaterThanOutstandingObligations")]
        PaymentAmountGreaterThanOutstandingObligations,
    }

    impl ErrorSeverity for CollectionsLedgerError {
        fn severity(&self) -> Level {
            match self {
                Self::Sqlx(_) => Level::ERROR,
                Self::CalaLedger(_) => Level::ERROR,
                Self::PaymentAmountGreaterThanOutstandingObligations => Level::WARN,
            }
        }
    }
}

use async_trait::async_trait;
use core_accounting::LedgerTransactionInitiator;

use crate::{
    obligation::primitives::{
        ObligationDefaultedReallocationData, ObligationDueReallocationData,
        ObligationOverdueReallocationData,
    },
    payment::Payment,
    payment_allocation::PaymentAllocation,
};

use error::CollectionsLedgerError;

#[async_trait]
pub trait CollectionsLedger: Send + Sync + 'static {
    async fn record_payment(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment: &Payment,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError>;

    async fn record_payment_allocations(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payments: Vec<PaymentAllocation>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError>;

    async fn record_obligation_due(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationDueReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError>;

    async fn record_obligation_overdue(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationOverdueReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError>;

    async fn record_obligation_defaulted(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationDefaultedReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError>;
}
