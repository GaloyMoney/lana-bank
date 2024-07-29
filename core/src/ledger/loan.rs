use crate::primitives::{LedgerAccountId, Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::{cala::graphql::*, error::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LoanAccountIds {
    pub collateral_account_id: LedgerAccountId,
    pub outstanding_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
}

impl LoanAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collateral_account_id: LedgerAccountId::new(),
            outstanding_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
        }
    }
}

pub struct LoanBalance {
    pub collateral: Satoshis,
    pub outstanding: UsdCents,
    pub interest_incurred: UsdCents,
}

impl TryFrom<loan_balance::ResponseData> for LoanBalance {
    type Error = LedgerError;

    fn try_from(data: loan_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(LoanBalance {
            collateral: data
                .collateral
                .map(|b| Satoshis::try_from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(Satoshis::ZERO))?,
            outstanding: data
                .loan_outstanding
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            interest_incurred: data
                .interest_income
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
        })
    }
}
