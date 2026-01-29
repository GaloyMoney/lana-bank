mod entity;
pub mod error;
mod jobs;
pub mod ledger;
mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use es_entity::clock::ClockHandle;

use crate::primitives::{CoreCreditAction, CoreCreditObject};
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
pub(crate) use repo::CollateralRepo;

/// Result of sending collateral to liquidation, containing all data needed for ledger posting.
#[derive(Debug, Clone)]
pub struct SendCollateralToLiquidationResult {
    pub ledger_tx_id: LedgerTxId,
    pub amount: Satoshis,
    pub collateral_account_id: CalaAccountId,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub collateral_update: CollateralUpdate,
}

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
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
            clock: jobs.clock().clone(),
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
        account_ids: crate::CollateralLedgerAccountIds,
    ) -> Result<Collateral, CollateralError> {
        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .credit_facility_id(pending_credit_facility_id)
            .pending_credit_facility_id(pending_credit_facility_id)
            .account_id(account_ids.collateral_account_id)
            .account_ids(account_ids)
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

    /// Sends collateral to the active liquidation with authorization check.
    /// Updates both the nested Liquidation entity and the Collateral's own amount,
    /// then posts to ledger. Returns the updated Collateral entity.
    #[record_error_severity]
    #[instrument(name = "collateral.send_collateral_to_liquidation", skip(self, sub))]
    pub async fn send_collateral_to_liquidation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
        amount: Satoshis,
    ) -> Result<Collateral, CollateralError>
    where
        <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
        <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;
        let active_liquidation = collateral
            .active_liquidation()
            .ok_or(CollateralError::NoActiveLiquidation)?;
        let liquidation_id = active_liquidation.id;

        self.authz
            .evaluate_permission(
                sub,
                CoreCreditObject::liquidation(liquidation_id),
                CoreCreditAction::LIQUIDATION_RECORD_COLLATERAL_SENT,
                true,
            )
            .await?
            .expect("audit info missing");

        let mut db = self.repo.begin_op().await?;
        let effective = self.clock.today();
        let initiated_by = LedgerTransactionInitiator::try_from_subject(sub)?;

        let data = if let es_entity::Idempotent::Executed(data) =
            collateral.send_collateral_to_liquidation(liquidation_id, amount, effective)?
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;
            data
        } else {
            return Ok(collateral);
        };

        self.ledger
            .record_collateral_sent_to_liquidation(
                &mut db,
                data.ledger_tx_id,
                amount,
                data.collateral_account_id,
                data.collateral_in_liquidation_account_id,
                initiated_by,
            )
            .await?;

        db.commit().await?;

        Ok(collateral)
    }

    /// Records proceeds received from liquidation with authorization check.
    /// Updates the nested Liquidation entity and posts to ledger.
    /// Returns the updated Collateral entity.
    #[record_error_severity]
    #[instrument(
        name = "collateral.record_liquidation_proceeds_received",
        skip(self, sub)
    )]
    pub async fn record_liquidation_proceeds_received(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
        amount_received: UsdCents,
    ) -> Result<Collateral, CollateralError>
    where
        <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
        <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;
        let active_liquidation = collateral
            .active_liquidation()
            .ok_or(CollateralError::NoActiveLiquidation)?;
        let liquidation_id = active_liquidation.id;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::liquidation(liquidation_id),
                CoreCreditAction::LIQUIDATION_RECORD_PAYMENT_RECEIVED,
            )
            .await?;

        let mut db = self.repo.begin_op().await?;
        let initiated_by = LedgerTransactionInitiator::try_from_subject(sub)?;

        let tx_id = LedgerTxId::new();
        let payment_id = PaymentId::new();

        let data = if let es_entity::Idempotent::Executed(data) =
            collateral.record_liquidation_proceeds_received(amount_received, payment_id, tx_id)?
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;
            data
        } else {
            return Ok(collateral);
        };

        self.ledger
            .record_proceeds_from_liquidation(&mut db, tx_id, data, initiated_by)
            .await?;

        db.commit().await?;

        Ok(collateral)
    }

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
        let result = self
            .repo
            .list_for_credit_facility_id_by_created_at(
                credit_facility_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?;

        result
            .entities
            .into_iter()
            .next()
            .ok_or_else(|| CollateralError::EsEntityError(es_entity::EsEntityError::NotFound))
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
