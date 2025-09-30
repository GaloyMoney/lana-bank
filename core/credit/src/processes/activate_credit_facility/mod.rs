mod job;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{
    CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject,
};
use core_price::Price;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::OutboxEventMarker;
use public_id::PublicIds;

use crate::{
    Jobs,
    credit_facility::{ActivationData, ActivationOutcome, CreditFacilities},
    disbursal::Disbursals,
    error::CoreCreditError,
    event::CoreCreditEvent,
    jobs::interest_accruals,
    ledger::CreditLedger,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId},
};

pub use job::*;

pub struct ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    credit_facilities: CreditFacilities<Perms, E>,
    disbursals: Disbursals<Perms, E>,
    ledger: CreditLedger,
    price: Price,
    jobs: Jobs,
    audit: Perms::Audit,
    public_ids: PublicIds,
}

impl<Perms, E> Clone for ActivateCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
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
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        credit_facilities: &CreditFacilities<Perms, E>,
        disbursals: &Disbursals<Perms, E>,
        ledger: &CreditLedger,
        price: &Price,
        jobs: &Jobs,
        audit: &Perms::Audit,
        public_ids: &PublicIds,
    ) -> Self {
        Self {
            credit_facilities: credit_facilities.clone(),
            disbursals: disbursals.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
            public_ids: public_ids.clone(),
        }
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(name = "credit.credit_facility.activation.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityId>,
    ) -> Result<(), CoreCreditError> {
        let id = id.into();
        let mut op = self
            .credit_facilities
            .begin_op()
            .await?
            .with_db_time()
            .await?;

        let ActivationData {
            credit_facility,
            next_accrual_period,
        } = match self.credit_facilities.activate_in_op(&mut op, id).await? {
            ActivationOutcome::Activated(data) => data,
            ActivationOutcome::Ignored => {
                return Ok(());
            }
        };

        let accrual_id = credit_facility
            .interest_accrual_cycle_in_progress()
            .expect("First accrual not found")
            .id;

        self.jobs
            .create_and_spawn_at_in_op(
                &mut op,
                accrual_id,
                interest_accruals::InterestAccrualJobConfig::<Perms, E> {
                    credit_facility_id: id,
                    _phantom: std::marker::PhantomData,
                },
                next_accrual_period.end,
            )
            .await?;

        if !credit_facility.structuring_fee().is_zero() {
            let disbursal_id = self
                .disbursals
                .create_first_disbursal_in_op(&mut op, &credit_facility)
                .await?;

            self.ledger
                .handle_activation_with_structuring_fee(
                    op,
                    credit_facility.activation_data(),
                    disbursal_id,
                )
                .await?;

            return Ok(());
        }

        self.ledger
            .handle_facility_activation(op, credit_facility.activation_data())
            .await?;

        Ok(())
    }
}
