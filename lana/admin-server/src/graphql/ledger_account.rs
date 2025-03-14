use async_graphql::{connection::*, *};
use serde::{Deserialize, Serialize};

use lana_app::chart_of_accounts::AccountDetails;

use crate::primitives::*;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct LedgerAccount {
    id: UUID,
    name: String,
    code: AccountCode,
    // amounts: AccountAmountsByCurrency,
}

impl From<AccountDetails> for LedgerAccount {
    fn from(account: AccountDetails) -> Self {
        LedgerAccount {
            id: account.id.into(),
            name: account.name.to_string(),
            code: AccountCode(account.code.to_string()),
            // amounts: account.into(),
        }
    }
}

#[ComplexObject]
impl LedgerAccount {
    async fn history(
        &self,
        _ctx: &Context<'_>,
        _first: i32,
        _after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LedgerAccountHistoryCursor, LedgerAccountHistoryEntry, EmptyFields, EmptyFields>,
    > {
        unimplemented!()
    }
}

#[derive(Union)]
pub(super) enum LedgerAccountHistoryEntry {
    Usd(UsdLedgerAccountHistoryEntry),
    Btc(BtcLedgerAccountHistoryEntry),
}

#[derive(SimpleObject)]
pub(super) struct UsdLedgerAccountHistoryEntry {
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
    pub amount: LayeredUsdAccountAmounts,
}

#[derive(SimpleObject)]
pub(super) struct BtcLedgerAccountHistoryEntry {
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
    pub amount: LayeredBtcAccountAmounts,
}

#[derive(Serialize, Deserialize)]
pub(super) struct LedgerAccountHistoryCursor {
    pub value: String,
}

impl CursorType for LedgerAccountHistoryCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        self.value.clone()
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            value: s.to_string(),
        })
    }
}

scalar!(AccountCode);
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct AccountCode(String);

#[derive(SimpleObject)]
pub struct LayeredUsdAccountAmounts {
    settled: UsdAccountAmounts,
    pending: UsdAccountAmounts,
    encumbrance: UsdAccountAmounts,
}

#[derive(SimpleObject)]
pub struct LayeredBtcAccountAmounts {
    settled: BtcAccountAmounts,
    pending: BtcAccountAmounts,
    encumbrance: BtcAccountAmounts,
}

#[derive(SimpleObject)]
struct UsdAccountAmounts {
    debit: UsdCents,
    credit: UsdCents,
}

#[derive(SimpleObject)]
struct BtcAccountAmounts {
    debit: Satoshis,
    credit: Satoshis,
}
