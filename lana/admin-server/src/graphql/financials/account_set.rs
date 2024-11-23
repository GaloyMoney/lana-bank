use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

use crate::{graphql::account::*, primitives::*};

use lana_app::app::LavaApp;

#[derive(SimpleObject)]
pub struct AccountSet {
    id: UUID,
    name: String,
    amounts: AccountAmountsByCurrency,
    has_sub_accounts: bool,
}

impl From<lana_app::ledger::account_set::LedgerAccountSetWithBalance> for AccountSet {
    fn from(line_item: lana_app::ledger::account_set::LedgerAccountSetWithBalance) -> Self {
        AccountSet {
            id: line_item.id.into(),
            name: line_item.name,
            amounts: line_item.balance.into(),
            has_sub_accounts: line_item.page_info.start_cursor.is_some(),
        }
    }
}

#[derive(Union)]
pub enum AccountSetSubAccount {
    Account(Account),
    AccountSet(AccountSet),
}

impl From<lana_app::ledger::account_set::PaginatedLedgerAccountSetSubAccountWithBalance>
    for AccountSetSubAccount
{
    fn from(
        member: lana_app::ledger::account_set::PaginatedLedgerAccountSetSubAccountWithBalance,
    ) -> Self {
        match member.value {
            lana_app::ledger::account_set::LedgerAccountSetSubAccountWithBalance::Account(val) => {
                AccountSetSubAccount::Account(Account::from(val))
            }
            lana_app::ledger::account_set::LedgerAccountSetSubAccountWithBalance::AccountSet(
                val,
            ) => AccountSetSubAccount::AccountSet(AccountSet::from(val)),
        }
    }
}

impl From<lana_app::ledger::account_set::LedgerAccountSetSubAccountWithBalance>
    for AccountSetSubAccount
{
    fn from(member: lana_app::ledger::account_set::LedgerAccountSetSubAccountWithBalance) -> Self {
        match member {
            lana_app::ledger::account_set::LedgerAccountSetSubAccountWithBalance::Account(val) => {
                AccountSetSubAccount::Account(Account::from(val))
            }
            lana_app::ledger::account_set::LedgerAccountSetSubAccountWithBalance::AccountSet(
                val,
            ) => AccountSetSubAccount::AccountSet(AccountSet::from(val)),
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct AccountSetAndSubAccounts {
    id: UUID,
    name: String,
    amounts: AccountAmountsByCurrency,
    #[graphql(skip)]
    from: DateTime<Utc>,
    #[graphql(skip)]
    until: Option<DateTime<Utc>>,
}

impl
    From<(
        DateTime<Utc>,
        Option<DateTime<Utc>>,
        lana_app::ledger::account_set::LedgerAccountSetAndSubAccountsWithBalance,
    )> for AccountSetAndSubAccounts
{
    fn from(
        (from, until, account_set): (
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            lana_app::ledger::account_set::LedgerAccountSetAndSubAccountsWithBalance,
        ),
    ) -> Self {
        AccountSetAndSubAccounts {
            id: account_set.id.into(),
            name: account_set.name,
            amounts: account_set.balance.into(),
            from,
            until,
        }
    }
}

#[ComplexObject]
impl AccountSetAndSubAccounts {
    async fn sub_accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> Result<Connection<SubAccountCursor, AccountSetSubAccount, EmptyFields, EmptyFields>> {
        let app = ctx.data_unchecked::<LavaApp>();
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .ledger()
                    .paginated_account_set_and_sub_accounts_with_balance(
                        uuid::Uuid::from(&self.id).into(),
                        self.from,
                        self.until,
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after
                                .map(lana_app::ledger::account_set::LedgerSubAccountCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|sub_account| {
                        let cursor = SubAccountCursor::from(sub_account.cursor.clone());
                        Edge::new(cursor, AccountSetSubAccount::from(sub_account))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct SubAccountCursor {
    pub value: String,
}

impl CursorType for SubAccountCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        self.value.clone()
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(SubAccountCursor {
            value: s.to_string(),
        })
    }
}

impl From<String> for SubAccountCursor {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<SubAccountCursor> for lana_app::ledger::account_set::LedgerSubAccountCursor {
    fn from(cursor: SubAccountCursor) -> Self {
        Self {
            value: cursor.value,
        }
    }
}
