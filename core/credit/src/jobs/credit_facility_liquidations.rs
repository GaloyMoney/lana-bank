use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use es_entity::DbOp;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::jobs::partial_liquidation;
use crate::liquidation::{Liquidations, NewLiquidation};
use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilities,
    CreditFacilityLedgerAccountIds,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityLiquidationsJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityLiquidationsJobConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for CreditFacilityLiquidationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = CreditFacilityLiquidationsInit<Perms, E>;
}

pub struct CreditFacilityLiquidationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidations: Liquidations<Perms, E>,
    facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> CreditFacilityLiquidationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(
        outbox: &Outbox<E>,
        jobs: &Jobs,
        liquidations: &Liquidations<Perms, E>,
        facilities: &CreditFacilities<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            jobs: jobs.clone(),
            liquidations: liquidations.clone(),
            facilities: facilities.clone(),
        }
    }
}

const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");
impl<Perms, E> JobInitializer for CreditFacilityLiquidationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_LIQUIDATIONS_JOB
    }

    fn init(&self, _job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityLiquidationsJobRunner::<Perms, E> {
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            liquidations: self.liquidations.clone(),
            facilities: self.facilities.clone(),
        }))
    }
}

pub struct CreditFacilityLiquidationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidations: Liquidations<Perms, E>,
    facilities: CreditFacilities<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityLiquidationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityLiquidationsJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CREDIT_FACILITY_LIQUIDATIONS_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {

                            let mut db = self.liquidations.begin_op().await?;
                            self.process_message(&mut db, message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;
                            db.commit().await?;
                        }
                        None => return Ok(JobCompletion::RescheduleNow)
                    }
                }
            }
        }
    }
}

impl<Perms, E> CreditFacilityLiquidationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    #[instrument(name = "outbox.core_credit.credit_facility_liquidations.process_message", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(
            event @ CoreCreditEvent::PartialLiquidationInitiated {
                liquidation_id,
                credit_facility_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
                ..
            },
        ) = message.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            let credit_facility = self
                .facilities
                .find_by_id_without_audit(*credit_facility_id)
                .await?;
            let CreditFacilityLedgerAccountIds {
                collateral_account_id,
                collateral_in_liquidation_account_id,
                liquidated_collateral_account_id,
                proceeds_from_liquidation_account_id,
                payment_holding_account_id,
                ..
            } = credit_facility.account_ids;

            let maybe_new_liqudation = self
                .liquidations
                .create_if_not_exist_for_facility_in_op(
                    db,
                    *credit_facility_id,
                    NewLiquidation::builder()
                        .id(*liquidation_id)
                        .credit_facility_id(*credit_facility_id)
                        .facility_liquidation_in_holding_account_id(
                            proceeds_from_liquidation_account_id,
                        )
                        .facility_payment_holding_account_id(payment_holding_account_id)
                        .collateral_account_id(collateral_account_id)
                        .collateral_in_liquidation_account_id(collateral_in_liquidation_account_id)
                        .liquidated_collateral_account_id(liquidated_collateral_account_id)
                        .trigger_price(*trigger_price)
                        .initially_expected_to_receive(*initially_expected_to_receive)
                        .initially_estimated_to_liquidate(*initially_estimated_to_liquidate),
                )
                .await?;

            if let Some(liquidation) = maybe_new_liqudation {
                self.jobs
                    .create_and_spawn_in_op(
                        db,
                        JobId::new(),
                        partial_liquidation::PartialLiquidationJobConfig::<Perms, E> {
                            liquidation_id: liquidation.id,
                            credit_facility_id: *credit_facility_id,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
