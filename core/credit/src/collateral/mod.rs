mod entity;
pub mod error;
mod jobs;
pub mod ledger;
mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CreditFacilityPublisher, event::CoreCreditEvent, liquidation::NewLiquidation, primitives::*,
};

use ledger::CollateralLedger;

pub use entity::Collateral;
pub(super) use entity::*;
use jobs::wallet_collateral_sync;

#[cfg(feature = "json-schema")]
pub use entity::CollateralEvent;
use error::CollateralError;
use repo::CollateralRepo;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
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
        ledger: Arc<CollateralLedger>,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
    ) -> Result<Self, CollateralError> {
        let repo_arc = Arc::new(CollateralRepo::new(pool, publisher, jobs.clock().clone()));

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
    #[instrument(
        name = "collateral.record_collateral_update_via_manual_input_in_op",
        skip(db, self)
    )]
    pub(super) async fn record_collateral_update_via_manual_input_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        if collateral.custody_wallet_id.is_some() {
            return Err(CollateralError::ManualUpdateError);
        }

        let res = if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_manual_input(updated_collateral, effective)
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.record_collateral_update_via_liquidation_in_op",
        skip(db, self)
    )]
    pub(super) async fn record_collateral_update_via_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        liquidation_id: LiquidationId,
        amount_sent: core_money::Satoshis,
        effective: chrono::NaiveDate,
        collateral_in_liquidation_account_id: CalaAccountId,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id_in_op(&mut *db, collateral_id).await?;

        let res = if let es_entity::Idempotent::Executed(data) = collateral
            .record_collateral_update_via_liquidation(liquidation_id, amount_sent, effective)?
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            self.ledger
                .record_collateral_sent_to_liquidation_in_op(
                    db,
                    data.tx_id,
                    amount_sent,
                    collateral.account_id,
                    collateral_in_liquidation_account_id,
                    initiated_by,
                )
                .await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }

    /// Creates a new liquidation for the given collateral via the Collateral aggregate.
    /// This enforces invariants like "no active liquidation already in progress".
    /// Creates a new liquidation for the given collateral via the Collateral aggregate.
    /// This enforces invariants like "no active liquidation already in progress".
    #[record_error_severity]
    #[instrument(name = "collateral.initiate_liquidation_in_op", skip(db, self))]
    pub(super) async fn initiate_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        new_liquidation: NewLiquidation,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Result<Collateral, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        if collateral
            .initiate_liquidation(
                new_liquidation,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
            )?
            .did_execute()
        {
            self.repo.update_in_op(db, &mut collateral).await?;
        }

        Ok(collateral)
    }

    /// Completes a liquidation via the Collateral aggregate.
    #[record_error_severity]
    #[instrument(name = "collateral.complete_liquidation_in_op", skip(db, self))]
    pub(super) async fn complete_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        liquidation_id: LiquidationId,
    ) -> Result<(), CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        if collateral
            .complete_liquidation(liquidation_id)?
            .did_execute()
        {
            self.repo.update_in_op(db, &mut collateral).await?;
        }

        Ok(())
    }

    /// Finds a Collateral by its credit_facility_id.
    pub async fn find_by_credit_facility_id(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Collateral, CollateralError> {
        // The credit_facility_id is stored in the Collateral entity
        // We need a custom query for this
        self.repo
            .find_by_credit_facility_id(credit_facility_id)
            .await
    }

    /// Checks if a collateral has an active liquidation.
    pub async fn has_active_liquidation(
        &self,
        collateral_id: CollateralId,
    ) -> Result<bool, CollateralError> {
        let collateral = self.repo.find_by_id(collateral_id).await?;
        Ok(collateral.has_active_liquidation())
    }

    /// Finds a Collateral by its ID.
    pub async fn find_by_id(
        &self,
        collateral_id: CollateralId,
    ) -> Result<Collateral, CollateralError> {
        self.repo.find_by_id(collateral_id).await
    }
}
