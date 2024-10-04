use crate::primitives::{LedgerAccountId, UsdCents};
use serde::{Deserialize, Serialize};

use super::{cala::graphql::*, error::*, primitives::LayeredUsdBalance};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CustomerLedgerAccountIds {
    pub on_balance_sheet_deposit_account_id: LedgerAccountId,
}

impl CustomerLedgerAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            on_balance_sheet_deposit_account_id: LedgerAccountId::new(),
        }
    }
}

pub struct CustomerBalance {
    pub usd_balance: LayeredUsdBalance,
}

impl TryFrom<customer_balance::ResponseData> for CustomerBalance {
    type Error = LedgerError;

    fn try_from(data: customer_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(CustomerBalance {
            usd_balance: LayeredUsdBalance {
                settled: data
                    .usd_balance
                    .clone()
                    .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                    .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
                pending: data
                    .usd_balance
                    .map(|b| UsdCents::try_from_usd(b.pending.normal_balance.units))
                    .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            },
        })
    }
}

impl CustomerBalance {
    pub fn check_withdraw_amount(self, amount: UsdCents) -> Result<(), LedgerError> {
        if self.usd_balance.settled < amount {
            return Err(LedgerError::InsufficientBalance(
                amount,
                self.usd_balance.settled,
            ));
        }

        Ok(())
    }
}
