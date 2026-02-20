use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use audit::SystemSubject;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use core_custody::CoreCustodyEvent;

use crate::{
    CoreCreditEvent,
    collateral::{CollateralError, CollateralRepo, ledger::CollateralLedgerOps},
};

pub const WALLET_COLLATERAL_SYNC_JOB: JobType = JobType::new("outbox.wallet-collateral-sync");

pub struct WalletCollateralSyncHandler<S, E, CL>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
    CL: CollateralLedgerOps,
{
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CL>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, E, CL> WalletCollateralSyncHandler<S, E, CL>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCustodyEvent> + OutboxEventMarker<CoreCreditEvent>,
    CL: CollateralLedgerOps,
{
    pub fn new(ledger: Arc<CL>, repo: Arc<CollateralRepo<E>>) -> Self {
        Self {
            ledger,
            repo,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, E, CL> OutboxEventHandler<E> for WalletCollateralSyncHandler<S, E, CL>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
    CL: CollateralLedgerOps,
{
    #[instrument(name = "core_credit.wallet_collateral_sync_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[allow(clippy::single_match)]
        match event.as_event() {
            Some(e @ CoreCustodyEvent::WalletBalanceUpdated { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                let balance = entity
                    .balance
                    .as_ref()
                    .expect("WalletBalanceUpdated must have balance");

                self.record_collateral_update_via_custodian_sync(
                    entity.id,
                    balance.amount,
                    balance.updated_at.date_naive(),
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl<S, E, CL> WalletCollateralSyncHandler<S, E, CL>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
    CL: CollateralLedgerOps,
{
    #[record_error_severity]
    #[instrument(
        name = "collateral.record_collateral_update_via_custodian_sync",
        fields(updated_collateral = %updated_collateral, effective = %effective),
        skip(self),
    )]
    async fn record_collateral_update_via_custodian_sync(
        &self,
        custody_wallet_id: crate::primitives::CustodyWalletId,
        updated_collateral: money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<(), CollateralError> {
        let mut collateral = self
            .repo
            .find_by_custody_wallet_id(Some(custody_wallet_id))
            .await?;

        let mut db = self.repo.begin_op().await?;

        if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_custodian_sync(updated_collateral, effective)
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .update_collateral_amount_in_op(
                    &mut db,
                    data,
                    &S::system(crate::primitives::COLLATERALIZATION_SYNC),
                )
                .await?;
            db.commit().await?;
        }

        Ok(())
    }
}
