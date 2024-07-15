use crate::primitives::{LedgerAccountId, Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::cala::graphql::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LoanAccountIds {
    pub collateral_account_id: LedgerAccountId,
    pub outstanding_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
    pub penalty_account_id: LedgerAccountId,
}

impl LoanAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collateral_account_id: LedgerAccountId::new(),
            outstanding_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
            penalty_account_id: LedgerAccountId::new(),
        }
    }
}

pub struct LoanBalance {
    pub collateral: Satoshis,
    pub outstanding: UsdCents,
    pub interest_incurred: UsdCents,
    pub penalty_incurred: UsdCents,
}

impl From<loan_balance::ResponseData> for LoanBalance {
    fn from(data: loan_balance::ResponseData) -> Self {
        LoanBalance {
            collateral: data
                .collateral
                .map(|b| Satoshis::from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Satoshis::ZERO),
            outstanding: data
                .loan_outstanding
                .map(|b| UsdCents::from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| UsdCents::ZERO),
            interest_incurred: data
                .interest_income
                .map(|b| UsdCents::from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| UsdCents::ZERO),
            penalty_incurred: data
                .penalty_income
                .map(|b| UsdCents::from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| UsdCents::ZERO),
        }
    }
}
