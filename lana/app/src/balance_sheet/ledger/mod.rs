pub mod error;

use cala_ledger::{
    account_set::{AccountSet, AccountSetMemberId, AccountSetsByCreatedAtCursor, NewAccountSet},
    balance::error::BalanceError,
    AccountSetId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
};

use crate::statement::*;

use error::*;

use super::{
    BalanceSheet, BalanceSheetIds, ASSETS_NAME, EQUITY_NAME, LIABILITIES_NAME, NET_INCOME_NAME,
    NI_EXPENSES_NAME, NI_REVENUE_NAME,
};

#[derive(Clone)]
pub struct BalanceSheetLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl BalanceSheetLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        statement_id: impl Into<AccountSetId>,
        name: &str,
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = statement_id.into();
        let new_account_set = NewAccountSet::builder()
            .id(statement_id)
            .journal_id(self.journal_id)
            .name(name)
            .description(name)
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;

        let assets_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(assets_id)
            .journal_id(self.journal_id)
            .name(ASSETS_NAME)
            .description(ASSETS_NAME)
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, assets_id)
            .await?;

        let liabilities_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(liabilities_id)
            .journal_id(self.journal_id)
            .name(LIABILITIES_NAME)
            .description(LIABILITIES_NAME)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, liabilities_id)
            .await?;

        let equity_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(equity_id)
            .journal_id(self.journal_id)
            .name(EQUITY_NAME)
            .description(EQUITY_NAME)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, equity_id)
            .await?;

        let net_income_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(net_income_id)
            .journal_id(self.journal_id)
            .name(NET_INCOME_NAME)
            .description(NET_INCOME_NAME)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, equity_id, net_income_id)
            .await?;

        let revenue_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(revenue_id)
            .journal_id(self.journal_id)
            .name(NI_REVENUE_NAME)
            .description(NI_REVENUE_NAME)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, net_income_id, revenue_id)
            .await?;

        let expenses_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(expenses_id)
            .name(NI_EXPENSES_NAME)
            .journal_id(self.journal_id)
            .description(NI_EXPENSES_NAME)
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, net_income_id, expenses_id)
            .await?;

        op.commit().await?;

        Ok(BalanceSheetIds {
            id: statement_id,
            assets: assets_id,
            liabilities: liabilities_id,
            equity: equity_id,
            revenue: revenue_id,
            expenses: expenses_id,
        })
    }

    pub async fn list_for_name(
        &self,
        name: String,
        args: es_entity::PaginatedQueryArgs<AccountSetsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<AccountSet, AccountSetsByCreatedAtCursor>,
        BalanceSheetLedgerError,
    > {
        Ok(self.cala.account_sets().list_for_name(name, args).await?)
    }

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        node_account_set_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), BalanceSheetLedgerError> {
        let node_account_set_id = node_account_set_id.into();

        let mut op = self.cala.ledger_operation_from_db_op(op);
        match self
            .cala
            .account_sets()
            .add_member_in_op(&mut op, node_account_set_id, member)
            .await
        {
            Ok(_) | Err(cala_ledger::account_set::error::AccountSetError::MemberAlreadyAdded) => {}
            Err(e) => return Err(e.into()),
        }

        op.commit().await?;
        Ok(())
    }

    async fn get_account_set_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<StatementAccountSet, BalanceSheetLedgerError> {
        let id = id.into();

        let values = self.cala.account_sets().find(id).await?.into_values();

        let btc_currency =
            Currency::try_from("BTC".to_string()).expect("Cannot deserialize 'BTC' as Currency");
        let btc_balance = match self
            .cala
            .balances()
            .find_in_op(op, self.journal_id, id, btc_currency)
            .await
        {
            Ok(balance) => balance.try_into()?,
            Err(BalanceError::NotFound(_, _, _)) => BtcStatementAccountSetBalance::ZERO,
            Err(e) => return Err(e.into()),
        };

        let usd_currency =
            Currency::try_from("USD".to_string()).expect("Cannot deserialize 'USD' as Currency");
        let usd_balance = match self
            .cala
            .balances()
            .find_in_op(op, self.journal_id, id, usd_currency)
            .await
        {
            Ok(balance) => balance.try_into()?,
            Err(BalanceError::NotFound(_, _, _)) => UsdStatementAccountSetBalance::ZERO,
            Err(e) => return Err(e.into()),
        };

        Ok(StatementAccountSet {
            id: values.id,
            name: values.name,
            description: values.description,
            btc_balance,
            usd_balance,
        })
    }

    async fn get_account_set(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<StatementAccountSetDetails, BalanceSheetLedgerError> {
        let id = id.into();

        let values = self.cala.account_sets().find(id).await?.into_values();

        Ok(StatementAccountSetDetails {
            id: values.id,
            name: values.name,
            description: values.description,
        })
    }

    async fn get_member_account_sets_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<Vec<StatementAccountSet>, BalanceSheetLedgerError> {
        let id = id.into();

        let member_ids = self
            .cala
            .account_sets()
            .list_members_in_op(op, id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(BalanceSheetLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, BalanceSheetLedgerError>>()?;

        let mut accounts: Vec<StatementAccountSet> = vec![];
        for id in member_ids {
            accounts.push(self.get_account_set_in_op(op, id).await?);
        }

        Ok(accounts)
    }

    pub async fn get_member_account_sets(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<Vec<StatementAccountSetDetails>, BalanceSheetLedgerError> {
        let id = id.into();

        let member_ids = self
            .cala
            .account_sets()
            .list_members(id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(BalanceSheetLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, BalanceSheetLedgerError>>()?;

        let mut accounts: Vec<StatementAccountSetDetails> = vec![];
        for id in member_ids {
            accounts.push(self.get_account_set(id).await?);
        }

        Ok(accounts)
    }

    pub async fn get_balance_sheet(
        &self,
        ids: BalanceSheetIds,
    ) -> Result<BalanceSheet, BalanceSheetLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let balance_sheet_set = self.get_account_set_in_op(&mut op, ids.id).await?;

        let liabilities_account_set = self.get_account_set_in_op(&mut op, ids.liabilities).await?;
        let liabilities_accounts = self
            .get_member_account_sets_in_op(&mut op, ids.liabilities)
            .await?;

        let equity_account_set = self.get_account_set_in_op(&mut op, ids.equity).await?;
        let equity_accounts = self
            .get_member_account_sets_in_op(&mut op, ids.equity)
            .await?;

        let assets_account_set = self.get_account_set_in_op(&mut op, ids.assets).await?;
        let assets_accounts = self
            .get_member_account_sets_in_op(&mut op, ids.assets)
            .await?;

        op.commit().await?;

        Ok(BalanceSheet {
            id: balance_sheet_set.id,
            name: balance_sheet_set.name,
            description: balance_sheet_set.description,
            btc_balance: balance_sheet_set.btc_balance,
            usd_balance: balance_sheet_set.usd_balance,
            categories: vec![
                StatementAccountSetWithAccounts {
                    id: assets_account_set.id,
                    name: assets_account_set.name,
                    description: assets_account_set.description,
                    btc_balance: assets_account_set.btc_balance,
                    usd_balance: assets_account_set.usd_balance,
                    accounts: assets_accounts,
                },
                StatementAccountSetWithAccounts {
                    id: liabilities_account_set.id,
                    name: liabilities_account_set.name,
                    description: liabilities_account_set.description,
                    btc_balance: liabilities_account_set.btc_balance,
                    usd_balance: liabilities_account_set.usd_balance,
                    accounts: liabilities_accounts,
                },
                StatementAccountSetWithAccounts {
                    id: equity_account_set.id,
                    name: equity_account_set.name,
                    description: equity_account_set.description,
                    btc_balance: equity_account_set.btc_balance,
                    usd_balance: equity_account_set.usd_balance,
                    accounts: equity_accounts,
                },
            ],
        })
    }
}
