mod collateral;
mod error;
mod event;
mod primitives;
mod publisher;
mod rbac;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::{Outbox, OutboxEventMarker};

use collateral::*;
use error::*;
use event::CoreCreditCollateralEvent;
use primitives::*;
use rbac::*;

pub struct CoreCreditCollaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<GovernanceEvent>,
{
    authz: Perms,
    collaterals: CollateralRepo<E>,
}

impl<Perms, E> Clone for CoreCreditCollaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            collaterals: self.collaterals.clone(),
        }
    }
}

impl<Perms, E> CoreCreditCollaterals<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditCollateralAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditCollateralObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn new(pool: &sqlx::PgPool, outbox: &Outbox<E>, authz: Perms) -> Self {
        let publisher = publisher::CollateralPublisher::new(outbox);
        let collaterals = CollateralRepo::new(pool, &publisher);
        Self { authz, collaterals }
    }
}
