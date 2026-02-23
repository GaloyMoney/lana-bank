mod entity;
pub mod error;
mod jobs;
pub mod ledger;
pub mod liquidation;
pub mod primitives;
pub mod public;
pub(crate) mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use core_custody::CoreCustodyEvent;
use es_entity::clock::ClockHandle;
use governance::GovernanceEvent;
use money::UsdCents;
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};

use crate::{
    FacilityProceedsFromLiquidationAccountId,
    primitives::{CoreCreditAction, CoreCreditCollectionEvent, CoreCreditObject},
};

use es_entity::Idempotent;

use crate::{collateral::public::CoreCreditCollateralEvent, primitives::*};

use ledger::{
    CollateralLedger, CollateralLedgerAccountIds, FacilityLedgerAccountIdsForLiquidation,
    LiquidationProceedsAccountIds,
};

pub(super) use entity::*;
use jobs::wallet_collateral_sync;
use jobs::wallet_collateral_sync_command::WalletCollateralSyncCommandInitializer;
pub use {
    entity::{Collateral, CollateralAdjustment},
    liquidation::Liquidation,
    primitives::*,
    repo::{CollateralRepo, liquidation_cursor},
};

#[cfg(feature = "json-schema")]
pub use entity::CollateralEvent;
use error::CollateralError;
#[cfg(feature = "json-schema")]
pub use liquidation::LiquidationEvent;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollateralEvent>
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
    E: OutboxEventMarker<CoreCreditCollateralEvent>
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
    E: OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        authz: Arc<Perms>,
        ledger: Arc<CollateralLedger>,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
        repo: Arc<CollateralRepo<E>>,
    ) -> Result<Self, CollateralError> {
        let clock = jobs.clock().clone();

        let wallet_collateral_sync_command_spawner =
            jobs.add_initializer(WalletCollateralSyncCommandInitializer::<
                <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
                E,
            >::new(ledger.clone(), repo.clone()));
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(wallet_collateral_sync::WALLET_COLLATERAL_SYNC_JOB),
                wallet_collateral_sync::WalletCollateralSyncHandler::new(
                    wallet_collateral_sync_command_spawner,
                ),
            )
            .await?;

        Ok(Self {
            authz,
            repo,
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

    pub async fn find_by_id_without_audit(
        &self,
        id: CollateralId,
    ) -> Result<Collateral, CollateralError> {
        self.repo.find_by_id(id).await
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CollateralError> {
        Ok(self.repo.begin_op().await?)
    }

    #[instrument(
        name = "collateral.record_liquidation_started_in_op",
        skip(self, db),
        err
    )]
    pub(super) async fn record_liquidation_started_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        liquidation_id: LiquidationId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
    ) -> Result<Option<SecuredLoanId>, CollateralError> {
        let mut collateral = self.repo.find_by_id_in_op(db, collateral_id).await?;

        let liquidation_proceeds_account_ids = LiquidationProceedsAccountIds::new(
            &collateral.account_ids,
            &collateral.facility_ledger_account_ids_for_liquidation,
            liquidation_proceeds_omnibus_account_id,
        );

        if collateral
            .record_liquidation_started(
                liquidation_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
                liquidation_proceeds_account_ids,
            )
            .did_execute()
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            return Ok(Some(collateral.secured_loan_id));
        }

        Ok(None)
    }

    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        secured_loan_id: SecuredLoanId,
        custody_wallet_id: Option<CustodyWalletId>,
        account_ids: CollateralLedgerAccountIds,
        facility_ledger_account_ids_for_liquidation: FacilityLedgerAccountIdsForLiquidation,
    ) -> Result<Collateral, CollateralError> {
        self.ledger
            .create_collateral_accounts_in_op(db, collateral_id, account_ids)
            .await?;

        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .secured_loan_id(secured_loan_id)
            .custody_wallet_id(custody_wallet_id)
            .account_ids(account_ids)
            .facility_ledger_account_ids_for_liquidation(
                facility_ledger_account_ids_for_liquidation,
            )
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

        let mut db = self.repo.begin_op().await?;

        let mut collateral = self.repo.find_by_id_in_op(&mut db, collateral_id).await?;

        if let es_entity::Idempotent::Executed(collateral_update) =
            collateral.record_collateral_update_via_manual_input(updated_collateral, effective)?
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .update_collateral_amount_in_op(&mut db, collateral_update, sub)
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

        let initiated_by = sub;
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
                    collateral.account_ids,
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
                .record_proceeds_from_liquidation_in_op(&mut db, data, sub)
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
        query: es_entity::PaginatedQueryArgs<liquidation_cursor::LiquidationsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            Liquidation,
            liquidation_cursor::LiquidationsByCreatedAtCursor,
        >,
        CollateralError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_liquidations(),
                CoreCreditAction::LIQUIDATION_LIST,
            )
            .await?;

        Ok(self
            .repo
            .list_liquidations_for_collateral_id(collateral_id, query)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "collateral.find_liquidation_by_id", skip(self, sub))]
    pub async fn find_liquidation_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        liquidation_id: impl Into<LiquidationId> + std::fmt::Debug,
    ) -> Result<Option<Liquidation>, CollateralError> {
        let id = liquidation_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::liquidation(id),
                CoreCreditAction::LIQUIDATION_READ,
            )
            .await?;

        Ok(self.repo.find_liquidation_by_id(id).await?)
    }

    pub async fn find_all_liquidations<T: From<Liquidation>>(
        &self,
        ids: &[LiquidationId],
    ) -> Result<HashMap<LiquidationId, T>, CollateralError> {
        Ok(self.repo.find_all_liquidations(ids).await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.collateral_ledger_account_ids_in_op",
        skip(self, db)
    )]
    pub async fn collateral_ledger_account_ids_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        id: CollateralId,
    ) -> Result<CollateralLedgerAccountIds, CollateralError> {
        let collateral = self.repo.find_by_id_in_op(db, id).await?;
        Ok(collateral.account_ids)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.liquidation_ledger_account_ids_in_op",
        skip(self, db)
    )]
    pub async fn liquidation_ledger_account_ids_in_op(
        &self,
        db: &mut impl es_entity::AtomicOperation,
        id: CollateralId,
    ) -> Result<FacilityLedgerAccountIdsForLiquidation, CollateralError> {
        let collateral = self.repo.find_by_id_in_op(db, id).await?;
        Ok(collateral.facility_ledger_account_ids_for_liquidation)
    }

    #[record_error_severity]
    #[instrument(name = "collateral.list_liquidations", skip(self, sub))]
    pub async fn list_liquidations(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<liquidation_cursor::LiquidationsByIdCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Liquidation, liquidation_cursor::LiquidationsByIdCursor>,
        CollateralError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_liquidations(),
                CoreCreditAction::LIQUIDATION_LIST,
            )
            .await?;

        Ok(self.repo.list_liquidations(query).await?)
    }
}

#[derive(Clone, Debug)]
pub struct RecordProceedsFromLiquidationData {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,

    pub amount_received: UsdCents,
    pub amount_liquidated: Satoshis,

    pub ledger_tx_id: LedgerTxId,
}

impl RecordProceedsFromLiquidationData {
    pub(crate) fn new(
        account_ids: LiquidationProceedsAccountIds,
        amount_received: UsdCents,
        amount_liquidated: Satoshis,
        ledger_tx_id: LedgerTxId,
    ) -> Self {
        Self {
            liquidation_proceeds_omnibus_account_id: account_ids
                .liquidation_proceeds_omnibus_account_id,
            proceeds_from_liquidation_account_id: account_ids.proceeds_from_liquidation_account_id,
            collateral_in_liquidation_account_id: account_ids.collateral_in_liquidation_account_id,
            liquidated_collateral_account_id: account_ids.liquidated_collateral_account_id,
            amount_received,
            amount_liquidated,
            ledger_tx_id,
        }
    }
}
pub(crate) mod publisher;
