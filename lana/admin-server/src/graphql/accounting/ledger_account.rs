use async_graphql::{connection::*, *};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use lana_app::accounting::{
    ledger_account::{
        LayeredLedgerAccountAmount as DomainLayeredLedgerAccountAmount,
        LedgerAccount as DomainLedgerAccount, LedgerAccountEntry as DomainLedgerAccountEntry,
        LedgerAccountHistoryCursor,
    },
    AccountCode as DomainAccountCode, AccountDetails,
};

use crate::primitives::*;

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct LedgerAccount {
    id: UUID,
    code: Option<AccountCode>,

    #[graphql(skip)]
    pub entity: Arc<DomainLedgerAccount>,
}

impl From<AccountDetails> for LedgerAccount {
    fn from(account: AccountDetails) -> Self {
        // LedgerAccount {
        //     id: account.id.into(),
        //     name: account.name.to_string(),
        //     code: AccountCode(account.code.to_string()),
        // }
        unimplemented!()
    }
}

impl From<DomainLedgerAccount> for LedgerAccount {
    fn from(account: DomainLedgerAccount) -> Self {
        LedgerAccount {
            id: account.id.into(),
            code: account.code.as_ref().map(|code| code.into()),
            entity: Arc::new(account),
        }
    }
}

#[ComplexObject]
impl LedgerAccount {
    async fn name(&self) -> &str {
        &self.entity.name
    }

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
                    .accounting()
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

    async fn balance(&self, _ctx: &Context<'_>) -> async_graphql::Result<LedgerAccountBalance> {
        if let Some(balance) = self.entity.btc_balance.as_ref() {
            Ok(Some(balance).into())
        } else {
            Ok(self.entity.usd_balance.as_ref().into())
        }
    }
}

#[derive(Union)]
pub(super) enum LedgerAccountBalance {
    Usd(UsdLedgerAccountBalance),
    Btc(BtcLedgerAccountBalance),
}

impl From<Option<&cala_ledger::balance::AccountBalance>> for LedgerAccountBalance {
    fn from(balance: Option<&cala_ledger::balance::AccountBalance>) -> Self {
        match balance {
            None => LedgerAccountBalance::Usd(UsdLedgerAccountBalance {
                settled: UsdCents::ZERO,
                pending: UsdCents::ZERO,
                encumbrance: UsdCents::ZERO,
            }),
            Some(balance) if balance.details.currency == "USD".parse().unwrap() => {
                LedgerAccountBalance::Usd(UsdLedgerAccountBalance {
                    settled: UsdCents::try_from_usd(balance.settled()).expect("positive"),
                    pending: UsdCents::try_from_usd(balance.pending()).expect("positive"),
                    encumbrance: UsdCents::try_from_usd(balance.encumbrance()).expect("positive"),
                })
            }
            Some(balance) if balance.details.currency == "BTC".parse().unwrap() => {
                LedgerAccountBalance::Btc(BtcLedgerAccountBalance {
                    settled: Satoshis::try_from_btc(balance.settled()).expect("positive"),
                    pending: Satoshis::try_from_btc(balance.pending()).expect("positive"),
                    encumbrance: Satoshis::try_from_btc(balance.encumbrance()).expect("positive"),
                })
            }
            _ => unimplemented!("Unexpected currency"),
        }
    }
}

#[derive(SimpleObject)]
pub(super) struct UsdLedgerAccountBalance {
    settled: UsdCents,
    pending: UsdCents,
    encumbrance: UsdCents,
}

#[derive(SimpleObject)]
pub(super) struct BtcLedgerAccountBalance {
    settled: Satoshis,
    pending: Satoshis,
    encumbrance: Satoshis,
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
            usd_amount: match entry.amount {
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
            btc_amount: match entry.amount {
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
pub struct AccountCode(String);

impl From<&DomainAccountCode> for AccountCode {
    fn from(value: &DomainAccountCode) -> Self {
        AccountCode(value.to_string())
    }
}

#[derive(SimpleObject)]
pub struct LayeredUsdAccountAmounts {
    settled: UsdAccountAmounts,
    pending: UsdAccountAmounts,
    encumbrance: UsdAccountAmounts,
}

impl From<lana_app::accounting::ledger_account::LayeredUsdLedgerAccountAmount>
    for LayeredUsdAccountAmounts
{
    fn from(amount: lana_app::accounting::ledger_account::LayeredUsdLedgerAccountAmount) -> Self {
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

impl From<lana_app::accounting::ledger_account::LayeredBtcLedgerAccountAmount>
    for LayeredBtcAccountAmounts
{
    fn from(amount: lana_app::accounting::ledger_account::LayeredBtcLedgerAccountAmount) -> Self {
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

impl From<lana_app::accounting::ledger_account::UsdLedgerAccountAmount> for UsdAccountAmounts {
    fn from(amount: lana_app::accounting::ledger_account::UsdLedgerAccountAmount) -> Self {
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

impl From<lana_app::accounting::ledger_account::BtcLedgerAccountAmount> for BtcAccountAmounts {
    fn from(amount: lana_app::accounting::ledger_account::BtcLedgerAccountAmount) -> Self {
        Self {
            debit: amount.dr_amount,
            credit: amount.cr_amount,
        }
    }
}
