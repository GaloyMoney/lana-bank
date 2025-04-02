use async_graphql::{connection::*, *};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use lana_app::accounting::journal::JournalEntryCursor;
use lana_app::accounting::ledger_account::LedgerAccount as DomainLedgerAccount;
use lana_app::accounting::AccountCode as DomainAccountCode;
use lana_app::primitives::Currency;

use crate::primitives::*;

use super::JournalEntry;

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct LedgerAccount {
    id: UUID,
    code: Option<AccountCode>,

    #[graphql(skip)]
    pub entity: Arc<DomainLedgerAccount>,
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

    async fn balance(&self, _ctx: &Context<'_>) -> async_graphql::Result<LedgerAccountBalance> {
        if let Some(balance) = self.entity.btc_balance.as_ref() {
            Ok(Some(balance).into())
        } else {
            Ok(self.entity.usd_balance.as_ref().into())
        }
    }

    async fn history(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<JournalEntryCursor, JournalEntry, EmptyFields, EmptyFields>>
    {
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
                        let cursor = JournalEntryCursor::from(&entry);
                        Edge::new(cursor, JournalEntry::from(entry))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
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
            Some(balance) if balance.details.currency == Currency::USD => {
                LedgerAccountBalance::Usd(UsdLedgerAccountBalance {
                    settled: UsdCents::try_from_usd(balance.settled()).expect("positive"),
                    pending: UsdCents::try_from_usd(balance.pending()).expect("positive"),
                    encumbrance: UsdCents::try_from_usd(balance.encumbrance()).expect("positive"),
                })
            }
            Some(balance) if balance.details.currency == Currency::BTC => {
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

scalar!(AccountCode);
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AccountCode(String);

impl From<&DomainAccountCode> for AccountCode {
    fn from(value: &DomainAccountCode) -> Self {
        AccountCode(value.to_string())
    }
}
