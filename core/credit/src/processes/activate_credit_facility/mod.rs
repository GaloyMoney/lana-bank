mod job;

use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::{CorePriceEvent, Price};
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::OutboxEventMarker;
use public_id::PublicIds;

pub use job::*;

use crate::{
    CoreCreditEvent,
    collateral::ledger::CollateralLedgerOps,
    credit_facility::CreditFacilities,
    disbursal::Disbursals,
    error::CoreCreditError,
    ledger::CreditLedgerOps,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId},
};

use core_credit_collection::CollectionLedgerOps;

pub struct ActivateCreditFacility<Perms, E, L, CL, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<crate::CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
    ColL: CollectionLedgerOps,
{
    credit_facilities: Arc<CreditFacilities<Perms, E, L, CL, ColL>>,
    disbursals: Arc<Disbursals<Perms, E, ColL>>,
    ledger: Arc<L>,
    price: Arc<Price>,
    audit: Arc<Perms::Audit>,
    public_ids: Arc<PublicIds>,
}

impl<Perms, E, L, CL, ColL> Clone for ActivateCreditFacility<Perms, E, L, CL, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<crate::CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
    ColL: CollectionLedgerOps,
{
    fn clone(&self) -> Self {
        Self {
            credit_facilities: self.credit_facilities.clone(),
            disbursals: self.disbursals.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            audit: self.audit.clone(),
            public_ids: self.public_ids.clone(),
        }
    }
}
impl<Perms, E, L, CL, ColL> ActivateCreditFacility<Perms, E, L, CL, ColL>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<crate::CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
    ColL: CollectionLedgerOps,
{
    pub fn new(
        credit_facilities: Arc<CreditFacilities<Perms, E, L, CL, ColL>>,
        disbursals: Arc<Disbursals<Perms, E, ColL>>,
        ledger: Arc<L>,
        price: Arc<Price>,
        audit: Arc<Perms::Audit>,
        public_ids: Arc<PublicIds>,
    ) -> Self {
        Self {
            credit_facilities,
            disbursals,
            ledger,
            price,
            audit,
            public_ids,
        }
    }

    #[record_error_severity]
    #[instrument(name = "credit.credit_facility.activation.execute", skip(self))]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn execute_activate_credit_facility(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityId>,
    ) -> Result<(), CoreCreditError> {
        self.credit_facilities.activate(id.into()).await?;
        Ok(())
    }
}
