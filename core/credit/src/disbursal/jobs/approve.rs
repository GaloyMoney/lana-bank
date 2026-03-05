use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_credit_collateral::{
    CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
};
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{
    ApprovalProcessId, ApprovalProcessType, GovernanceAction, GovernanceEvent, GovernanceObject,
};
use job::*;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};
use tracing_macros::observe_error;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
    credit_facility::CreditFacilities, disbursal::Disbursals, ledger::CreditLedger,
    primitives::DisbursalId,
};

pub const DISBURSAL_APPROVE_JOB: JobType = JobType::new("outbox.disbursal-approval");

pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

pub const APPROVE_DISBURSAL_COMMAND: JobType = JobType::new("command.credit.approve-disbursal");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveDisbursalConfig {
    pub approval_process_id: ApprovalProcessId,
    pub approved: bool,
}

pub struct ApproveDisbursalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    disbursals: Arc<Disbursals<Perms, E>>,
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
    ledger: Arc<CreditLedger>,
    clock: es_entity::clock::ClockHandle,
}

impl<Perms, E> ApproveDisbursalJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        disbursals: Arc<Disbursals<Perms, E>>,
        credit_facilities: Arc<CreditFacilities<Perms, E>>,
        ledger: Arc<CreditLedger>,
        clock: es_entity::clock::ClockHandle,
    ) -> Self {
        Self {
            disbursals,
            credit_facilities,
            ledger,
            clock,
        }
    }
}

impl<Perms, E> JobInitializer for ApproveDisbursalJobInitializer<Perms, E>
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
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = ApproveDisbursalConfig;

    fn job_type(&self) -> JobType {
        APPROVE_DISBURSAL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ApproveDisbursalJobRunner {
            config: job.config()?,
            disbursals: self.disbursals.clone(),
            credit_facilities: self.credit_facilities.clone(),
            ledger: self.ledger.clone(),
            clock: self.clock.clone(),
        }))
    }
}

struct ApproveDisbursalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: ApproveDisbursalConfig,
    disbursals: Arc<Disbursals<Perms, E>>,
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
    ledger: Arc<CreditLedger>,
    clock: es_entity::clock::ClockHandle,
}

#[async_trait]
impl<Perms, E> JobRunner for ApproveDisbursalJobRunner<Perms, E>
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
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[observe_error(allow_single_error_alert)]
    #[tracing::instrument(name = "credit.approve_disbursal.process_command", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        let id: DisbursalId = self.config.approval_process_id.into();

        match self
            .disbursals
            .conclude_approval_process_in_op(
                &mut op,
                id,
                self.config.approved,
                self.clock.now().date_naive(),
            )
            .await?
        {
            crate::ApprovalProcessOutcome::AlreadyApplied(_disbursal) => {
                tracing::Span::current().record("already_applied", true);
            }
            crate::ApprovalProcessOutcome::Approved((disbursal, obligation)) => {
                tracing::Span::current().record("already_applied", false);

                let credit_facility = self
                    .credit_facilities
                    .find_by_id_without_audit_in_op(&mut op, disbursal.facility_id)
                    .await?;
                self.ledger
                    .settle_disbursal_in_op(
                        &mut op,
                        disbursal.id,
                        disbursal.disbursal_credit_account_id,
                        obligation,
                        credit_facility.account_ids,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::DISBURSAL_APPROVAL,
                        ),
                    )
                    .await?;
            }
            crate::ApprovalProcessOutcome::Denied(disbursal) => {
                tracing::Span::current().record("already_applied", false);
                let credit_facility = self
                    .credit_facilities
                    .find_by_id_without_audit_in_op(&mut op, disbursal.facility_id)
                    .await?;
                self.ledger
                    .cancel_disbursal_in_op(
                        &mut op,
                        disbursal.id,
                        disbursal.initiated_tx_id,
                        disbursal.amount,
                        credit_facility.account_ids,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::DISBURSAL_APPROVAL,
                        ),
                    )
                    .await?;
            }
        };

        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub struct DisbursalApprovalHandler {
    approve_disbursal: JobSpawner<ApproveDisbursalConfig>,
}

impl DisbursalApprovalHandler {
    pub fn new(approve_disbursal: JobSpawner<ApproveDisbursalConfig>) -> Self {
        Self { approve_disbursal }
    }
}

impl<E> OutboxEventHandler<E> for DisbursalApprovalHandler
where
    E: OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "core_credit.disbursal_approval_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ GovernanceEvent::ApprovalProcessConcluded { entity }) = event.as_event()
            && entity.process_type == APPROVE_DISBURSAL_PROCESS
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            Span::current().record("process_type", entity.process_type.to_string());
            self.approve_disbursal
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    ApproveDisbursalConfig {
                        approval_process_id: entity.id,
                        approved: entity.status.is_approved(),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
