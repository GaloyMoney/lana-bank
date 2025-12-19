mod entity;
pub mod error;
mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use authz::PermissionCheck;
use obix::out::OutboxEventMarker;

use crate::{CreditFacilityPublisher, CreditLedger, event::CoreCreditEvent, primitives::*};

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
    authz: Arc<Perms>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CreditLedger>,
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
        authz: Arc<Perms>,
        publisher: &CreditFacilityPublisher<E>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            authz,
            repo: Arc::new(CollateralRepo::new(pool, publisher)),
            ledger,
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
        name = "collateral.record_collateral_update_via_custodian_sync",
        fields(updated_collateral = %updated_collateral, effective = %effective),
        skip(self),
    )]
    pub(super) async fn record_collateral_update_via_custodian_sync(
        &self,
        custody_wallet_id: CustodyWalletId,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<(), CollateralError> {
        let mut collateral = self
            .repo
            .find_by_custody_wallet_id(Some(custody_wallet_id))
            .await?;

        if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_custodian_sync(updated_collateral, effective)
        {
            let mut db = self.repo.begin_op().await?;

            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .update_credit_facility_collateral(db, data, collateral.account_id)
                .await?;
        }

        Ok(())
    }
}
