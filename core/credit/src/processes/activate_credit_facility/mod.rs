mod job;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::{CreditFacility, CreditFacilityRepo},
    disbursal::{Disbursals, NewDisbursal},
    error::CoreCreditError,
    event::CoreCreditEvent,
    jobs::interest_accruals,
    ledger::CreditLedger,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId, DisbursalId},
    Jobs,
};

pub use job::*;

pub struct ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    credit_facility_repo: CreditFacilityRepo<E>,
    disbursals: Disbursals<Perms, E>,
    ledger: CreditLedger,
    price: Price,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms, E> Clone for ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            credit_facility_repo: self.credit_facility_repo.clone(),
            disbursals: self.disbursals.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
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
        credit_facility_repo: &CreditFacilityRepo<E>,
        disbursals: &Disbursals<Perms, E>,
        ledger: &CreditLedger,
        price: &Price,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            credit_facility_repo: credit_facility_repo.clone(),
            disbursals: disbursals.clone(),
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
    ) -> Result<CreditFacility, CoreCreditError> {
        let id = id.into();
        let mut credit_facility = self.credit_facility_repo.find_by_id(id).await?;

        let mut db = self.credit_facility_repo.begin_op().await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_ACTIVATE,
            )
            .await?;

        let price = self.price.usd_cents_per_btc().await?;
        let now = db.now();

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        let Ok(es_entity::Idempotent::Executed((credit_facility_activation, next_accrual_period))) =
            credit_facility.activate(now, price, balances, audit_info.clone())
        else {
            return Ok(credit_facility);
        };

        let due_date = credit_facility.matures_at.expect("Facility is not active");
        let overdue_date = credit_facility
            .terms
            .obligation_overdue_duration
            .map(|d| d.end_date(due_date));
        let new_disbursal = NewDisbursal::builder()
            .id(DisbursalId::new())
            .credit_facility_id(credit_facility.id)
            .approval_process_id(credit_facility.approval_process_id)
            .amount(credit_facility.structuring_fee())
            .account_ids(credit_facility.account_ids)
            .disbursal_credit_account_id(credit_facility.disbursal_credit_account_id)
            .disbursal_due_date(due_date)
            .disbursal_overdue_date(overdue_date)
            .audit_info(audit_info.clone())
            .build()
            .expect("could not build new disbursal");

        self.disbursals
            .create_first_disbursal_in_op(&mut db, new_disbursal, &audit_info)
            .await?;

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        let accrual_id = credit_facility
            .interest_accrual_cycle_in_progress()
            .expect("First accrual not found")
            .id;
        self.jobs
            .create_and_spawn_at_in_op(
                &mut db,
                accrual_id,
                interest_accruals::CreditFacilityJobConfig::<Perms, E> {
                    credit_facility_id: id,
                    _phantom: std::marker::PhantomData,
                },
                next_accrual_period.end,
            )
            .await?;

        self.ledger
            .activate_credit_facility(db, credit_facility_activation)
            .await?;

        Ok(credit_facility)
    }
}
