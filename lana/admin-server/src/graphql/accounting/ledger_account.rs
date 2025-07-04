use async_graphql::{connection::*, *};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use lana_app::accounting::{
    AccountCode as DomainAccountCode, AccountCodeSection as DomainAccountCodeSection,
    journal::JournalEntryCursor, ledger_account::LedgerAccount as DomainLedgerAccount,
};
use lana_app::primitives::Currency;

use crate::{graphql::loader::*, primitives::*};

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

    async fn balance_range(&self) -> async_graphql::Result<LedgerAccountBalanceRange> {
        if let Some(balance) = self.entity.btc_balance_range.as_ref() {
            Ok(Some(balance).into())
        } else {
            Ok(self.entity.usd_balance_range.as_ref().into())
        }
    }

    async fn ancestors(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<LedgerAccount>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let mut ancestors = loader.load_many(self.entity.ancestor_ids.clone()).await?;

        let mut result = Vec::with_capacity(self.entity.ancestor_ids.len());

        for id in self.entity.ancestor_ids.iter() {
            if let Some(account) = ancestors.remove(id) {
                result.push(account);
            }
        }

        Ok(result)
    }

    async fn closest_account_with_code(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<LedgerAccount>> {
        if self.code.is_some() {
            return Ok(Some(self.clone()));
        }

        let ancestors = self.ancestors(ctx).await?;
        let closest = ancestors.into_iter().find(|a| a.code.is_some());

        Ok(closest)
    }

    async fn children(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<LedgerAccount>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let mut children = loader.load_many(self.entity.children_ids.clone()).await?;

        let mut result = Vec::with_capacity(self.entity.children_ids.len());

        for id in self.entity.children_ids.iter() {
            if let Some(account) = children.remove(id) {
                result.push(account);
            }
        }

        Ok(result)
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
pub(super) enum LedgerAccountBalanceRange {
    Usd(UsdLedgerAccountBalanceRange),
    Btc(BtcLedgerAccountBalanceRange),
}

#[derive(SimpleObject)]
pub(super) struct LedgerAccountBalanceRangeByCurrency {
    pub usd: UsdLedgerAccountBalanceRange,
    pub btc: BtcLedgerAccountBalanceRange,
}

impl From<Option<&lana_app::primitives::BalanceRange>> for LedgerAccountBalanceRange {
    fn from(balance_range_opt: Option<&lana_app::primitives::BalanceRange>) -> Self {
        match balance_range_opt {
            None => LedgerAccountBalanceRange::Usd(UsdLedgerAccountBalanceRange::default()),
            Some(balance_range) => {
                let currency = match &balance_range.close {
                    None => Currency::USD,
                    Some(balance) if balance.details.currency == Currency::USD => Currency::USD,
                    Some(balance) if balance.details.currency == Currency::BTC => Currency::BTC,
                    _ => unimplemented!("unexpected currency"),
                };

                if currency == Currency::USD {
                    LedgerAccountBalanceRange::Usd(UsdLedgerAccountBalanceRange::from(
                        balance_range,
                    ))
                } else {
                    LedgerAccountBalanceRange::Btc(BtcLedgerAccountBalanceRange::from(
                        balance_range,
                    ))
                }
            }
        }
    }
}

#[derive(SimpleObject, Default)]
pub(super) struct UsdLedgerAccountBalanceRange {
    open: UsdLedgerAccountBalance,
    period_activity: UsdLedgerAccountBalance,
    close: UsdLedgerAccountBalance,
}

impl From<&lana_app::primitives::BalanceRange> for UsdLedgerAccountBalanceRange {
    fn from(balance_range: &lana_app::primitives::BalanceRange) -> Self {
        Self {
            open: UsdLedgerAccountBalance::from(balance_range.open.as_ref()),
            period_activity: UsdLedgerAccountBalance::from(balance_range.period_activity.as_ref()),
            close: UsdLedgerAccountBalance::from(balance_range.close.as_ref()),
        }
    }
}

#[derive(SimpleObject, Default)]
pub(super) struct BtcLedgerAccountBalanceRange {
    open: BtcLedgerAccountBalance,
    period_activity: BtcLedgerAccountBalance,
    close: BtcLedgerAccountBalance,
}

impl From<&lana_app::primitives::BalanceRange> for BtcLedgerAccountBalanceRange {
    fn from(balance_range: &lana_app::primitives::BalanceRange) -> Self {
        Self {
            open: BtcLedgerAccountBalance::from(balance_range.open.as_ref()),
            period_activity: BtcLedgerAccountBalance::from(balance_range.period_activity.as_ref()),
            close: BtcLedgerAccountBalance::from(balance_range.close.as_ref()),
        }
    }
}

#[derive(SimpleObject, Default)]
pub(super) struct UsdLedgerAccountBalance {
    settled: UsdBalanceDetails,
    pending: UsdBalanceDetails,
    encumbrance: UsdBalanceDetails,
}

impl From<Option<&cala_ledger::balance::AccountBalance>> for UsdLedgerAccountBalance {
    fn from(balance: Option<&cala_ledger::balance::AccountBalance>) -> Self {
        match balance {
            None => UsdLedgerAccountBalance {
                settled: UsdBalanceDetails::default(),
                pending: UsdBalanceDetails::default(),
                encumbrance: UsdBalanceDetails::default(),
            },
            Some(balance) => UsdLedgerAccountBalance {
                settled: UsdBalanceDetails {
                    debit: UsdCents::try_from_usd(balance.details.settled.dr_balance)
                        .expect("positive"),
                    credit: UsdCents::try_from_usd(balance.details.settled.cr_balance)
                        .expect("positive"),
                    net: SignedUsdCents::from_usd(balance.settled()),
                },
                pending: UsdBalanceDetails {
                    debit: UsdCents::try_from_usd(balance.details.pending.dr_balance)
                        .expect("positive"),
                    credit: UsdCents::try_from_usd(balance.details.pending.cr_balance)
                        .expect("positive"),
                    net: SignedUsdCents::from_usd(balance.pending()),
                },
                encumbrance: UsdBalanceDetails {
                    debit: UsdCents::try_from_usd(balance.details.encumbrance.dr_balance)
                        .expect("positive"),
                    credit: UsdCents::try_from_usd(balance.details.encumbrance.cr_balance)
                        .expect("positive"),
                    net: SignedUsdCents::from_usd(balance.encumbrance()),
                },
            },
        }
    }
}

#[derive(SimpleObject, Default)]
struct UsdBalanceDetails {
    debit: UsdCents,
    credit: UsdCents,
    net: SignedUsdCents,
}

#[derive(SimpleObject, Default)]
pub(super) struct BtcLedgerAccountBalance {
    settled: BtcBalanceDetails,
    pending: BtcBalanceDetails,
    encumbrance: BtcBalanceDetails,
}

impl From<Option<&cala_ledger::balance::AccountBalance>> for BtcLedgerAccountBalance {
    fn from(balance: Option<&cala_ledger::balance::AccountBalance>) -> Self {
        match balance {
            None => BtcLedgerAccountBalance {
                settled: BtcBalanceDetails::default(),
                pending: BtcBalanceDetails::default(),
                encumbrance: BtcBalanceDetails::default(),
            },
            Some(balance) => BtcLedgerAccountBalance {
                settled: BtcBalanceDetails {
                    debit: Satoshis::try_from_btc(balance.details.settled.dr_balance)
                        .expect("positive"),
                    credit: Satoshis::try_from_btc(balance.details.settled.cr_balance)
                        .expect("positive"),
                    net: SignedSatoshis::from_btc(balance.settled()),
                },
                pending: BtcBalanceDetails {
                    debit: Satoshis::try_from_btc(balance.details.pending.dr_balance)
                        .expect("positive"),
                    credit: Satoshis::try_from_btc(balance.details.pending.cr_balance)
                        .expect("positive"),
                    net: SignedSatoshis::from_btc(balance.pending()),
                },
                encumbrance: BtcBalanceDetails {
                    debit: Satoshis::try_from_btc(balance.details.encumbrance.dr_balance)
                        .expect("positive"),
                    credit: Satoshis::try_from_btc(balance.details.encumbrance.cr_balance)
                        .expect("positive"),
                    net: SignedSatoshis::from_btc(balance.encumbrance()),
                },
            },
        }
    }
}

#[derive(SimpleObject, Default)]
struct BtcBalanceDetails {
    debit: Satoshis,
    credit: Satoshis,
    net: SignedSatoshis,
}

scalar!(AccountCode);
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AccountCode(String);

impl From<&DomainAccountCode> for AccountCode {
    fn from(value: &DomainAccountCode) -> Self {
        AccountCode(value.to_string())
    }
}

impl TryFrom<AccountCode> for DomainAccountCode {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(value: AccountCode) -> Result<Self, Self::Error> {
        Ok(value.0.parse()?)
    }
}

impl TryFrom<AccountCode> for Vec<DomainAccountCodeSection> {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(value: AccountCode) -> Result<Self, Self::Error> {
        Ok(Self::from(DomainAccountCode::try_from(value)?))
    }
}
