use async_graphql::*;
use connection::CursorType;

use crate::{
    app::LavaApp,
    credit_facility::FacilityCVLData,
    ledger,
    primitives::*,
    server::{
        admin::{graphql::user::User, AdminAuthContext},
        shared_graphql::{
            convert::ToGlobalId,
            customer::Customer,
            objects::*,
            primitives::{Timestamp, UUID},
            terms::*,
        },
    },
    terms::CollateralizationState,
};

pub use crate::primitives::DisbursementIdx;

scalar!(DisbursementIdx);

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

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
    approved_at: Option<Timestamp>,
    expires_at: Option<Timestamp>,
    created_at: Timestamp,
    credit_facility_terms: TermValues,
    status: CreditFacilityStatus,
    approvals: Vec<CreditFacilityApproval>,
    collateralization_state: CollateralizationState,
    facility_amount: UsdCents,
    collateral: Satoshis,
    can_be_completed: bool,
    transactions: Vec<CreditFacilityHistoryEntry>,
    #[graphql(skip)]
    customer_id: UUID,
    #[graphql(skip)]
    account_ids: crate::ledger::credit_facility::CreditFacilityAccountIds,
    #[graphql(skip)]
    cvl_data: FacilityCVLData,
}

#[derive(async_graphql::Union)]
pub enum CreditFacilityHistoryEntry {
    Payment(CreditFacilityIncrementalPayment),
    Collateral(CreditFacilityCollateralUpdated),
    Origination(CreditFacilityOrigination),
    Collateralization(CreditFacilityCollateralizationUpdated),
    Disbursement(CreditFacilityDisbursementExecuted),
}

#[derive(SimpleObject)]
pub struct CreditFacilityIncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: Timestamp,
    pub action: CollateralAction,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityOrigination {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralizationUpdated {
    pub state: CollateralizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursement: UsdCents,
    pub recorded_at: Timestamp,
    pub price: UsdCents,
}

#[derive(SimpleObject)]
pub struct CreditFacilityDisbursementExecuted {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityApproval {
    user_id: UUID,
    approved_at: Timestamp,
}

#[ComplexObject]
impl CreditFacilityApproval {
    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app
            .users()
            .find_by_id_internal(UserId::from(&self.user_id))
            .await?
            .expect("should always find user for a given UserId");
        Ok(User::from(user))
    }
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

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app
            .customers()
            .find_by_id(None, CustomerId::from(&self.customer_id))
            .await?;

        match user {
            Some(user) => Ok(Customer::from(user)),
            None => panic!("user not found for a loan. should not be possible"),
        }
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

    async fn user_can_approve(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_approve(sub, false)
            .await
            .is_ok())
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

    async fn user_can_approve_disbursement(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_approve_disbursement(sub, false)
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
pub struct CreditFacilityApproveInput {
    pub credit_facility_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityApprovePayload {
    credit_facility: CreditFacility,
}

impl From<crate::credit_facility::CreditFacility> for CreditFacilityApprovePayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityCompleteInput {
    pub credit_facility_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCompletePayload {
    credit_facility: CreditFacility,
}

impl From<crate::credit_facility::CreditFacility> for CreditFacilityCompletePayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
}

impl ToGlobalId for crate::primitives::CreditFacilityId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("credit-facility:{}", self))
    }
}

impl From<crate::credit_facility::CreditFacility> for CreditFacility {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        let approved_at: Option<Timestamp> = credit_facility.approved_at.map(|t| t.into());
        let expires_at: Option<Timestamp> = credit_facility.expires_at.map(|t| t.into());
        let approvals = credit_facility
            .approvals()
            .into_iter()
            .map(CreditFacilityApproval::from)
            .collect();
        let transactions = credit_facility
            .history()
            .into_iter()
            .map(CreditFacilityHistoryEntry::from)
            .collect();

        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            approved_at,
            expires_at,
            created_at: credit_facility.created_at().into(),
            account_ids: credit_facility.account_ids,
            cvl_data: credit_facility.facility_cvl_data(),
            credit_facility_terms: TermValues::from(credit_facility.terms),
            status: credit_facility.status(),
            approvals,
            can_be_completed: credit_facility.can_be_completed(),
            transactions,
            facility_amount: credit_facility.initial_facility(),
            collateral: credit_facility.collateral(),
            collateralization_state: credit_facility.last_collateralization_state(),
            customer_id: UUID::from(credit_facility.customer_id),
        }
    }
}

impl From<crate::credit_facility::CreditFacility> for CreditFacilityCreatePayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
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

impl From<crate::credit_facility::CreditFacility> for CreditFacilityPartialPaymentPayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
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
    approvals: Vec<DisbursementApproval>,
    created_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DisbursementApproval {
    user_id: UUID,
    approved_at: Timestamp,
}

#[ComplexObject]
impl DisbursementApproval {
    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app
            .users()
            .find_by_id_internal(UserId::from(&self.user_id))
            .await?
            .expect("should always find user for a given UserId");
        Ok(User::from(user))
    }
}

impl From<crate::credit_facility::Disbursement> for CreditFacilityDisbursement {
    fn from(disbursement: crate::credit_facility::Disbursement) -> Self {
        let approvals = disbursement
            .approvals()
            .into_iter()
            .map(DisbursementApproval::from)
            .collect();
        Self {
            id: disbursement.id.to_global_id(),
            index: disbursement.idx,
            amount: disbursement.amount,
            approvals,
            status: disbursement.status(),
            created_at: disbursement.created_at().into(),
        }
    }
}

impl ToGlobalId for crate::primitives::DisbursementId {
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

impl From<crate::credit_facility::Disbursement> for CreditFacilityDisbursementInitiatePayload {
    fn from(disbursement: crate::credit_facility::Disbursement) -> Self {
        Self {
            disbursement: CreditFacilityDisbursement::from(disbursement),
        }
    }
}

impl From<crate::credit_facility::DisbursementApproval> for DisbursementApproval {
    fn from(disbursement_approval: crate::credit_facility::DisbursementApproval) -> Self {
        Self {
            user_id: UUID::from(disbursement_approval.user_id),
            approved_at: disbursement_approval.approved_at.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityDisbursementApproveInput {
    pub credit_facility_id: UUID,
    pub disbursement_idx: DisbursementIdx,
}

#[derive(SimpleObject)]
pub struct CreditFacilityDisbursementApprovePayload {
    disbursement: CreditFacilityDisbursement,
}

impl From<crate::credit_facility::Disbursement> for CreditFacilityDisbursementApprovePayload {
    fn from(disbursement: crate::credit_facility::Disbursement) -> Self {
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

impl From<crate::credit_facility::CreditFacility> for CreditFacilityCollateralUpdatePayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
}

pub use crate::credit_facility::CreditFacilityByCreatedAtCursor;
impl CursorType for CreditFacilityByCreatedAtCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(&self).expect("could not serialize token");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

impl From<crate::credit_facility::CreditFacilityApproval> for CreditFacilityApproval {
    fn from(approver: crate::credit_facility::CreditFacilityApproval) -> Self {
        CreditFacilityApproval {
            user_id: UUID::from(approver.user_id),
            approved_at: approver.approved_at.into(),
        }
    }
}

impl From<crate::credit_facility::CreditFacilityHistoryEntry> for CreditFacilityHistoryEntry {
    fn from(transaction: crate::credit_facility::CreditFacilityHistoryEntry) -> Self {
        match transaction {
            crate::credit_facility::CreditFacilityHistoryEntry::Payment(payment) => {
                CreditFacilityHistoryEntry::Payment(payment.into())
            }
            crate::credit_facility::CreditFacilityHistoryEntry::Collateral(collateral) => {
                CreditFacilityHistoryEntry::Collateral(collateral.into())
            }
            crate::credit_facility::CreditFacilityHistoryEntry::Origination(origination) => {
                CreditFacilityHistoryEntry::Origination(origination.into())
            }
            crate::credit_facility::CreditFacilityHistoryEntry::Collateralization(
                collateralization,
            ) => CreditFacilityHistoryEntry::Collateralization(collateralization.into()),
            crate::credit_facility::CreditFacilityHistoryEntry::Disbursement(disbursement) => {
                CreditFacilityHistoryEntry::Disbursement(disbursement.into())
            }
        }
    }
}

impl From<crate::credit_facility::IncrementalPayment> for CreditFacilityIncrementalPayment {
    fn from(payment: crate::credit_facility::IncrementalPayment) -> Self {
        Self {
            cents: payment.cents,
            recorded_at: payment.recorded_at.into(),
            tx_id: UUID::from(payment.tx_id),
        }
    }
}

impl From<crate::credit_facility::CollateralUpdated> for CreditFacilityCollateralUpdated {
    fn from(collateral: crate::credit_facility::CollateralUpdated) -> Self {
        Self {
            satoshis: collateral.satoshis,
            recorded_at: collateral.recorded_at.into(),
            action: collateral.action,
            tx_id: UUID::from(collateral.tx_id),
        }
    }
}

impl From<crate::credit_facility::CreditFacilityOrigination> for CreditFacilityOrigination {
    fn from(origination: crate::credit_facility::CreditFacilityOrigination) -> Self {
        Self {
            cents: origination.cents,
            recorded_at: origination.recorded_at.into(),
            tx_id: UUID::from(origination.tx_id),
        }
    }
}

impl From<crate::credit_facility::DisbursementExecuted> for CreditFacilityDisbursementExecuted {
    fn from(disbursement: crate::credit_facility::DisbursementExecuted) -> Self {
        Self {
            cents: disbursement.cents,
            recorded_at: disbursement.recorded_at.into(),
            tx_id: UUID::from(disbursement.tx_id),
        }
    }
}

impl From<crate::credit_facility::CollateralizationUpdated>
    for CreditFacilityCollateralizationUpdated
{
    fn from(collateralization: crate::credit_facility::CollateralizationUpdated) -> Self {
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

impl From<crate::credit_facility::FacilityCVL> for FacilityCVL {
    fn from(value: crate::credit_facility::FacilityCVL) -> Self {
        Self {
            total: value.total,
            disbursed: value.disbursed,
        }
    }
}
