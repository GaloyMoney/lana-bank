use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use crate::{credit_facility::CreditFacilities, event::CoreCreditEvent, primitives::*};

#[derive(Serialize, Deserialize)]
pub struct PermanentCreditFacilityCollateralizationFromEventsJobConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for PermanentCreditFacilityCollateralizationFromEventsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = PermanentCreditFacilityCollateralizationFromEventsInit<Perms, E>;
}

pub struct PermanentCreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    credit_facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> PermanentCreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(outbox: &Outbox<E>, credit_facilities: &CreditFacilities<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            credit_facilities: credit_facilities.clone(),
        }
    }
}

const CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("permanent-credit-facility-collateralization-from-events");

impl<Perms, E> JobInitializer for PermanentCreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            PermanentCreditFacilityCollateralizationFromEventsRunner::<Perms, E> {
                outbox: self.outbox.clone(),
                credit_facilities: self.credit_facilities.clone(),
            },
        ))
    }
}

// TODO: reproduce 'collateralization_ratio' test from old credit facility

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct PermanentCreditFacilityCollateralizationFromEventsData {
    sequence: EventSequence,
}

pub struct PermanentCreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    credit_facilities: CreditFacilities<Perms, E>,
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for PermanentCreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PermanentCreditFacilityCollateralizationFromEventsData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.as_ref().as_event() {
                Some(CoreCreditEvent::FacilityCollateralUpdated {
                    credit_facility_id: id,
                    ..
                })
                | Some(CoreCreditEvent::ObligationCreated {
                    credit_facility_id: id,
                    ..
                })
                | Some(CoreCreditEvent::FacilityRepaymentRecorded {
                    credit_facility_id: id,
                    ..
                }) => {
                    self.credit_facilities
                        .update_collateralization_from_events(*id, CVLPct::UPGRADE_BUFFER)
                        .await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => (),
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
