use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use crate::{
    credit_facility_proposal::CreditFacilityProposals, event::CoreCreditEvent, primitives::*,
};

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityProposalCollateralizationFromEventsJobConfig<Perms, E> {
    pub upgrade_buffer_cvl_pct: CVLPct,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityProposalCollateralizationFromEventsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = CreditFacilityProposalCollateralizationFromEventsInit<Perms, E>;
}

pub struct CreditFacilityProposalCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    credit_facility_proposals: CreditFacilityProposals<Perms, E>,
}

impl<Perms, E> CreditFacilityProposalCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        credit_facility_proposals: &CreditFacilityProposals<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            credit_facility_proposals: credit_facility_proposals.clone(),
        }
    }
}

const CREDIT_FACILITY_PROPOSAL_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("credit-facility-proposal-collateralization-from-events");
impl<Perms, E> JobInitializer for CreditFacilityProposalCollateralizationFromEventsInit<Perms, E>
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
        CREDIT_FACILITY_PROPOSAL_COLLATERALIZATION_FROM_EVENTS_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            CreditFacilityProposalCollateralizationFromEventsRunner::<Perms, E> {
                config: job.config()?,
                outbox: self.outbox.clone(),
                credit_facility_proposals: self.credit_facility_proposals.clone(),
            },
        ))
    }
}

// TODO: reproduce 'collateralization_ratio' test from old credit facility

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct CreditFacilityProposalCollateralizationFromEventsData {
    sequence: EventSequence,
}

pub struct CreditFacilityProposalCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    config: CreditFacilityProposalCollateralizationFromEventsJobConfig<Perms, E>,
    outbox: Outbox<E>,
    credit_facility_proposals: CreditFacilityProposals<Perms, E>,
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for CreditFacilityProposalCollateralizationFromEventsRunner<Perms, E>
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
            .execution_state::<CreditFacilityProposalCollateralizationFromEventsData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.as_ref().as_event() {
                Some(CoreCreditEvent::FacilityCollateralUpdated {
                    credit_facility_id: id,
                    ..
                }) => {
                    self.credit_facility_proposals
                        .update_collateralization_from_events(
                            CreditFacilityProposalId::from(*id),
                            self.config.upgrade_buffer_cvl_pct,
                        )
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
