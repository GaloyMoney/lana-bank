mod error;

use std::sync::Arc;

use async_trait::async_trait;
use core_accounting::LedgerTransactionInitiator;

pub use error::*;
use crate::primitives::*;

// Forward declare types that will be defined in the module files
use crate::payment::Payment;
use crate::payment_allocation::PaymentAllocation;

#[async_trait]
pub trait LedgerOps: Send + Sync {
    async fn record_payment(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment: &Payment,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError>;

    async fn record_payment_allocations(
        &self,
        op: &mut es_entity::DbOp<'_>,
        allocations: Vec<PaymentAllocation>,
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

pub struct CollectionsLedger<L: LedgerOps> {
    ledger: Arc<L>,
}

impl<L: LedgerOps> CollectionsLedger<L> {
    pub fn new(ledger: Arc<L>) -> Self {
        Self { ledger }
    }

    pub async fn record_payment(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment: &Payment,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError> {
        self.ledger.record_payment(op, payment, initiated_by).await
    }

    pub async fn record_payment_allocations(
        &self,
        op: &mut es_entity::DbOp<'_>,
        allocations: Vec<PaymentAllocation>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError> {
        self.ledger
            .record_payment_allocations(op, allocations, initiated_by)
            .await
    }

    pub async fn record_obligation_due(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationDueReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError> {
        self.ledger
            .record_obligation_due(op, data, initiated_by)
            .await
    }

    pub async fn record_obligation_overdue(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationOverdueReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError> {
        self.ledger
            .record_obligation_overdue(op, data, initiated_by)
            .await
    }

    pub async fn record_obligation_defaulted(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationDefaultedReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionsLedgerError> {
        self.ledger
            .record_obligation_defaulted(op, data, initiated_by)
            .await
    }
}
