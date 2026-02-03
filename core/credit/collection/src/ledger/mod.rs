pub mod error;
pub mod templates;

use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    CalaLedger, Currency, JournalId,
    error::LedgerError,
    velocity::error::{LimitExceededError, VelocityError},
};
use core_accounting::LedgerTransactionInitiator;

pub use error::CollectionLedgerError;

use crate::{
    obligation::primitives::{
        ObligationDefaultedReallocationData, ObligationDueReallocationData,
        ObligationOverdueReallocationData,
    },
    payment::Payment,
    payment_allocation::PaymentAllocation,
    primitives::CalaAccountId,
};

pub const UNCOVERED_OUTSTANDING_LIMIT_ID: uuid::Uuid =
    uuid::uuid!("00000000-0000-0000-0000-000000000003");

#[derive(Clone)]
pub struct CollectionLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    usd: Currency,
    payments_made_omnibus_account_id: CalaAccountId,
}

impl CollectionLedger {
    #[record_error_severity]
    #[instrument(name = "collection_ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        payments_made_omnibus_account_id: CalaAccountId,
    ) -> Result<Self, CollectionLedgerError> {
        templates::RecordPayment::init(cala).await?;
        templates::RecordPaymentAllocation::init(cala).await?;
        templates::RecordObligationDueBalance::init(cala).await?;
        templates::RecordObligationOverdueBalance::init(cala).await?;
        templates::RecordObligationDefaultedBalance::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            usd: Currency::USD,
            payments_made_omnibus_account_id,
        })
    }

    #[record_error_severity]
    #[instrument(name = "collection_ledger.record_payment", skip(self, op, payment))]
    pub async fn record_payment(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment @ Payment {
            ledger_tx_id,
            facility_payment_holding_account_id,
            facility_uncovered_outstanding_account_id,
            payment_source_account_id,
            amount,
            effective,
            ..
        }: &Payment,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionLedgerError> {
        let params = templates::RecordPaymentParams {
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            payment_source_account_id: *payment_source_account_id,
            payment_holding_account_id: *facility_payment_holding_account_id,
            uncovered_outstanding_account_id: *facility_uncovered_outstanding_account_id,
            payments_made_omnibus_account_id: self.payments_made_omnibus_account_id,
            tx_ref: payment.tx_ref(),
            effective: *effective,
            initiated_by,
        };

        match self
            .cala
            .post_transaction_in_op(op, *ledger_tx_id, templates::RECORD_PAYMENT_CODE, params)
            .await
        {
            Err(LedgerError::VelocityError(VelocityError::Enforcement(LimitExceededError {
                limit_id,
                ..
            }))) if limit_id == UNCOVERED_OUTSTANDING_LIMIT_ID.into() => {
                return Err(CollectionLedgerError::PaymentAmountGreaterThanOutstandingObligations);
            }
            Err(e) => return Err(e.into()),
            _ => (),
        };

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "collection_ledger.record_payment_allocations",
        skip(self, op, payments)
    )]
    pub async fn record_payment_allocations(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payments: Vec<PaymentAllocation>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionLedgerError> {
        for payment in payments {
            self.record_obligation_repayment_in_op(op, payment, initiated_by)
                .await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "collection_ledger.record_obligation_due", skip(self, op))]
    pub async fn record_obligation_due(
        &self,
        op: &mut es_entity::DbOp<'_>,
        ObligationDueReallocationData {
            tx_id,
            amount: outstanding_amount,
            not_yet_due_account_id,
            due_account_id,
            effective,
            ..
        }: ObligationDueReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionLedgerError> {
        self.cala
            .post_transaction_in_op(
                op,
                tx_id,
                templates::RECORD_OBLIGATION_DUE_BALANCE_CODE,
                templates::RecordObligationDueBalanceParams {
                    journal_id: self.journal_id,
                    amount: outstanding_amount.to_usd(),
                    receivable_not_yet_due_account_id: not_yet_due_account_id,
                    receivable_due_account_id: due_account_id,
                    effective,
                    initiated_by,
                },
            )
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "collection_ledger.record_obligation_overdue", skip(self, op))]
    pub async fn record_obligation_overdue(
        &self,
        op: &mut es_entity::DbOp<'_>,
        ObligationOverdueReallocationData {
            tx_id,
            amount: outstanding_amount,
            due_account_id,
            overdue_account_id,
            effective,
            ..
        }: ObligationOverdueReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionLedgerError> {
        self.cala
            .post_transaction_in_op(
                op,
                tx_id,
                templates::RECORD_OBLIGATION_OVERDUE_BALANCE_CODE,
                templates::RecordObligationOverdueBalanceParams {
                    journal_id: self.journal_id,
                    amount: outstanding_amount.to_usd(),
                    receivable_due_account_id: due_account_id,
                    receivable_overdue_account_id: overdue_account_id,
                    effective,
                    initiated_by,
                },
            )
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "collection_ledger.record_obligation_defaulted", skip(self, op))]
    pub async fn record_obligation_defaulted(
        &self,
        op: &mut es_entity::DbOp<'_>,
        ObligationDefaultedReallocationData {
            tx_id,
            amount: outstanding_amount,
            receivable_account_id,
            defaulted_account_id,
            effective,
            ..
        }: ObligationDefaultedReallocationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionLedgerError> {
        self.cala
            .post_transaction_in_op(
                op,
                tx_id,
                templates::RECORD_OBLIGATION_DEFAULTED_BALANCE_CODE,
                templates::RecordObligationDefaultedBalanceParams {
                    journal_id: self.journal_id,
                    amount: outstanding_amount.to_usd(),
                    receivable_account_id,
                    defaulted_account_id,
                    effective,
                    initiated_by,
                },
            )
            .await?;
        Ok(())
    }

    async fn record_obligation_repayment_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        allocation @ PaymentAllocation {
            ledger_tx_id,
            amount,
            payment_holding_account_id,
            receivable_account_id,
            effective,
            ..
        }: PaymentAllocation,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollectionLedgerError> {
        let params = templates::RecordPaymentAllocationParams {
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            receivable_account_id,
            payment_holding_account_id,
            tx_ref: allocation.tx_ref(),
            effective,
            initiated_by,
        };
        self.cala
            .post_transaction_in_op(
                op,
                ledger_tx_id,
                templates::RECORD_PAYMENT_ALLOCATION_CODE,
                params,
            )
            .await?;

        Ok(())
    }
}
