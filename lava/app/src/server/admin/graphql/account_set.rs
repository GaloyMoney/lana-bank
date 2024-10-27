use async_graphql::{types::connection::*, *};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{app::LavaApp, server::shared_graphql::primitives::UUID};

use super::account::AccountAmountsByCurrency;

#[derive(SimpleObject)]
pub struct AccountSet {
    id: UUID,
    name: String,
    amounts: AccountAmountsByCurrency,
    has_sub_accounts: bool,
}

impl From<crate::ledger::account_set::LedgerAccountSetWithBalance> for AccountSet {
    fn from(line_item: crate::ledger::account_set::LedgerAccountSetWithBalance) -> Self {
        AccountSet {
            id: line_item.id.into(),
            name: line_item.name,
            amounts: line_item.balance.into(),
            has_sub_accounts: line_item.page_info.start_cursor.is_some(),
        }
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

impl From<SubAccountCursor> for crate::ledger::account_set::LedgerSubAccountCursor {
    fn from(cursor: SubAccountCursor) -> Self {
        Self {
            value: cursor.value,
        }
    }
}

#[derive(Union)]
enum AccountSetSubAccount {
    Account(super::account::Account),
    AccountSet(AccountSet),
}

impl From<crate::ledger::account_set::PaginatedLedgerAccountSetSubAccountWithBalance>
    for AccountSetSubAccount
{
    fn from(
        member: crate::ledger::account_set::PaginatedLedgerAccountSetSubAccountWithBalance,
    ) -> Self {
        match member.value {
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::Account(val) => {
                AccountSetSubAccount::Account(super::account::Account::from(val))
            }
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::AccountSet(val) => {
                AccountSetSubAccount::AccountSet(AccountSet::from(val))
            }
        }
    }
}

impl From<crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance>
    for AccountSetSubAccount
{
    fn from(member: crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance) -> Self {
        match member {
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::Account(val) => {
                AccountSetSubAccount::Account(super::account::Account::from(val))
            }
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::AccountSet(val) => {
                AccountSetSubAccount::AccountSet(AccountSet::from(val))
            }
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
        crate::ledger::account_set::LedgerAccountSetAndSubAccountsWithBalance,
    )> for AccountSetAndSubAccounts
{
    fn from(
        (from, until, account_set): (
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            crate::ledger::account_set::LedgerAccountSetAndSubAccountsWithBalance,
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
                        self.id.clone().into(),
                        self.from,
                        self.until,
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after
                                .map(crate::ledger::account_set::LedgerSubAccountCursor::from),
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
#[derive(SimpleObject)]
pub struct TrialBalance {
    name: String,
    total: AccountAmountsByCurrency,
    sub_accounts: Vec<AccountSetSubAccount>,
}

impl From<crate::ledger::account_set::LedgerTrialBalance> for TrialBalance {
    fn from(trial_balance: crate::ledger::account_set::LedgerTrialBalance) -> Self {
        TrialBalance {
            name: trial_balance.name,
            total: trial_balance.balance.into(),
            sub_accounts: trial_balance
                .accounts
                .into_iter()
                .map(AccountSetSubAccount::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartOfAccounts {
    name: String,
    categories: Vec<StatementCategory>,
}

impl From<crate::ledger::account_set::LedgerChartOfAccounts> for ChartOfAccounts {
    fn from(chart_of_accounts: crate::ledger::account_set::LedgerChartOfAccounts) -> Self {
        ChartOfAccounts {
            name: chart_of_accounts.name,
            categories: chart_of_accounts
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct StatementCategory {
    name: String,
    amounts: AccountAmountsByCurrency,
    accounts: Vec<AccountSetSubAccount>,
}

impl From<crate::ledger::account_set::LedgerStatementCategoryWithBalance> for StatementCategory {
    fn from(account_set: crate::ledger::account_set::LedgerStatementCategoryWithBalance) -> Self {
        StatementCategory {
            name: account_set.name,
            amounts: account_set.balance.into(),
            accounts: account_set
                .accounts
                .into_iter()
                .map(AccountSetSubAccount::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct BalanceSheet {
    name: String,
    balance: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<crate::ledger::account_set::LedgerBalanceSheet> for BalanceSheet {
    fn from(balance_sheet: crate::ledger::account_set::LedgerBalanceSheet) -> Self {
        BalanceSheet {
            name: balance_sheet.name,
            balance: balance_sheet.balance.into(),
            categories: balance_sheet
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ProfitAndLossStatement {
    name: String,
    net: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<crate::ledger::account_set::LedgerProfitAndLossStatement> for ProfitAndLossStatement {
    fn from(profit_and_loss: crate::ledger::account_set::LedgerProfitAndLossStatement) -> Self {
        ProfitAndLossStatement {
            name: profit_and_loss.name,
            net: profit_and_loss.balance.into(),
            categories: profit_and_loss
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct CashFlowStatement {
    name: String,
    total: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<crate::ledger::account_set::LedgerCashFlowStatement> for CashFlowStatement {
    fn from(profit_and_loss: crate::ledger::account_set::LedgerCashFlowStatement) -> Self {
        CashFlowStatement {
            name: profit_and_loss.name,
            total: profit_and_loss.balance.into(),
            categories: profit_and_loss
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
