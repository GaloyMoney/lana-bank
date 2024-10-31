mod balance;
mod disbursal;
mod history;

use async_graphql::*;

use crate::primitives::*;

use super::{approval_process::*, customer::*, loader::LavaDataLoader, terms::*};
pub use lava_app::{
    credit_facility::{CreditFacility as DomainCreditFacility, CreditFacilityByCreatedAtCursor},
    primitives::CreditFacilityStatus,
};

pub use balance::*;
pub use disbursal::*;
pub use history::*;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
    approval_process_id: UUID,
    activated_at: Option<Timestamp>,
    expires_at: Option<Timestamp>,
    created_at: Timestamp,
    status: CreditFacilityStatus,
    collateralization_state: CollateralizationState,
    facility_amount: UsdCents,
    collateral: Satoshis,
    // can_be_completed: bool,
    // transactions: Vec<CreditFacilityHistoryEntry>,
    // #[graphql(skip)]
    // account_ids: lava_app::ledger::credit_facility::CreditFacilityAccountIds,
    // #[graphql(skip)]
    // cvl_data: FacilityCVLData,
    // #[graphql(skip)]
    // domain_approval_process_id: governance::ApprovalProcessId,
    // #[graphql(skip)]
    // domain_customer_id: CustomerId,
    //
    #[graphql(skip)]
    pub(super) entity: Arc<DomainCreditFacility>,
}

impl From<DomainCreditFacility> for CreditFacility {
    fn from(credit_facility: DomainCreditFacility) -> Self {
        let activated_at: Option<Timestamp> = credit_facility.activated_at.map(|t| t.into());
        let expires_at: Option<Timestamp> = credit_facility.expires_at.map(|t| t.into());

        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            approval_process_id: UUID::from(credit_facility.approval_process_id),
            activated_at,
            expires_at,
            created_at: credit_facility.created_at().into(),
            status: credit_facility.status(),
            facility_amount: credit_facility.initial_facility(),
            collateral: credit_facility.collateral(),
            collateralization_state: credit_facility.last_collateralization_state(),

            entity: Arc::new(credit_facility),
        }
    }
}

#[ComplexObject]
impl CreditFacility {
    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<FacilityCVL> {
        let app = ctx.data_unchecked::<LavaApp>();
        let price = app.price().usd_cents_per_btc().await?;
        Ok(FacilityCVL::from(
            self.entity.facility_cvl_data().cvl(price),
        ))
    }

    async fn transactions(&self) -> Vec<CreditFacilityHistoryEntry> {
        self.entity
            .history()
            .into_iter()
            .map(CreditFacilityHistoryEntry::from)
            .collect()
    }

    async fn disbursements(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursement>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursements = app
            .credit_facilities()
            .list_disbursements(sub, self.entity.id)
            .await?;

        Ok(disbursements
            .into_iter()
            .map(CreditFacilityDisbursement::from)
            .collect())
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn user_can_update_collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit_facilities()
            .subject_can_update_collateral(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_initiate_disbursement(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit_facilities()
            .subject_can_initiate_disbursement(sub, false)
            .await
            .is_ok())
    }

    //     async fn user_can_approve_disbursement(
    //         &self,
    //         ctx: &Context<'_>,
    //     ) -> async_graphql::Result<bool> {
    //         let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
    //         Ok(app
    //             .credit_facilities()
    //             .subject_can_approve_disbursement(sub, false)
    //             .await
    //             .is_ok())
    //     }

    async fn user_can_record_payment(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit_facilities()
            .subject_can_record_payment(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_complete(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit_facilities()
            .subject_can_complete(sub, false)
            .await
            .is_ok())
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app
            .ledger()
            .get_credit_facility_balance(self.entity.account_ids)
            .await?;
        Ok(CreditFacilityBalance::from(balance))
    }
}

#[derive(SimpleObject)]
pub struct FacilityCVL {
    total: CVLPct,
    disbursed: CVLPct,
}

impl From<lava_app::credit_facility::FacilityCVL> for FacilityCVL {
    fn from(value: lava_app::credit_facility::FacilityCVL) -> Self {
        Self {
            total: value.total,
            disbursed: value.disbursed,
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityCreateInput {
    pub customer_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
}
crate::mutation_payload! { CreditFacilityCreatePayload, credit_facility: CreditFacility }

#[derive(InputObject)]
pub struct CreditFacilityCollateralUpdateInput {
    pub credit_facility_id: UUID,
    pub collateral: Satoshis,
}
crate::mutation_payload! { CreditFacilityCollateralUpdatePayload, credit_facility: CreditFacility }

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}
crate::mutation_payload! { CreditFacilityPartialPaymentPayload, credit_facility: CreditFacility }

#[derive(InputObject)]
pub struct CreditFacilityCompleteInput {
    pub credit_facility_id: UUID,
}
crate::mutation_payload! { CreditFacilityCompletePayload, credit_facility: CreditFacility }
