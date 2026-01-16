use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use core_accounting::LedgerTransactionInitiator;
use core_custody::CoreCustodyEvent;
use governance::GovernanceEvent;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use core_money::UsdCents;

use crate::{
    CoreCreditEvent, CreditFacilityId, LiquidationId, NewPayment, PaymentId,
    PaymentLedgerAccountIds, ledger::CreditLedger, payment::PaymentRepo,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct LiquidationPaymentJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct LiquidationPaymentJobConfig<E> {
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for LiquidationPaymentJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            liquidation_id: self.liquidation_id,
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct LiquidationPaymentInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    payment_repo: Arc<PaymentRepo<E>>,
    ledger: Arc<CreditLedger>,
}

impl<E> LiquidationPaymentInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        payment_repo: Arc<PaymentRepo<E>>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            payment_repo,
            ledger,
        }
    }
}

const LIQUIDATION_PAYMENT_JOB: JobType = JobType::new("outbox.liquidation-payment");

impl<E> JobInitializer for LiquidationPaymentInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = LiquidationPaymentJobConfig<E>;

    fn job_type(&self) -> JobType {
        LIQUIDATION_PAYMENT_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(LiquidationPaymentJobRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            payment_repo: self.payment_repo.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub type LiquidationPaymentJobSpawner<E> = JobSpawner<LiquidationPaymentJobConfig<E>>;

pub struct LiquidationPaymentJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: LiquidationPaymentJobConfig<E>,
    outbox: Outbox<E>,
    payment_repo: Arc<PaymentRepo<E>>,
    ledger: Arc<CreditLedger>,
}

impl<E> LiquidationPaymentJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "outbox.core_credit.liquidation_payment.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(
                event @ PartialLiquidationProceedsReceived {
                    amount,
                    credit_facility_id,
                    liquidation_id,
                    payment_id,
                    facility_payment_holding_account_id,
                    facility_proceeds_from_liquidation_account_id,
                    facility_uncovered_outstanding_account_id,
                    ..
                },
            ) if *liquidation_id == self.config.liquidation_id => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("payment_id", tracing::field::display(payment_id));

                let payment_ledger_account_ids = PaymentLedgerAccountIds {
                    facility_payment_holding_account_id: *facility_payment_holding_account_id,
                    facility_uncovered_outstanding_account_id:
                        *facility_uncovered_outstanding_account_id,
                    payment_source_account_id: facility_proceeds_from_liquidation_account_id.into(),
                };

                self.record_payment(
                    db,
                    *payment_id,
                    *credit_facility_id,
                    payment_ledger_account_ids,
                    *amount,
                )
                .await?;

                Ok(ControlFlow::Break(()))
            }
            Some(event @ PartialLiquidationCompleted { liquidation_id, .. })
                if *liquidation_id == self.config.liquidation_id =>
            {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                tracing::info!(
                    liquidation_id = %liquidation_id,
                    "Liquidation completed, terminating liquidation payment job"
                );

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }

    #[instrument(
        name = "outbox.core_credit.liquidation_payment.record_payment",
        skip(self, db)
    )]
    async fn record_payment(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        credit_facility_id: CreditFacilityId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        amount: UsdCents,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let effective = chrono::Utc::now().date_naive();

        let new_payment = NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .payment_ledger_account_ids(payment_ledger_account_ids)
            .effective(effective)
            .build()
            .expect("could not build new payment");

        if self
            .payment_repo
            .maybe_find_by_id_in_op(&mut *db, payment_id)
            .await?
            .is_some()
        {
            return Ok(());
        }

        let payment = self.payment_repo.create_in_op(db, new_payment).await?;

        self.ledger
            .record_payment(db, &payment, LedgerTransactionInitiator::System)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for LiquidationPaymentJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<LiquidationPaymentJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %LIQUIDATION_PAYMENT_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self
                                .payment_repo
                                .begin_op_with_clock(current_job.clock())
                                .await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&mut db, &message).await?;

                            db.commit().await?;

                            if next.is_break() {
                                return Ok(JobCompletion::Complete);
                            }
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
