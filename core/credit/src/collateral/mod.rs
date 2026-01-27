mod entity;
pub mod error;
mod jobs;
mod liquidation;
mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use core_accounting::LedgerTransactionInitiator;
use es_entity::Idempotent;
use job::ClockHandle;
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::*;
use authz::PermissionCheck;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{CreditFacilityPublisher, CreditLedger, event::CoreCreditEvent, primitives::*};
use cala_ledger::primitives::TransactionId as CalaTransactionId;

pub use entity::Collateral;
pub(super) use entity::*;
use jobs::wallet_collateral_sync;

#[cfg(feature = "json-schema")]
pub use entity::CollateralEvent;
use error::CollateralError;
pub use repo::CollateralRepo;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CreditLedger>,
    clock: ClockHandle,
}

impl<Perms, E> Clone for Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<Perms, E> Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<core_custody::CoreCustodyEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        publisher: &CreditFacilityPublisher<E>,
        ledger: Arc<CreditLedger>,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
        clock: ClockHandle,
    ) -> Result<Self, CollateralError> {
        let repo_arc = Arc::new(CollateralRepo::new(pool, publisher));

        let wallet_collateral_sync_job_spawner =
            jobs.add_initializer(wallet_collateral_sync::WalletCollateralSyncInit::new(
                outbox,
                ledger.clone(),
                repo_arc.clone(),
            ));

        wallet_collateral_sync_job_spawner
            .spawn_unique(
                job::JobId::new(),
                wallet_collateral_sync::WalletCollateralSyncJobConfig::new(),
            )
            .await?;

        Ok(Self {
            authz,
            repo: repo_arc,
            ledger,
            clock,
        })
    }

    pub async fn find_all<T: From<Collateral>>(
        &self,
        ids: &[CollateralId],
    ) -> Result<HashMap<CollateralId, T>, CollateralError> {
        self.repo.find_all(ids).await
    }

    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        pending_credit_facility_id: PendingCreditFacilityId,
        custody_wallet_id: Option<CustodyWalletId>,
        account_id: CalaAccountId,
    ) -> Result<Collateral, CollateralError> {
        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .credit_facility_id(pending_credit_facility_id)
            .pending_credit_facility_id(pending_credit_facility_id)
            .account_id(account_id)
            .custody_wallet_id(custody_wallet_id)
            .build()
            .expect("all fields for new collateral provided");

        self.repo.create_in_op(db, new_collateral).await
    }

    #[record_error_severity]
    #[instrument(name = "credit.colateral.send_collateral_to_liquidation", skip(self))]
    pub async fn send_collateral_to_liquidation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: impl Into<CollateralId> + std::fmt::Debug + Copy,
        amount_sent: Satoshis,
    ) -> Result<Collateral, CollateralError> {
        let collateral_id = collateral_id.into();
        let collateral = self.repo.find_by_id(collateral_id).await?;

        let updated_collateral = collateral.amount - amount_sent;

        self.update_collateral(sub, collateral_id, updated_collateral, self.clock.today())
            .await
    }

    #[record_error_severity]
    #[instrument(name = "credit.collateral.update_collateral", skip(self))]
    pub async fn update_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: impl Into<CollateralId> + std::fmt::Debug + Copy,
        updated_collateral: Satoshis,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<Collateral, CollateralError> {
        let collateral_id = collateral_id.into();
        let effective = effective.into();

        // self.subject_can_update_collateral(sub, true)
        // .await?
        // .expect("audit info missing");

        let mut db = self.repo.begin_op_with_clock(&self.clock).await?;

        let mut collateral = self.repo.find_by_id_in_op(&mut db, collateral_id).await?;

        if collateral.custody_wallet_id.is_some() {
            return Err(CollateralError::ManualUpdateError);
        }

        let collateral_update = if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_manual_input(updated_collateral, effective)
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;
            data
        } else {
            return Ok(collateral);
        };

        self.ledger
            .update_credit_facility_collateral(
                &mut db,
                collateral_update,
                collateral.collateral_account_id,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;

        db.commit().await?;

        Ok(collateral)
    }

    pub async fn record_proceeds_from_liquidation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
        amount_received: UsdCents,
    ) -> Result<(), CollateralError> {
        // self.authz
        //     .enforce_permission(
        //         sub,
        //         CoreCreditObject::liquidation(liquidation_id),
        //         CoreCreditAction::LIQUIDATION_RECORD_PAYMENT_RECEIVED,
        //     )
        //     .await?;

        let mut db = self.repo.begin_op_with_clock(&self.clock).await?;

        let mut collateral = self.repo.find_by_id_in_op(&mut db, collateral_id).await?;

        let tx_id = CalaTransactionId::new();

        if let Idempotent::Executed(data) =
            collateral.record_proceeds_from_liquidation(amount_received, PaymentId::new(), tx_id)
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;
            self.ledger
                .record_proceeds_from_liquidation_in_op(
                    &mut db,
                    tx_id,
                    data,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;
        }

        db.commit().await?;

        Ok(())
    }

    pub async fn empty_up_collateral_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        effective: chrono::NaiveDate,
    ) -> Result<(), CollateralError> {
        todo!()
    }
}
