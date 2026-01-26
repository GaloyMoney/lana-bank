use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use core_accounting::LedgerTransactionInitiator;
use core_money::UsdCents;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::CollateralId;
use crate::{
    credit_facility::CreditFacilityRepo,
    event::CoreCreditEvent,
    ledger::CreditLedger,
    payment::PaymentRepo,
    payment::{NewPayment, PaymentLedgerAccountIds},
    primitives::{CreditFacilityId, LiquidationId, PaymentId},
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct LiquidationPaymentJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct LiquidationPaymentJobConfig<E> {
    pub collateral_id: CollateralId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for LiquidationPaymentJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            collateral_id: self.collateral_id,
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct LiquidationPaymentInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    payment_repo: Arc<PaymentRepo<E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ledger: Arc<CreditLedger>,
}

impl<E> LiquidationPaymentInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        payment_repo: Arc<PaymentRepo<E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            payment_repo,
            credit_facility_repo,
            ledger,
        }
    }
}

const LIQUIDATION_PAYMENT_JOB: JobType = JobType::new("outbox.liquidation-payment");

impl<E> JobInitializer for LiquidationPaymentInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
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
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub struct LiquidationPaymentJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: LiquidationPaymentJobConfig<E>,
    outbox: Outbox<E>,
    payment_repo: Arc<PaymentRepo<E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ledger: Arc<CreditLedger>,
}

impl<E> LiquidationPaymentJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(
        name = "outbox.core_credit.partial_liquidation.acknowledge_payment_in_credit_facility",
        skip(self, db)
    )]
    async fn acknowledge_payment_in_credit_facility(
        &self,
        db: &mut es_entity::DbOp<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(db, self.config.credit_facility_id)
            .await?;

        if credit_facility
            .acknowledge_payment_from_liquidation()?
            .did_execute()
        {
            self.credit_facility_repo
                .update_in_op(db, &mut credit_facility)
                .await?;
        }

        Ok(())
    }

    async fn record_payment(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        amount: UsdCents,
        credit_facility_id: CreditFacilityId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        clock: &es_entity::clock::ClockHandle,
    ) -> Result<Option<crate::payment::Payment>, Box<dyn std::error::Error>> {
        if self
            .payment_repo
            .maybe_find_by_id_in_op(&mut *db, payment_id)
            .await?
            .is_some()
        {
            return Ok(None);
        }

        let new_payment = NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .payment_ledger_account_ids(payment_ledger_account_ids)
            .effective(clock.today())
            .build()
            .expect("could not build new payment");

        let payment = self.payment_repo.create_in_op(db, new_payment).await?;
        self.ledger
            .record_payment(db, &payment, LedgerTransactionInitiator::System)
            .await?;

        Ok(Some(payment))
    }

    #[instrument(name = "outbox.core_credit.liquidation_payment.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        clock: &es_entity::clock::ClockHandle,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(
                event @ PartialLiquidationProceedsReceived {
                    amount,
                    credit_facility_id,
                    collateral_id,
                    payment_id,
                    facility_payment_holding_account_id,
                    facility_proceeds_from_liquidation_account_id,
                    facility_uncovered_outstanding_account_id,
                    ..
                },
            ) if *collateral_id == self.config.collateral_id => {
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
                    *amount,
                    *credit_facility_id,
                    payment_ledger_account_ids,
                    clock,
                )
                .await?;

                self.acknowledge_payment_in_credit_facility(db).await?;

                Ok(ControlFlow::Break(()))
            }
            Some(event @ PartialLiquidationCompleted { collateral_id, .. })
                if *collateral_id == self.config.collateral_id =>
            {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                tracing::info!(
                    collateral_id = %collateral_id,
                    "Liquidation completed, terminating liquidation payment job"
                );

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

#[async_trait]
impl<E> JobRunner for LiquidationPaymentJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
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

                            let next = self.process_message(&mut db, message.as_ref(), current_job.clock()).await?;

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

pub type LiquidationPaymentJobSpawner<E> = JobSpawner<LiquidationPaymentJobConfig<E>>;
