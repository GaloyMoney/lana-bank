use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use core_accounting::LedgerTransactionInitiator;
use core_custody::CoreCustodyEvent;
use es_entity::{DbOp, Idempotent};
use governance::GovernanceEvent;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CoreCreditEvent, CreditFacilityId, LiquidationId, NewPayment, PaymentLedgerAccountIds,
    credit_facility::CreditFacilityRepo, ledger::CreditLedger, liquidation::LiquidationRepo,
    obligation::ObligationRepo, payment::PaymentRepo, payment_allocation::PaymentAllocationRepo,
    primitives::*,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct PartialLiquidationJobConfig<E> {
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for PartialLiquidationJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            liquidation_id: self.liquidation_id,
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    liquidation_repo: Arc<LiquidationRepo<E>>,
    payment_repo: Arc<PaymentRepo>,
    obligation_repo: Arc<ObligationRepo<E>>,
    payment_allocation_repo: Arc<PaymentAllocationRepo<E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ledger: Arc<CreditLedger>,
}

impl<E> PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        liquidation_repo: Arc<LiquidationRepo<E>>,
        payment_repo: Arc<PaymentRepo>,
        obligation_repo: Arc<ObligationRepo<E>>,
        payment_allocation_repo: Arc<PaymentAllocationRepo<E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            liquidation_repo,
            payment_repo,
            obligation_repo,
            payment_allocation_repo,
            credit_facility_repo,
            ledger,
        }
    }
}

const PARTIAL_LIQUIDATION_JOB: JobType = JobType::new("outbox.partial-liquidation");

impl<E> JobInitializer for PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = PartialLiquidationJobConfig<E>;

    fn job_type(&self) -> JobType {
        PARTIAL_LIQUIDATION_JOB
    }

    fn init(
        &self,
        job: &job::Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        let config: PartialLiquidationJobConfig<E> = job.config()?;
        Ok(Box::new(PartialLiquidationJobRunner::<E> {
            config,
            outbox: self.outbox.clone(),
            liquidation_repo: self.liquidation_repo.clone(),
            payment_repo: self.payment_repo.clone(),
            obligation_repo: self.obligation_repo.clone(),
            payment_allocation_repo: self.payment_allocation_repo.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub struct PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: PartialLiquidationJobConfig<E>,
    outbox: Outbox<E>,
    liquidation_repo: Arc<LiquidationRepo<E>>,
    payment_repo: Arc<PaymentRepo>,
    obligation_repo: Arc<ObligationRepo<E>>,
    payment_allocation_repo: Arc<PaymentAllocationRepo<E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ledger: Arc<CreditLedger>,
}

#[async_trait]
impl<E> JobRunner for PartialLiquidationJobRunner<E>
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
            .execution_state::<PartialLiquidationJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %PARTIAL_LIQUIDATION_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self
                                .liquidation_repo
                                .begin_op_with_clock(current_job.clock())
                                .await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&mut db, message.as_ref(), current_job.clock()).await?;

                            db.commit().await?;

                            if next.is_break() {
                                // If the partial liquidation has been completed,
                                // terminate the job, too.
                                return Ok(JobCompletion::Complete);
                            }
                        }
                        None => return Ok(JobCompletion::RescheduleNow)
                    }
                }
            }
        }
    }
}

impl<E> PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn record_payment(
        &self,
        db: &mut DbOp<'_>,
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

    async fn allocate_payment(
        &self,
        db: &mut DbOp<'_>,
        payment: &crate::payment::Payment,
        credit_facility_id: CreditFacilityId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut obligations = Vec::new();
        let mut query = Default::default();

        loop {
            let mut res = self
                .obligation_repo
                .list_for_credit_facility_id_by_created_at(
                    credit_facility_id,
                    query,
                    es_entity::ListDirection::Ascending,
                )
                .await?;

            obligations.append(&mut res.entities);
            if let Some(q) = res.into_next_query() {
                query = q;
            } else {
                break;
            }
        }
        obligations.sort();

        let mut remaining = payment.amount;
        let mut new_allocations = Vec::new();

        for obligation in obligations.iter_mut() {
            if let Idempotent::Executed(new_allocation) =
                obligation.allocate_payment(remaining, payment)
            {
                self.obligation_repo.update_in_op(db, obligation).await?;
                remaining -= new_allocation.amount;
                new_allocations.push(new_allocation);
                if remaining == UsdCents::ZERO {
                    break;
                }
            }
        }

        let allocations = self
            .payment_allocation_repo
            .create_all_in_op(db, new_allocations)
            .await?;
        self.ledger
            .record_payment_allocations(db, allocations, LedgerTransactionInitiator::System)
            .await?;

        Ok(())
    }

    async fn complete_facility_liquidation(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(db, credit_facility_id)
            .await?;

        if credit_facility
            .complete_liquidation(self.config.liquidation_id)?
            .did_execute()
        {
            self.credit_facility_repo
                .update_in_op(db, &mut credit_facility)
                .await?;
        }

        Ok(())
    }

    async fn complete_liquidation(
        &self,
        db: &mut DbOp<'_>,
        payment_id: PaymentId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut liquidation = self
            .liquidation_repo
            .find_by_id(self.config.liquidation_id)
            .await?;

        if liquidation.complete(payment_id).did_execute() {
            self.liquidation_repo
                .update_in_op(db, &mut liquidation)
                .await?;
        }

        Ok(())
    }

    #[instrument(name = "outbox.core_credit.partial_liquidation.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        clock: &es_entity::clock::ClockHandle,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
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
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("payment_id", tracing::field::display(payment_id));

                let payment_ledger_account_ids = PaymentLedgerAccountIds {
                    facility_payment_holding_account_id: *facility_payment_holding_account_id,
                    facility_uncovered_outstanding_account_id:
                        *facility_uncovered_outstanding_account_id,
                    payment_source_account_id: facility_proceeds_from_liquidation_account_id.into(),
                };

                let payment_opt = self
                    .record_payment(
                        db,
                        *payment_id,
                        *amount,
                        *credit_facility_id,
                        payment_ledger_account_ids,
                        clock,
                    )
                    .await?;

                if let Some(payment) = payment_opt {
                    self.allocate_payment(db, &payment, *credit_facility_id)
                        .await?;
                    self.complete_facility_liquidation(db, *credit_facility_id)
                        .await?;
                    self.complete_liquidation(db, *payment_id).await?;
                }

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

pub type PartialLiquidationJobSpawner<E> = JobSpawner<PartialLiquidationJobConfig<E>>;
