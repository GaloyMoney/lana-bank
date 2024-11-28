mod job;

use tracing::instrument;

use crate::{
    audit::{Audit, AuditSvc},
    credit_facility::{error::CreditFacilityError, interest, CreditFacility, CreditFacilityRepo},
    job::{error::JobError, Jobs},
    ledger::Ledger,
    price::Price,
    primitives::CreditFacilityId,
};
use rbac_types::{AppObject, CreditFacilityAction};

pub use job::*;

#[derive(Clone)]
pub struct ActivateCreditFacility {
    credit_facility_repo: CreditFacilityRepo,
    ledger: Ledger,
    price: Price,
    jobs: Jobs,
    audit: Audit,
}

impl ActivateCreditFacility {
    pub(in crate::credit_facility) fn new(
        credit_facility_repo: &CreditFacilityRepo,
        ledger: &Ledger,
        price: &Price,
        jobs: &Jobs,
        audit: &Audit,
    ) -> Self {
        Self {
            credit_facility_repo: credit_facility_repo.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(name = "credit_facility.activation.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityId>,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut credit_facility = self.credit_facility_repo.find_by_id(id.into()).await?;

        let price = self.price.usd_cents_per_btc().await?;
        let Ok(credit_facility_activation) = credit_facility.activation_data(price) else {
            return Ok(credit_facility);
        };

        let mut db = self.credit_facility_repo.begin_op().await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                AppObject::CreditFacility,
                CreditFacilityAction::Activate,
            )
            .await?;

        let now = crate::time::now();

        let next_incurrence_period =
            match credit_facility.activate(credit_facility_activation.clone(), now, audit_info) {
                es_entity::Idempotent::Executed(next_incurrence_period) => next_incurrence_period,
                es_entity::Idempotent::AlreadyApplied => {
                    return Ok(credit_facility);
                }
            };

        match self
            .jobs
            .create_and_spawn_at_in_op(
                &mut db,
                credit_facility.id,
                interest::CreditFacilityJobConfig {
                    credit_facility_id: credit_facility.id,
                },
                next_incurrence_period.end,
            )
            .await
        {
            Ok(_) | Err(JobError::DuplicateId) => (),
            Err(err) => Err(err)?,
        };

        self.ledger
            .activate_credit_facility(credit_facility_activation)
            .await?;

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        db.commit().await?;
        Ok(credit_facility)
    }
}
