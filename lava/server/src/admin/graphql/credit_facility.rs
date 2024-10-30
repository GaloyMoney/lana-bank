use async_graphql::{dataloader::DataLoader, *};

use super::{approval_process::ApprovalProcess, loader::LavaDataLoader};
use crate::{
    admin::{graphql::user::User, AdminAuthContext},
    shared_graphql::{
        convert::ToGlobalId,
        customer::Customer,
        objects::*,
        primitives::{Timestamp, UUID},
        terms::*,
    },
};
use lava_app::{
    app::LavaApp, credit_facility::FacilityCVLData, ledger, primitives::*,
    terms::CollateralizationState,
};

pub use lava_app::primitives::DisbursementIdx;

#[derive(SimpleObject)]
pub(super) struct CreditFacilityBalance {
    facility_remaining: FacilityRemaining,
    disbursed: Disbursed,
    interest: Interest,
    outstanding: Outstanding,
    collateral: Collateral,
}

impl From<ledger::credit_facility::CreditFacilityBalance> for CreditFacilityBalance {
    fn from(balance: ledger::credit_facility::CreditFacilityBalance) -> Self {
        Self {
            facility_remaining: FacilityRemaining {
                usd_balance: balance.facility,
            },
            disbursed: Disbursed {
                total: Total {
                    usd_balance: balance.disbursed,
                },
                outstanding: Outstanding {
                    usd_balance: balance.disbursed_receivable,
                },
            },
            interest: Interest {
                total: Total {
                    usd_balance: balance.interest,
                },
                outstanding: Outstanding {
                    usd_balance: balance.accrued_interest_receivable,
                },
            },
            outstanding: Outstanding {
                usd_balance: balance.disbursed_receivable + balance.accrued_interest_receivable,
            },
            collateral: Collateral {
                btc_balance: balance.collateral,
            },
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityCreateInput {
    pub customer_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
    approval_process_id: UUID,
    activated_at: Option<Timestamp>,
    expires_at: Option<Timestamp>,
    created_at: Timestamp,
    credit_facility_terms: TermValues,
    status: CreditFacilityStatus,
    collateralization_state: CollateralizationState,
    facility_amount: UsdCents,
    collateral: Satoshis,
    can_be_completed: bool,
    transactions: Vec<CreditFacilityHistoryEntry>,
    #[graphql(skip)]
    account_ids: lava_app::ledger::credit_facility::CreditFacilityAccountIds,
    #[graphql(skip)]
    cvl_data: FacilityCVLData,
    #[graphql(skip)]
    domain_approval_process_id: governance::ApprovalProcessId,
    #[graphql(skip)]
    domain_customer_id: CustomerId,
}

#[derive(async_graphql::Union, Clone)]
pub enum CreditFacilityHistoryEntry {
    Payment(CreditFacilityIncrementalPayment),
    Collateral(CreditFacilityCollateralUpdated),
    Origination(CreditFacilityOrigination),
    Collateralization(CreditFacilityCollateralizationUpdated),
    Disbursement(CreditFacilityDisbursementExecuted),
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityIncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityCollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: Timestamp,
    pub action: CollateralAction,
    pub tx_id: UUID,
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityOrigination {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityCollateralizationUpdated {
    pub state: CollateralizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursement: UsdCents,
    pub recorded_at: Timestamp,
    pub price: UsdCents,
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityDisbursementExecuted {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[ComplexObject]
impl CreditFacility {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app
            .ledger()
            .get_credit_facility_balance(self.account_ids)
            .await?;
        Ok(CreditFacilityBalance::from(balance))
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let process = loader
            .load_one(self.domain_approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let customer = loader
            .load_one(self.domain_customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn disbursements(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursement>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let disbursements = app
            .credit_facilities()
            .list_disbursements(sub, CreditFacilityId::from(&self.credit_facility_id))
            .await?;

        Ok(disbursements
            .into_iter()
            .map(CreditFacilityDisbursement::from)
            .collect())
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<FacilityCVL> {
        let app = ctx.data_unchecked::<LavaApp>();
        let price = app.price().usd_cents_per_btc().await?;
        Ok(FacilityCVL::from(self.cvl_data.cvl(price)))
    }

    async fn user_can_update_collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_update_collateral(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_initiate_disbursement(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_initiate_disbursement(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_record_payment(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_record_payment(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_complete(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_complete(sub, false)
            .await
            .is_ok())
    }
}

#[derive(SimpleObject)]
pub struct CreditFacilityCreatePayload {
    credit_facility: CreditFacility,
}

#[derive(InputObject)]
pub struct CreditFacilityCompleteInput {
    pub credit_facility_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCompletePayload {
    credit_facility: CreditFacility,
}

impl From<lava_app::credit_facility::CreditFacility> for CreditFacilityCompletePayload {
    fn from(credit_facility: lava_app::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
}

impl ToGlobalId for lava_app::primitives::CreditFacilityId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("credit-facility:{}", self))
    }
}

impl From<lava_app::credit_facility::CreditFacility> for CreditFacility {
    fn from(credit_facility: lava_app::credit_facility::CreditFacility) -> Self {
        let activated_at: Option<Timestamp> = credit_facility.activated_at.map(|t| t.into());
        let expires_at: Option<Timestamp> = credit_facility.expires_at.map(|t| t.into());
        let transactions = credit_facility
            .history()
            .into_iter()
            .map(CreditFacilityHistoryEntry::from)
            .collect();

        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            approval_process_id: UUID::from(credit_facility.approval_process_id),
            activated_at,
            expires_at,
            created_at: credit_facility.created_at().into(),
            account_ids: credit_facility.account_ids,
            cvl_data: credit_facility.facility_cvl_data(),
            credit_facility_terms: TermValues::from(credit_facility.terms),
            status: credit_facility.status(),
            can_be_completed: credit_facility.can_be_completed(),
            transactions,
            facility_amount: credit_facility.initial_facility(),
            collateral: credit_facility.collateral(),
            collateralization_state: credit_facility.last_collateralization_state(),
            domain_customer_id: credit_facility.customer_id,
            domain_approval_process_id: credit_facility.approval_process_id,
        }
    }
}

impl From<lava_app::credit_facility::CreditFacility> for CreditFacilityCreatePayload {
    fn from(credit_facility: lava_app::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: CreditFacility::from(credit_facility),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}

#[derive(SimpleObject)]
pub struct CreditFacilityPartialPaymentPayload {
    credit_facility: CreditFacility,
}

impl From<lava_app::credit_facility::CreditFacility> for CreditFacilityPartialPaymentPayload {
    fn from(credit_facility: lava_app::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct CreditFacilityDisbursement {
    id: ID,
    index: DisbursementIdx,
    amount: UsdCents,
    status: DisbursementStatus,
    created_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DisbursementApproval {
    #[graphql(skip)]
    user_id: lava_app::primitives::UserId,
    approved_at: Timestamp,
}

#[ComplexObject]
impl DisbursementApproval {
    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let user = app
            .users()
            .find_by_id(sub, self.user_id)
            .await?
            .expect("should always find user for a given UserId");
        Ok(User::from(user))
    }
}

impl From<lava_app::credit_facility::Disbursement> for CreditFacilityDisbursement {
    fn from(disbursement: lava_app::credit_facility::Disbursement) -> Self {
        Self {
            id: disbursement.id.to_global_id(),
            index: disbursement.idx,
            amount: disbursement.amount,
            status: disbursement.status(),
            created_at: disbursement.created_at().into(),
        }
    }
}

impl ToGlobalId for lava_app::primitives::DisbursementId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("disbursement:{}", self))
    }
}
#[derive(InputObject)]
pub struct CreditFacilityDisbursementInitiateInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}

#[derive(SimpleObject)]
pub struct CreditFacilityDisbursementInitiatePayload {
    disbursement: CreditFacilityDisbursement,
}

impl From<lava_app::credit_facility::Disbursement> for CreditFacilityDisbursementInitiatePayload {
    fn from(disbursement: lava_app::credit_facility::Disbursement) -> Self {
        Self {
            disbursement: CreditFacilityDisbursement::from(disbursement),
        }
    }
}

impl From<lava_app::credit_facility::DisbursementApproval> for DisbursementApproval {
    fn from(disbursement_approval: lava_app::credit_facility::DisbursementApproval) -> Self {
        Self {
            user_id: disbursement_approval.user_id,
            approved_at: disbursement_approval.approved_at.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityDisbursementConfirmInput {
    pub credit_facility_id: UUID,
    pub disbursement_idx: DisbursementIdx,
}

#[derive(SimpleObject)]
pub struct CreditFacilityDisbursementConfirmPayload {
    disbursement: CreditFacilityDisbursement,
}

impl From<lava_app::credit_facility::Disbursement> for CreditFacilityDisbursementConfirmPayload {
    fn from(disbursement: lava_app::credit_facility::Disbursement) -> Self {
        Self {
            disbursement: CreditFacilityDisbursement::from(disbursement),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityCollateralUpdateInput {
    pub credit_facility_id: UUID,
    pub collateral: Satoshis,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralUpdatePayload {
    credit_facility: CreditFacility,
}

impl From<lava_app::credit_facility::CreditFacility> for CreditFacilityCollateralUpdatePayload {
    fn from(credit_facility: lava_app::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
}

impl From<lava_app::credit_facility::CreditFacilityHistoryEntry> for CreditFacilityHistoryEntry {
    fn from(transaction: lava_app::credit_facility::CreditFacilityHistoryEntry) -> Self {
        match transaction {
            lava_app::credit_facility::CreditFacilityHistoryEntry::Payment(payment) => {
                CreditFacilityHistoryEntry::Payment(payment.into())
            }
            lava_app::credit_facility::CreditFacilityHistoryEntry::Collateral(collateral) => {
                CreditFacilityHistoryEntry::Collateral(collateral.into())
            }
            lava_app::credit_facility::CreditFacilityHistoryEntry::Origination(origination) => {
                CreditFacilityHistoryEntry::Origination(origination.into())
            }
            lava_app::credit_facility::CreditFacilityHistoryEntry::Collateralization(
                collateralization,
            ) => CreditFacilityHistoryEntry::Collateralization(collateralization.into()),
            lava_app::credit_facility::CreditFacilityHistoryEntry::Disbursement(disbursement) => {
                CreditFacilityHistoryEntry::Disbursement(disbursement.into())
            }
        }
    }
}

impl From<lava_app::credit_facility::IncrementalPayment> for CreditFacilityIncrementalPayment {
    fn from(payment: lava_app::credit_facility::IncrementalPayment) -> Self {
        Self {
            cents: payment.cents,
            recorded_at: payment.recorded_at.into(),
            tx_id: UUID::from(payment.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::CollateralUpdated> for CreditFacilityCollateralUpdated {
    fn from(collateral: lava_app::credit_facility::CollateralUpdated) -> Self {
        Self {
            satoshis: collateral.satoshis,
            recorded_at: collateral.recorded_at.into(),
            action: collateral.action,
            tx_id: UUID::from(collateral.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::CreditFacilityOrigination> for CreditFacilityOrigination {
    fn from(origination: lava_app::credit_facility::CreditFacilityOrigination) -> Self {
        Self {
            cents: origination.cents,
            recorded_at: origination.recorded_at.into(),
            tx_id: UUID::from(origination.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::DisbursementExecuted> for CreditFacilityDisbursementExecuted {
    fn from(disbursement: lava_app::credit_facility::DisbursementExecuted) -> Self {
        Self {
            cents: disbursement.cents,
            recorded_at: disbursement.recorded_at.into(),
            tx_id: UUID::from(disbursement.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::CollateralizationUpdated>
    for CreditFacilityCollateralizationUpdated
{
    fn from(collateralization: lava_app::credit_facility::CollateralizationUpdated) -> Self {
        Self {
            state: collateralization.state,
            collateral: collateralization.collateral,
            outstanding_interest: collateralization.outstanding_interest,
            outstanding_disbursement: collateralization.outstanding_disbursement,
            recorded_at: collateralization.recorded_at.into(),
            price: collateralization.price.into_inner(),
        }
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
