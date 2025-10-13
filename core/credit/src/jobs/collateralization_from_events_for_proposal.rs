use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{
    event::CoreCreditEvent, pending_credit_facility::PendingCreditFacilities, primitives::*,
};

#[derive(Serialize, Deserialize)]
pub struct PendingCreditFacilityCollateralizationFromEventsJobConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for PendingCreditFacilityCollateralizationFromEventsJobConfig<Perms, E>
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
    type Initializer = PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>;
}

pub struct PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>
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
    credit_facility_proposals: PendingCreditFacilities<Perms, E>,
}

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>
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
        credit_facility_proposals: &PendingCreditFacilities<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            credit_facility_proposals: credit_facility_proposals.clone(),
        }
    }
}

const CREDIT_FACILITY_PROPOSAL_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("credit-facility-proposal-collateralization-from-events");
impl<Perms, E> JobInitializer for PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>
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
        CREDIT_FACILITY_PROPOSAL_COLLATERALIZATION_FROM_EVENTS_JOB
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            PendingCreditFacilityCollateralizationFromEventsRunner::<Perms, E> {
                outbox: self.outbox.clone(),
                pending_credit_facility: self.credit_facility_proposals.clone(),
            },
        ))
    }
}

// TODO: reproduce 'collateralization_ratio' test from old credit facility

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct PendingCreditFacilityCollateralizationFromEventsData {
    sequence: EventSequence,
}

pub struct PendingCreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    pending_credit_facility: PendingCreditFacilities<Perms, E>,
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for PendingCreditFacilityCollateralizationFromEventsRunner<Perms, E>
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
            .execution_state::<PendingCreditFacilityCollateralizationFromEventsData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCreditEvent::FacilityCollateralUpdated {
                pending_credit_facility_id: id,
                ..
            }) = message.as_ref().as_event()
            {
                self.pending_credit_facility
                    .update_collateralization_from_events(*id)
                    .await?;
                state.sequence = message.sequence;
                current_job.update_execution_state(state).await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
