#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
mod job;

use error::*;
use job::*;

use ::job::Jobs;
use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};
use sumsub::SumsubClient;
use tracing_macros::record_error_severity;

use lana_events::LanaEvent;

pub struct DepositSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
    _outbox: Outbox<E>,
}

impl<Perms, E> Clone for DepositSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> DepositSync<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    #[record_error_severity]
    #[tracing::instrument(name = "deposit_sync.init", skip_all)]
    pub async fn init(
        jobs: &mut Jobs,
        outbox: &Outbox<E>,
        deposits: &CoreDeposit<Perms, E>,
        customers: &Customers<Perms, E>,
        sumsub_client: SumsubClient,
    ) -> Result<Self, DepositSyncError> {
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(SUMSUB_EXPORT_JOB),
                SumsubExportHandler::new(sumsub_client, deposits, customers),
            )
            .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
