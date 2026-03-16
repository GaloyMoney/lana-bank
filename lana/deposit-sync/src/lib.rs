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
use core_time_events::CoreTimeEvent;
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};
use sumsub::SumsubClient;
use tracing_macros::record_error_severity;

pub struct DepositSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreTimeEvent>
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
        + OutboxEventMarker<CoreTimeEvent>
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
        + OutboxEventMarker<CoreTimeEvent>
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
        let classify_spawner =
            jobs.add_initializer(ClassifyDepositAccountActivityJobInit::new(deposits));

        let sweep_spawner = jobs.add_initializer(SweepDepositActivityStatusJobInit::new(
            deposits,
            classify_spawner,
        ));

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(DEPOSIT_END_OF_DAY),
                DepositEndOfDayHandler::new(sweep_spawner),
            )
            .await?;

        let export_sumsub_deposit_spawner = jobs.add_initializer(
            ExportSumsubDepositJobInitializer::new(sumsub_client.clone(), deposits, customers),
        );

        let export_sumsub_withdrawal_spawner = jobs.add_initializer(
            ExportSumsubWithdrawalJobInitializer::new(sumsub_client, deposits, customers),
        );

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(SUMSUB_EXPORT_JOB),
                SumsubExportHandler::new(
                    export_sumsub_deposit_spawner,
                    export_sumsub_withdrawal_spawner,
                ),
            )
            .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
