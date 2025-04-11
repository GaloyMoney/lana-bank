mod entity;
pub mod error;
mod facility_obligations;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId, ObligationId},
    publisher::CreditFacilityPublisher,
};

pub use entity::Obligation;
pub(crate) use entity::*;
use error::ObligationError;
pub(crate) use facility_obligations::FacilityObligations;
pub(crate) use repo::*;

pub struct Obligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Perms,
    repo: ObligationRepo<E>,
}

impl<Perms, E> Clone for Obligations<Perms, E>
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

impl<Perms, E> Obligations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub(crate) fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        _cala: &CalaLedger,
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let obligation_repo = ObligationRepo::new(pool, publisher);
        Self {
            authz: authz.clone(),
            repo: obligation_repo,
        }
    }

    pub async fn facility_obligations(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<FacilityObligations, ObligationError> {
        let mut obligations = vec![];
        let mut query = Default::default();
        loop {
            let mut res = self
                .repo
                .list_for_credit_facility_id_by_created_at(
                    credit_facility_id,
                    query,
                    es_entity::ListDirection::Ascending,
                )
                .await?;

            obligations.append(&mut res.entities);

            if let Some(q) = res.into_next_query() {
                query = q;
            } else {
                break;
            };
        }

        Ok(FacilityObligations::new(credit_facility_id, obligations))
    }
}
