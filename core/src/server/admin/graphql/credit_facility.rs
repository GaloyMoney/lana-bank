use async_graphql::*;
use connection::CursorType;

use crate::{
    app::LavaApp,
    ledger,
    primitives::{Satoshis, UsdCents},
    server::shared_graphql::{
        convert::ToGlobalId,
        objects::{Collateral, Outstanding},
        primitives::UUID,
        terms::*,
    },
    terms::CollateralizationState,
};

pub use crate::primitives::DisbursementIdx;

scalar!(DisbursementIdx);

#[derive(SimpleObject)]
pub(super) struct CreditFacilityBalance {
    outstanding: Outstanding,
    collateral: Collateral,
}

impl From<ledger::credit_facility::CreditFacilityBalance> for CreditFacilityBalance {
    fn from(balance: ledger::credit_facility::CreditFacilityBalance) -> Self {
        Self {
            outstanding: Outstanding {
                usd_balance: balance.disbursed_receivable + balance.interest_receivable,
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
    collateralization_state: CollateralizationState,
    #[graphql(skip)]
    account_ids: crate::ledger::credit_facility::CreditFacilityAccountIds,
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
        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
            account_ids: credit_facility.account_ids,
            collateralization_state: credit_facility.collateralization(),
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
}

impl From<crate::credit_facility::Disbursement> for CreditFacilityDisbursement {
    fn from(disbursement: crate::credit_facility::Disbursement) -> Self {
        Self {
            id: disbursement.id.to_global_id(),
            index: disbursement.idx,
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
