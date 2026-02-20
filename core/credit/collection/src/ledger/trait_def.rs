use audit::SystemSubject;

use crate::{
    obligation::{
        ObligationDefaultedReallocationData, ObligationDueReallocationData,
        ObligationOverdueReallocationData,
    },
    payment::Payment,
    payment_allocation::PaymentAllocation,
};

use super::error::CollectionLedgerError;

pub trait CollectionLedgerOps: Clone + Send + Sync + 'static {
    fn record_payment_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment: &Payment,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollectionLedgerError>> + Send;

    fn record_payment_allocations_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payments: Vec<PaymentAllocation>,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollectionLedgerError>> + Send;

    fn record_obligation_due_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationDueReallocationData,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollectionLedgerError>> + Send;

    fn record_obligation_overdue_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationOverdueReallocationData,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollectionLedgerError>> + Send;

    fn record_obligation_defaulted_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        data: ObligationDefaultedReallocationData,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollectionLedgerError>> + Send;
}
