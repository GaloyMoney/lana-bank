use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use crate::server::shared_graphql::primitives::{Satoshis, UsdCents};

#[derive(SimpleObject)]
struct BtcAccountBalance {
    debit: Satoshis,
    credit: Satoshis,
    net: Satoshis,
}

impl From<crate::ledger::account_ledger::BtcAccountBalance> for BtcAccountBalance {
    fn from(balance: crate::ledger::account_ledger::BtcAccountBalance) -> Self {
        BtcAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net: balance.net,
        }
    }
}

#[derive(SimpleObject)]
struct UsdAccountBalance {
    debit: UsdCents,
    credit: UsdCents,
    net: UsdCents,
}

impl From<crate::ledger::account_ledger::UsdAccountBalance> for UsdAccountBalance {
    fn from(balance: crate::ledger::account_ledger::UsdAccountBalance) -> Self {
        UsdAccountBalance {
            debit: balance.debit,
            credit: balance.credit,
            net: balance.net,
        }
    }
}

#[derive(SimpleObject)]
struct LayeredBtcAccountBalances {
    settled: BtcAccountBalance,
    pending: BtcAccountBalance,
    encumbrance: BtcAccountBalance,
}

impl From<crate::ledger::account_ledger::LayeredBtcAccountBalances> for LayeredBtcAccountBalances {
    fn from(balances: crate::ledger::account_ledger::LayeredBtcAccountBalances) -> Self {
        LayeredBtcAccountBalances {
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct LayeredUsdAccountBalances {
    settled: UsdAccountBalance,
    pending: UsdAccountBalance,
    encumbrance: UsdAccountBalance,
}

impl From<crate::ledger::account_ledger::LayeredUsdAccountBalances> for LayeredUsdAccountBalances {
    fn from(balances: crate::ledger::account_ledger::LayeredUsdAccountBalances) -> Self {
        LayeredUsdAccountBalances {
            settled: balances.settled.into(),
            pending: balances.pending.into(),
            encumbrance: balances.encumbrance.into(),
        }
    }
}

#[derive(SimpleObject)]
struct AccountBalancesByCurrency {
    btc: LayeredBtcAccountBalances,
    usd: LayeredUsdAccountBalances,
    usdt: LayeredUsdAccountBalances,
}

impl From<crate::ledger::account_ledger::AccountBalancesByCurrency> for AccountBalancesByCurrency {
    fn from(balances: crate::ledger::account_ledger::AccountBalancesByCurrency) -> Self {
        AccountBalancesByCurrency {
            btc: balances.btc.into(),
            usd: balances.usd.into(),
            usdt: balances.usdt.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountLedgerLineItem {
    cursor: String,
    name: String,
    total_balance: AccountBalancesByCurrency,
}

impl From<crate::ledger::account_ledger::AccountLedgerLineItemAndCursor> for AccountLedgerLineItem {
    fn from(
        item_and_cursor: crate::ledger::account_ledger::AccountLedgerLineItemAndCursor,
    ) -> Self {
        AccountLedgerLineItem {
            cursor: item_and_cursor.cursor,
            name: item_and_cursor.line_item.name,
            total_balance: item_and_cursor.line_item.total_balance.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct AccountLedgerLineItemCursor {
    pub value: String,
}

impl CursorType for AccountLedgerLineItemCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD_NO_PAD.encode(&self.value)
    }

    fn decode_cursor(cursor: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let decoded = general_purpose::STANDARD_NO_PAD
            .decode(cursor)
            .map_err(|e| e.to_string())?;
        let value = String::from_utf8(decoded).map_err(|e| e.to_string())?;
        Ok(AccountLedgerLineItemCursor { value })
    }
}

#[derive(SimpleObject)]
pub struct AccountLedgerSummary {
    name: String,
    total_balance: AccountBalancesByCurrency,
    line_item_balances:
        Connection<AccountLedgerLineItemCursor, AccountLedgerLineItem, EmptyFields, EmptyFields>,
}

fn create_line_item_connection(
    has_next_page: bool,
    has_previous_page: bool,
    nodes: Vec<AccountLedgerLineItem>,
) -> Connection<AccountLedgerLineItemCursor, AccountLedgerLineItem> {
    let mut connection = Connection::new(has_previous_page, has_next_page);

    connection.edges.extend(nodes.into_iter().map(|node| {
        let cursor = AccountLedgerLineItemCursor {
            value: node.cursor.clone(),
        };
        Edge::new(cursor, node)
    }));

    connection
}

impl From<crate::ledger::account_ledger::AccountLedgerSummary> for AccountLedgerSummary {
    fn from(account_ledger: crate::ledger::account_ledger::AccountLedgerSummary) -> Self {
        let nodes = account_ledger
            .line_item_balances
            .iter()
            .map(|l| AccountLedgerLineItem::from(l.clone()))
            .collect();

        AccountLedgerSummary {
            name: account_ledger.name,
            total_balance: account_ledger.total_balance.into(),
            line_item_balances: create_line_item_connection(
                account_ledger.has_next_page,
                account_ledger.has_previous_page,
                nodes,
            ),
        }
    }
}
