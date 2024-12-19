mod job;

use tracing::instrument;

use crate::{
    audit::{Audit, AuditSvc},
    credit_facility::{
        error::CreditFacilityError, interest_accruals, interest_incurrences, ledger::CreditLedger,
        CreditFacility, CreditFacilityRepo, DisbursalRepo,
    },
    job::{error::JobError, Jobs},
    price::Price,
    primitives::CreditFacilityId,
};
use rbac_types::{AppObject, CreditFacilityAction};

pub use job::*;

#[derive(Clone)]
pub struct ActivateCreditFacility {
    credit_facility_repo: CreditFacilityRepo,
    disbursal_repo: DisbursalRepo,
    ledger: CreditLedger,
    price: Price,
    jobs: Jobs,
    audit: Audit,
}

impl ActivateCreditFacility {
    pub(in crate::credit_facility) fn new(
        credit_facility_repo: &CreditFacilityRepo,
        disbursal_repo: &DisbursalRepo,
        ledger: &CreditLedger,
        price: &Price,
        jobs: &Jobs,
        audit: &Audit,
    ) -> Self {
        Self {
            credit_facility_repo: credit_facility_repo.clone(),
            disbursal_repo: disbursal_repo.clone(),
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
        let mut credit_facility @ CreditFacility {
            id: credit_facility_id,
            ..
        } = self.credit_facility_repo.find_by_id(id.into()).await?;

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

        let es_entity::Idempotent::Executed(next_incurrence_period) =
            credit_facility.activate(credit_facility_activation.clone(), now, audit_info.clone())
        else {
            return Ok(credit_facility);
        };

        let new_disbursal = credit_facility.initiate_disbursal(
            credit_facility.structuring_fee(),
            now,
            price,
            Some(credit_facility.approval_process_id),
            audit_info.clone(),
        )?;
        let mut disbursal = self
            .disbursal_repo
            .create_in_op(&mut db, new_disbursal)
            .await?;
        disbursal
            .approval_process_concluded(true, audit_info.clone())
            .did_execute();
        let disbursal_data = disbursal.record(now, audit_info.clone())?;
        credit_facility.confirm_disbursal(
            &disbursal,
            Some(disbursal_data.tx_id),
            now,
            audit_info.clone(),
        );

        self.disbursal_repo
            .update_in_op(&mut db, &mut disbursal)
            .await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        let accrual_id = credit_facility
            .interest_accrual_in_progress()
            .expect("First accrual not found")
            .id;
        match self
            .jobs
            .create_and_spawn_at_in_op(
                &mut db,
                accrual_id,
                interest_incurrences::CreditFacilityJobConfig { credit_facility_id },
                next_incurrence_period.incurrence.end,
            )
            .await
        {
            Ok(_) | Err(JobError::DuplicateId) => (),
            Err(err) => Err(err)?,
        };
        match self
            .jobs
            .create_and_spawn_at_in_op(
                &mut db,
                credit_facility_id,
                interest_accruals::CreditFacilityJobConfig { credit_facility_id },
                next_incurrence_period.accrual.end,
            )
            .await
        {
            Ok(_) | Err(JobError::DuplicateId) => (),
            Err(err) => Err(err)?,
        };

        self.ledger
            .activate_credit_facility(db, credit_facility_activation)
            .await?;

        Ok(credit_facility)
    }
}
