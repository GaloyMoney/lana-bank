#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

use crate::DepositAccountId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CurrencyLedgerAccountIds {
    pub deposit_account_id: CalaAccountId,
    pub frozen_deposit_account_id: CalaAccountId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DepositAccountLedgerAccountIds {
    pub usd: Option<CurrencyLedgerAccountIds>,
    pub btc: Option<CurrencyLedgerAccountIds>,
}

impl DepositAccountLedgerAccountIds {
    pub fn new(account_id: DepositAccountId, usd: bool, btc: bool) -> Self {
        Self {
            usd: if usd {
                Some(CurrencyLedgerAccountIds {
                    deposit_account_id: account_id.into(),
                    frozen_deposit_account_id: CalaAccountId::new(),
                })
            } else {
                None
            },
            btc: if btc {
                Some(CurrencyLedgerAccountIds {
                    deposit_account_id: CalaAccountId::new(),
                    frozen_deposit_account_id: CalaAccountId::new(),
                })
            } else {
                None
            },
        }
    }
}
