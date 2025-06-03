mod entity;
pub mod error;
mod repo;

use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{CreditFacilityPublisher, event::CoreCreditEvent, primitives::*};

pub use entity::Collateral;
pub(super) use entity::*;
use error::CollateralError;
use repo::CollateralRepo;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Perms,
    repo: CollateralRepo<E>,
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
        }
    }
}

impl<Perms, E> Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            authz: authz.clone(),
            repo: CollateralRepo::new(pool, publisher),
        }
    }

    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        credit_facility_id: CreditFacilityId,
        account_id: CalaAccountId,
    ) -> Result<Collateral, CollateralError> {
        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .credit_facility_id(credit_facility_id)
            .account_id(account_id)
            .build()
            .expect("all fields for new collateral provided");

        self.repo.create_in_op(db, new_collateral).await
    }

    pub(super) async fn record_collateral_update_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
        audit_info: &audit::AuditInfo,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        let res = if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update(updated_collateral, effective, audit_info)
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }
}
