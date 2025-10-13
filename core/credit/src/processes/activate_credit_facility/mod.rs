mod job;

use std::sync::Arc;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::OutboxEventMarker;
use public_id::PublicIds;

use crate::{
    Jobs,
    credit_facility::CreditFacilities,
    disbursal::Disbursals,
    error::CoreCreditError,
    event::CoreCreditEvent,
    ledger::CreditLedger,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId},
};

pub use job::*;

pub struct ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
    disbursals: Arc<Disbursals<Perms, E>>,
    ledger: Arc<CreditLedger>,
    price: Arc<Price>,
    jobs: Arc<Jobs>,
    audit: Arc<Perms::Audit>,
    public_ids: Arc<PublicIds>,
}

impl<Perms, E> Clone for ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            credit_facilities: self.credit_facilities.clone(),
            disbursals: self.disbursals.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
            public_ids: self.public_ids.clone(),
        }
    }
}
impl<Perms, E> ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        credit_facilities: Arc<CreditFacilities<Perms, E>>,
        disbursals: Arc<Disbursals<Perms, E>>,
        ledger: Arc<CreditLedger>,
        price: Arc<Price>,
        jobs: Arc<Jobs>,
        audit: Arc<Perms::Audit>,
        public_ids: Arc<PublicIds>,
    ) -> Self {
        Self {
            credit_facilities,
            disbursals,
            ledger,
            price,
            jobs,
            audit,
            public_ids,
        }
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(name = "credit.credit_facility.activation.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityId>,
    ) -> Result<(), CoreCreditError> {
        self.credit_facilities.activate(id.into()).await?;
        Ok(())
    }
}
