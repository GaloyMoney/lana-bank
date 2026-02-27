use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::SystemSubject;
use command_job::AtomicCommandJob;
use core_custody::{CUSTODIAN_SYNC, CoreCustodyEvent, WalletId as CustodyWalletId};
use tracing_macros::record_error_severity;

use crate::{ledger::CollateralLedger, public::CoreCreditCollateralEvent, repo::CollateralRepo};

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordCollateralUpdateCommand {
    pub custody_wallet_id: CustodyWalletId,
    pub updated_collateral: money::Satoshis,
    pub effective: chrono::NaiveDate,
}

pub const RECORD_COLLATERAL_UPDATE_COMMAND: JobType =
    JobType::new("command.core-credit.record-collateral-update");

pub struct RecordCollateralUpdateCommandJob<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, E> RecordCollateralUpdateCommandJob<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(ledger: Arc<CollateralLedger>, repo: Arc<CollateralRepo<E>>) -> Self {
        Self {
            repo,
            ledger,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, E> AtomicCommandJob for RecordCollateralUpdateCommandJob<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    type Command = RecordCollateralUpdateCommand;

    fn job_type() -> JobType {
        RECORD_COLLATERAL_UPDATE_COMMAND
    }

    fn queue_id(command: &Self::Command) -> String {
        command.custody_wallet_id.to_string()
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "core_credit.record_collateral_update_job.process_command",
        skip(self, op, command),
        fields(
            custody_wallet_id = %command.custody_wallet_id,
            updated_collateral = %command.updated_collateral,
            effective = %command.effective,
        ),
    )]
    async fn run(
        &self,
        op: &mut es_entity::DbOp<'static>,
        command: &Self::Command,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut collateral = self
            .repo
            .find_by_custody_wallet_id_in_op(op, Some(command.custody_wallet_id))
            .await?;

        if let es_entity::Idempotent::Executed(data) = collateral
            .record_collateral_update_via_custodian_sync(
                command.updated_collateral,
                command.effective,
            )
        {
            self.repo.update_in_op(op, &mut collateral).await?;

            self.ledger
                .update_collateral_amount_in_op(op, data, &S::system(CUSTODIAN_SYNC))
                .await?;
        }

        Ok(())
    }
}
