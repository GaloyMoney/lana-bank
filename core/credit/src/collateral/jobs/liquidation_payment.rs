use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use core_credit_collection::{CoreCreditCollection, PaymentLedgerAccountIds};

use crate::{
    CoreCreditCollectionEvent,
    credit_facility::CreditFacilityRepo,
    event::CoreCreditEvent,
    primitives::{CreditFacilityId, LiquidationId},
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

pub struct LiquidationPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    outbox: Outbox<E>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
}

impl<Perms, E> LiquidationPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        collections: Arc<CoreCreditCollection<Perms, E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            collections,
            credit_facility_repo,
        }
    }
}

const LIQUIDATION_PAYMENT_JOB: JobType = JobType::new("outbox.liquidation-payment");

impl<Perms, E> JobInitializer for LiquidationPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
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
            collections: self.collections.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
        }))
    }
}

pub struct LiquidationPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: LiquidationPaymentJobConfig<E>,
    outbox: Outbox<E>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
}

impl<Perms, E> LiquidationPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(
        name = "outbox.core_credit.partial_liquidation.acknowledge_payment_in_credit_facility_in_op",
        skip(self, db)
    )]
    async fn acknowledge_payment_in_credit_facility_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(db, self.config.credit_facility_id)
            .await?;

        if credit_facility
            .acknowledge_payment_from_liquidation(self.config.liquidation_id)?
            .did_execute()
        {
            self.credit_facility_repo
                .update_in_op(db, &mut credit_facility)
                .await?;
        }

        Ok(())
    }

    #[instrument(name = "outbox.core_credit.liquidation_payment.process_message_in_op", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message_in_op(
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

                self.collections
                    .payments()
                    .record_in_op(
                        db,
                        *payment_id,
                        (*credit_facility_id).into(),
                        payment_ledger_account_ids,
                        *amount,
                        clock.today(),
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(),
                    )
                    .await?;

                self.acknowledge_payment_in_credit_facility_in_op(db)
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
}

#[async_trait]
impl<Perms, E> JobRunner for LiquidationPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
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
                                .credit_facility_repo
                                .begin_op()
                                .await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message_in_op(&mut db, message.as_ref(), current_job.clock()).await?;

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
