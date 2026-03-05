use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{
    CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
};
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};
use tracing_macros::observe_error;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
    PendingCreditFacilityCollateralizationState, PendingCreditFacilityId,
    credit_facility::CreditFacilities,
};

pub const CREDIT_FACILITY_ACTIVATE: JobType = JobType::new("outbox.credit-facility-activation");

pub const ACTIVATE_CREDIT_FACILITY_COMMAND: JobType =
    JobType::new("command.credit.activate-credit-facility");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivateCreditFacilityConfig {
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

pub struct ActivateCreditFacilityJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
}

impl<Perms, E> ActivateCreditFacilityJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(credit_facilities: Arc<CreditFacilities<Perms, E>>) -> Self {
        Self { credit_facilities }
    }
}

impl<Perms, E> JobInitializer for ActivateCreditFacilityJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = ActivateCreditFacilityConfig;

    fn job_type(&self) -> JobType {
        ACTIVATE_CREDIT_FACILITY_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ActivateCreditFacilityJobRunner {
            config: job.config()?,
            credit_facilities: self.credit_facilities.clone(),
        }))
    }
}

pub struct ActivateCreditFacilityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: ActivateCreditFacilityConfig,
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
}

#[async_trait]
impl<Perms, E> JobRunner for ActivateCreditFacilityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[observe_error(allow_single_error_alert)]
    #[tracing::instrument(name = "credit.activate_credit_facility.process_command", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        self.credit_facilities
            .activate_in_op(&mut op, self.config.pending_credit_facility_id.into())
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub struct CreditFacilityActivationHandler {
    activate_credit_facility: JobSpawner<ActivateCreditFacilityConfig>,
}

impl CreditFacilityActivationHandler {
    pub fn new(activate_credit_facility: JobSpawner<ActivateCreditFacilityConfig>) -> Self {
        Self {
            activate_credit_facility,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityActivationHandler
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "core_credit.credit_facility_activation_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditEvent::*;

        if let Some(e @ PendingCreditFacilityCollateralizationChanged { entity }) = event.as_event()
            && entity.collateralization.state
                == PendingCreditFacilityCollateralizationState::FullyCollateralized
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record(
                "pending_credit_facility_id",
                tracing::field::display(entity.id),
            );
            Span::current().record("event_type", e.as_ref());

            self.activate_credit_facility
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ActivateCreditFacilityConfig {
                        pending_credit_facility_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
