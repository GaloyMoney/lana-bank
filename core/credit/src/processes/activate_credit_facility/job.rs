use async_trait::async_trait;
use futures::StreamExt;
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId};

use super::ActivateCreditFacility;

#[derive(serde::Serialize)]
pub(crate) struct CreditFacilityActivationJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CreditFacilityActivationJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for CreditFacilityActivationJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = CreditFacilityActivationInit<Perms, E>;
}

pub(crate) struct CreditFacilityActivationInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    process: ActivateCreditFacility<Perms, E>,
}

impl<Perms, E> CreditFacilityActivationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(outbox: &Outbox<E>, process: &ActivateCreditFacility<Perms, E>) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CREDIT_FACILITY_ACTIVATE: JobType = JobType::new("credit-facility-activation");
impl<Perms, E> JobInitializer for CreditFacilityActivationInit<Perms, E>
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
        CREDIT_FACILITY_ACTIVATE
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityActivationJobRunner {
            outbox: self.outbox.clone(),
            process: self.process.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct CreditFacilityActivationJobData {
    sequence: outbox::EventSequence,
}

pub struct CreditFacilityActivationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    process: ActivateCreditFacility<Perms, E>,
}

impl<Perms, E> CreditFacilityActivationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "core_credit.credit_facility_activation_job.process_msg", parent = None, skip(self, message), fields(seq = ?message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(
                event @ FacilityCollateralUpdated {
                    credit_facility_id, ..
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.process.execute(*credit_facility_id).await?;
            }
            Some(event @ FacilityProposalApproved { id, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let facility_id: CreditFacilityId = (*id).into();
                self.process.execute(facility_id).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityActivationJobRunner<Perms, E>
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
            .execution_state::<CreditFacilityActivationJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            self.process_message(message.as_ref()).await?;
            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
