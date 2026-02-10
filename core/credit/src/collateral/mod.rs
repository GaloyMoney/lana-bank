mod entity;
pub mod error;
mod jobs;
pub mod ledger;
pub mod liquidation;
mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use core_custody::CoreCustodyEvent;
use es_entity::clock::ClockHandle;
use governance::GovernanceEvent;
use money::UsdCents;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    collateral::ledger::CollateralLedgerAccountIds,
    primitives::{CoreCreditAction, CoreCreditCollectionEvent, CoreCreditObject},
};

use es_entity::Idempotent;

use crate::{event::CoreCreditEvent, primitives::*, publisher::CreditFacilityPublisher};

use ledger::CollateralLedger;

pub(super) use entity::*;
use jobs::{collateral_liquidations, liquidation_payment, wallet_collateral_sync};
pub use {
    entity::Collateral,
    liquidation::{Liquidation, LiquidationError, RecordProceedsFromLiquidationData},
    repo::liquidation_cursor,
};

#[cfg(feature = "json-schema")]
pub use entity::CollateralEvent;
use error::CollateralError;
#[cfg(feature = "json-schema")]
pub use liquidation::LiquidationEvent;
use repo::CollateralRepo;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
    clock: ClockHandle,
}

impl<Perms, E> Clone for Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        publisher: &CreditFacilityPublisher<E>,
        ledger: Arc<CollateralLedger>,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
        collections: Arc<core_credit_collection::CoreCreditCollection<Perms, E>>,
    ) -> Result<Self, CollateralError> {
        let clock = jobs.clock().clone();
        let repo_arc = Arc::new(CollateralRepo::new(pool, publisher, clock.clone()));

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

        let credit_facility_repo = Arc::new(crate::credit_facility::CreditFacilityRepo::new(
            pool,
            publisher,
            clock.clone(),
        ));

        let liquidation_payment_job_spawner =
            jobs.add_initializer(liquidation_payment::LiquidationPaymentInit::new(
                outbox,
                collections,
                credit_facility_repo,
            ));

        let collateral_liquidations_job_spawner = jobs.add_initializer(
            collateral_liquidations::CreditFacilityLiquidationsInit::new(
                outbox,
                repo_arc.clone(),
                liquidation_payment_job_spawner,
            ),
        );

        collateral_liquidations_job_spawner
            .spawn_unique(
                job::JobId::new(),
                collateral_liquidations::CreditFacilityLiquidationsJobConfig,
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

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CollateralError> {
        Ok(self.repo.begin_op().await?)
    }

    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        pending_credit_facility_id: PendingCreditFacilityId,
        custody_wallet_id: Option<CustodyWalletId>,
        account_ids: CollateralLedgerAccountIds,
    ) -> Result<Collateral, CollateralError> {
        self.ledger
            .create_collateral_accounts_in_op(db, collateral_id, account_ids)
            .await?;

        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .credit_facility_id(pending_credit_facility_id)
            .pending_credit_facility_id(pending_credit_facility_id)
            .custody_wallet_id(custody_wallet_id)
            .account_ids(account_ids)
            .build()
            .expect("all fields for new collateral provided");

        self.repo.create_in_op(db, new_collateral).await
    }

    pub async fn subject_can_update_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CollateralError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_collaterals(),
                CoreCreditAction::COLLATERAL_RECORD_MANUAL_UPDATE,
                enforce,
            )
            .await?)
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
        updated_collateral: money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        let res = if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_manual_input(updated_collateral, effective)?
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }

    #[record_error_severity]
    #[instrument(name = "collateral.update_collateral_by_id", skip(self, sub), err)]
    pub async fn update_collateral_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
        updated_collateral: money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<Collateral, CollateralError> {
        self.subject_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let initiated_by = LedgerTransactionInitiator::try_from_subject(sub)?;

        let mut db = self.repo.begin_op().await?;

        let mut collateral = self.repo.find_by_id_in_op(&mut db, collateral_id).await?;

        if let es_entity::Idempotent::Executed(collateral_update) =
            collateral.record_collateral_update_via_manual_input(updated_collateral, effective)?
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .update_collateral_amount_in_op(
                    &mut db,
                    collateral_update,
                    collateral.account_ids.collateral_account_id,
                    initiated_by,
                )
                .await?;
        }

        db.commit().await?;

        Ok(collateral)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.record_collateral_update_via_liquidation",
        skip(self, sub),
        err
    )]
    pub async fn record_collateral_update_via_liquidation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
        amount_sent: money::Satoshis,
    ) -> Result<Collateral, CollateralError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::collateral(collateral_id),
                CoreCreditAction::COLLATERAL_RECORD_LIQUIDATION_UPDATE,
            )
            .await?;

        let initiated_by = LedgerTransactionInitiator::try_from_subject(sub)?;
        let effective = self.clock.today();

        let mut db = self.repo.begin_op().await?;

        let mut collateral = self.repo.find_by_id_in_op(&mut db, collateral_id).await?;

        if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_liquidation(amount_sent, effective)?
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .record_collateral_sent_to_liquidation_in_op(
                    &mut db,
                    data.tx_id,
                    amount_sent,
                    collateral.account_ids.collateral_account_id,
                    collateral.account_ids.collateral_in_liquidation_account_id,
                    initiated_by,
                )
                .await?;
        }

        db.commit().await?;

        Ok(collateral)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.record_proceeds_received_and_liquidation_completed",
        skip(self, sub),
        err
    )]
    pub async fn record_proceeds_received_and_liquidation_completed(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
        amount_received: UsdCents,
    ) -> Result<Collateral, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::collateral(collateral_id),
                CoreCreditAction::COLLATERAL_RECORD_PAYMENT_RECEIVED_FROM_LIQUIDATION,
            )
            .await?;

        let mut db = self.repo.begin_op().await?;

        if let Idempotent::Executed(data) =
            collateral.record_proceeds_received_and_liquidation_completed(amount_received)?
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;
            self.ledger
                .record_proceeds_from_liquidation_in_op(
                    &mut db,
                    data,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;
        }

        db.commit().await?;

        Ok(collateral)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.list_liquidations_for_collateral_by_created_at",
        skip(self, sub)
    )]
    pub async fn list_liquidations_for_collateral_by_created_at(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateral_id: CollateralId,
    ) -> Result<Vec<Liquidation>, LiquidationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_liquidations(),
                CoreCreditAction::LIQUIDATION_LIST,
            )
            .await?;

        self.repo
            .list_liquidations_for_collateral_id(collateral_id)
            .await
    }

    #[record_error_severity]
    #[instrument(name = "collateral.find_liquidation_by_id", skip(self, sub))]
    pub async fn find_liquidation_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        liquidation_id: impl Into<LiquidationId> + std::fmt::Debug,
    ) -> Result<Option<Liquidation>, LiquidationError> {
        let id = liquidation_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::liquidation(id),
                CoreCreditAction::LIQUIDATION_READ,
            )
            .await?;

        self.repo.find_liquidation_by_id(id).await
    }

    pub async fn find_all_liquidations<T: From<Liquidation>>(
        &self,
        ids: &[LiquidationId],
    ) -> Result<HashMap<LiquidationId, T>, LiquidationError> {
        self.repo.find_all_liquidations(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "collateral.list_liquidations", skip(self, sub))]
    pub async fn list_liquidations(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<liquidation_cursor::LiquidationsByIdCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Liquidation, liquidation_cursor::LiquidationsByIdCursor>,
        LiquidationError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_liquidations(),
                CoreCreditAction::LIQUIDATION_LIST,
            )
            .await?;

        self.repo.list_liquidations(query).await
    }
}
