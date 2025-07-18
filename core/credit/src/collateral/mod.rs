mod entity;
pub mod error;
mod repo;

use std::collections::HashMap;

use tracing::instrument;

use authz::PermissionCheck;
use core_custody::WalletId;
use outbox::OutboxEventMarker;

use crate::{
    CreditFacility, CreditFacilityPublisher, CreditLedger, event::CoreCreditEvent, primitives::*,
};

pub use entity::Collateral;
pub(super) use entity::*;

#[cfg(feature = "json-schema")]
pub use entity::CollateralEvent;
use error::CollateralError;
use repo::CollateralRepo;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Perms,
    repo: CollateralRepo<E>,
    ledger: CreditLedger,
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
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        publisher: &CreditFacilityPublisher<E>,
        ledger: &CreditLedger,
    ) -> Self {
        Self {
            authz: authz.clone(),
            repo: CollateralRepo::new(pool, publisher),
            ledger: ledger.clone(),
        }
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
        credit_facility_id: CreditFacilityId,
        wallet_id: Option<WalletId>,
        account_id: CalaAccountId,
    ) -> Result<Collateral, CollateralError> {
        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .credit_facility_id(credit_facility_id)
            .account_id(account_id)
            .wallet_id(wallet_id)
            .build()
            .expect("all fields for new collateral provided");

        self.repo.create_in_op(db, new_collateral).await
    }

    #[instrument(
        name = "collateral.record_collateral_update_via_manual_input_in_op",
        skip(db, self),
        err
    )]
    pub(super) async fn record_collateral_update_via_manual_input_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
        audit_info: &audit::AuditInfo,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        if collateral.wallet_id.is_some() {
            return Err(CollateralError::ManualUpdateError);
        }

        let res = if let es_entity::Idempotent::Executed(data) = collateral
            .record_collateral_update_via_manual_input(updated_collateral, effective, audit_info)
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }

    #[instrument(
        name = "collateral.record_collateral_update_via_custodian_sync",
        fields(credit_facility = %credit_facility.id, updated_collateral = %updated_collateral, effective = %effective),
        skip(self),
        err
    )]
    pub(super) async fn record_collateral_update_via_custodian_sync(
        &self,
        credit_facility: &CreditFacility,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<(), CollateralError> {
        let mut collateral = self.repo.find_by_id(credit_facility.collateral_id).await?;

        if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_custodian_sync(updated_collateral, effective)
        {
            let mut db = self.repo.begin_op().await?;

            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .update_credit_facility_collateral(db, data, credit_facility.account_ids)
                .await?;
        }

        Ok(())
    }
}
