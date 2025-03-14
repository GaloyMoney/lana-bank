use async_graphql::{connection::*, *};
use serde::{Deserialize, Serialize};

use lana_app::{
    chart_of_accounts::AccountDetails,
    ledger_account::{
        LayeredLedgerAccountAmount as DomainLayeredLedgerAccountAmount,
        LedgerAccountEntry as DomainLedgerAccountEntry, LedgerAccountHistoryCursor,
    },
};

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
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LedgerAccountHistoryCursor, LedgerAccountHistoryEntry, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let query_args = es_entity::PaginatedQueryArgs { first, after };
                let res = app
                    .ledger_accounts()
                    .history(sub, self.id, query_args)
                    .await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entry| {
                        let cursor = LedgerAccountHistoryCursor::from(&entry);
                        Edge::new(cursor, LedgerAccountHistoryEntry::from(entry))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

#[derive(Union)]
pub(super) enum LedgerAccountHistoryEntry {
    Usd(UsdLedgerAccountHistoryEntry),
    Btc(BtcLedgerAccountHistoryEntry),
}

impl From<DomainLedgerAccountEntry> for LedgerAccountHistoryEntry {
    fn from(entry: DomainLedgerAccountEntry) -> Self {
        match entry.amount {
            DomainLayeredLedgerAccountAmount::Usd(_) => Self::Usd(entry.into()),
            DomainLayeredLedgerAccountAmount::Btc(_) => Self::Btc(entry.into()),
        }
    }
}

#[derive(SimpleObject)]
pub(super) struct UsdLedgerAccountHistoryEntry {
    pub entry_id: UUID,
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
    pub usd_amount: LayeredUsdAccountAmounts,
}

impl From<DomainLedgerAccountEntry> for UsdLedgerAccountHistoryEntry {
    fn from(entry: DomainLedgerAccountEntry) -> Self {
        Self {
            entry_id: entry.entry_id.into(),
            tx_id: entry.tx_id.into(),
            recorded_at: entry.recorded_at.into(),
            amount: match entry.amount {
                DomainLayeredLedgerAccountAmount::Usd(amount) => amount.into(),
                DomainLayeredLedgerAccountAmount::Btc(_) => {
                    panic!("Uexpected currency for USD entry")
                }
            },
        }
    }
}

#[derive(SimpleObject)]
pub(super) struct BtcLedgerAccountHistoryEntry {
    pub entry_id: UUID,
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
    pub btc_amount: LayeredBtcAccountAmounts,
}

impl From<DomainLedgerAccountEntry> for BtcLedgerAccountHistoryEntry {
    fn from(entry: DomainLedgerAccountEntry) -> Self {
        Self {
            entry_id: entry.entry_id.into(),
            tx_id: entry.tx_id.into(),
            recorded_at: entry.recorded_at.into(),
            amount: match entry.amount {
                DomainLayeredLedgerAccountAmount::Btc(amount) => amount.into(),
                DomainLayeredLedgerAccountAmount::Usd(_) => {
                    panic!("Uexpected currency for BTC entry")
                }
            },
        }
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

impl From<lana_app::ledger_account::LayeredUsdLedgerAccountAmount> for LayeredUsdAccountAmounts {
    fn from(amount: lana_app::ledger_account::LayeredUsdLedgerAccountAmount) -> Self {
        Self {
            settled: amount.settled.into(),
            pending: amount.pending.into(),
            encumbrance: amount.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct LayeredBtcAccountAmounts {
    settled: BtcAccountAmounts,
    pending: BtcAccountAmounts,
    encumbrance: BtcAccountAmounts,
}

impl From<lana_app::ledger_account::LayeredBtcLedgerAccountAmount> for LayeredBtcAccountAmounts {
    fn from(amount: lana_app::ledger_account::LayeredBtcLedgerAccountAmount) -> Self {
        Self {
            settled: amount.settled.into(),
            pending: amount.pending.into(),
            encumbrance: amount.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountAmounts {
    debit: UsdCents,
    credit: UsdCents,
}

impl From<lana_app::ledger_account::UsdLedgerAccountAmount> for UsdAccountAmounts {
    fn from(amount: lana_app::ledger_account::UsdLedgerAccountAmount) -> Self {
        Self {
            debit: amount.dr_amount,
            credit: amount.cr_amount,
        }
    }
}

#[derive(SimpleObject)]
struct BtcAccountAmounts {
    debit: Satoshis,
    credit: Satoshis,
}

impl From<lana_app::ledger_account::BtcLedgerAccountAmount> for BtcAccountAmounts {
    fn from(amount: lana_app::ledger_account::BtcLedgerAccountAmount) -> Self {
        Self {
            debit: amount.dr_amount,
            credit: amount.cr_amount,
        }
    }
}
