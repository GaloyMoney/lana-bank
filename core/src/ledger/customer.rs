use crate::primitives::{LedgerAccountId, Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::{
    cala::{error::*, graphql::*},
    primitives::LayeredUsdBalance,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CustomerLedgerAccountIds {
    pub off_balance_sheet_deposit_account_id: LedgerAccountId,
    pub on_balance_sheet_deposit_account_id: LedgerAccountId,
}

impl CustomerLedgerAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            off_balance_sheet_deposit_account_id: LedgerAccountId::new(),
            on_balance_sheet_deposit_account_id: LedgerAccountId::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerLedgerAccountAddresses {
    pub tron_usdt_address: String,
    pub btc_address: String,
}

pub struct CustomerBalance {
    pub btc_balance: Satoshis,
    pub usdt_balance: LayeredUsdBalance,
}

impl TryFrom<customer_balance::ResponseData> for CustomerBalance {
    type Error = CalaError;

    fn try_from(data: customer_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(CustomerBalance {
            btc_balance: data
                .btc_balance
                .map(|b| Satoshis::try_from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(Satoshis::ZERO))?,
            usdt_balance: LayeredUsdBalance {
                settled: data
                    .usdt_balance
                    .clone()
                    .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                    .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
                pending: data
                    .usdt_balance
                    .map(|b| UsdCents::try_from_usd(b.pending.normal_balance.units))
                    .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            },
        })
    }
}
