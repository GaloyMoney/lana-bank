//! Credit Facility Health listens to changes in collateralization
//! state of credit facilities and initiates a partial liquidation of
//! credit facility whose CVL drops below liquidation threshold
//! (i. e. became unhealthy), unless this credit facility is already
//! in an active liquidation.
//!
//! All other state changes are ignored by this job.

use async_trait::async_trait;
use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use futures::StreamExt as _;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker, PersistentOutboxEvent};
use serde::{Deserialize, Serialize};

use crate::jobs::{partial_liquidation, partial_liquidation_cala};
use crate::{
    CollateralizationState, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilities,
    liquidation_process::LiquidationProcessRepo,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityHealthJobData {
    sequence: EventSequence,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CreditFacilityHealthJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for CreditFacilityHealthJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = CreditFacilityHealthInit<Perms, E>;
}

pub struct CreditFacilityHealthInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
    facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> CreditFacilityHealthInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        jobs: &Jobs,
        facilities: &CreditFacilities<Perms, E>,
        liquidation_process_repo: &LiquidationProcessRepo<E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            jobs: jobs.clone(),
            facilities: facilities.clone(),
            liquidation_process_repo: liquidation_process_repo.clone(),
        }
    }
}

const CREDIT_FACILITY_HEALTH_JOB: JobType = JobType::new("outbox.credit-facility-health");
impl<Perms, E> JobInitializer for CreditFacilityHealthInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_HEALTH_JOB
    }

    fn init(&self, job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityHealthJobRunner::<Perms, E> {
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            liquidation_process_repo: self.liquidation_process_repo.clone(),
            facilities: self.facilities.clone(),
        }))
    }
}

pub struct CreditFacilityHealthJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
    facilities: CreditFacilities<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityHealthJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityHealthJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let mut db = self.liquidation_process_repo.begin().await?;
            self.process_message(message.as_ref(), &mut db).await?;
            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_op(&mut db, &state)
                .await?;

            db.commit().await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> CreditFacilityHealthJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        if let Some(event) = message.as_event() {
            match event {
                FacilityCollateralizationChanged {
                    id,
                    state: CollateralizationState::UnderLiquidationThreshold,
                    price,
                    ..
                } => {
                    match self
                        .liquidation_process_repo
                        .find_by_credit_facility_id(*id)
                        .await
                    {
                        Err(e) if e.was_not_found() => {
                            let new_liquidation =
                                self.facilities.initiate_liquidation(*id, *price).await?;
                            let liquidation = self
                                .liquidation_process_repo
                                .create(new_liquidation)
                                .await?;
                            self.jobs
                                .create_and_spawn_in_op(
                                    db,
                                    JobId::new(),
                                    partial_liquidation::PartialLiquidationJobConfig::<E> {
                                        liquidation_process_id: liquidation.id,
                                        credit_facility_id: *id,
                                        _phantom: std::marker::PhantomData,
                                    },
                                )
                                .await?;
                            self.jobs
                                .create_and_spawn_in_op(
                                    db,
                                    JobId::new(),
                                    partial_liquidation_cala::PartialLiquidationCalaJobConfig::<E> {
                                        receivable_account_id: todo!(),
                                        liquidation_process_id: liquidation.id,
                                        _phantom: std::marker::PhantomData,
                                    },
                                )
                                .await?;
                        }
                        Err(e) => return Err(Box::new(e)),
                        Ok(_) => {
                            // liquidation process already running for this facility
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(())
    }
}
