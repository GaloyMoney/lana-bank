pub mod error;

use cala_ledger::{
    account_set::{AccountSet, AccountSetMemberId, AccountSetsByCreatedAtCursor, NewAccountSet},
    balance::error::BalanceError,
    AccountSetId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
};

use crate::statement::*;

use error::*;

use super::{ProfitAndLossStatement, ProfitAndLossStatementIds, EXPENSES_NAME, REVENUE_NAME};

#[derive(Clone)]
pub struct ProfitAndLossStatementLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl ProfitAndLossStatementLedger {
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
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = statement_id.into();
        let new_account_set = NewAccountSet::builder()
            .id(statement_id)
            .journal_id(self.journal_id)
            .name(name)
            .description(name)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;

        let revenue_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(revenue_id)
            .journal_id(self.journal_id)
            .name(REVENUE_NAME)
            .description(REVENUE_NAME)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, revenue_id)
            .await?;

        let expenses_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(expenses_id)
            .journal_id(self.journal_id)
            .name(EXPENSES_NAME)
            .description(EXPENSES_NAME)
            .normal_balance_type(DebitOrCredit::Credit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, expenses_id)
            .await?;

        op.commit().await?;

        Ok(ProfitAndLossStatementIds {
            id: statement_id.into(),
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
        ProfitAndLossStatementLedgerError,
    > {
        Ok(self.cala.account_sets().list_for_name(name, args).await?)
    }

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        node_account_set_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), ProfitAndLossStatementLedgerError> {
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
    ) -> Result<StatementAccountSet, ProfitAndLossStatementLedgerError> {
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
    ) -> Result<StatementAccountSetDetails, ProfitAndLossStatementLedgerError> {
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
    ) -> Result<Vec<StatementAccountSet>, ProfitAndLossStatementLedgerError> {
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
                _ => Err(ProfitAndLossStatementLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, ProfitAndLossStatementLedgerError>>()?;

        let mut accounts: Vec<StatementAccountSet> = vec![];
        for id in member_ids {
            accounts.push(self.get_account_set_in_op(op, id).await?);
        }

        Ok(accounts)
    }

    pub async fn get_member_account_sets(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<Vec<StatementAccountSetDetails>, ProfitAndLossStatementLedgerError> {
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
                _ => Err(ProfitAndLossStatementLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, ProfitAndLossStatementLedgerError>>()?;

        let mut accounts: Vec<StatementAccountSetDetails> = vec![];
        for id in member_ids {
            accounts.push(self.get_account_set(id).await?);
        }

        Ok(accounts)
    }

    pub async fn get_pl_statement(
        &self,
        ids: ProfitAndLossStatementIds,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let pl_statement_set = self.get_account_set_in_op(&mut op, ids.id).await?;

        let revenue_account_set = self.get_account_set_in_op(&mut op, ids.revenue).await?;
        let revenue_accounts = self
            .get_member_account_sets_in_op(&mut op, ids.revenue)
            .await?;

        let expenses_account_set = self.get_account_set_in_op(&mut op, ids.expenses).await?;
        let expenses_accounts = self
            .get_member_account_sets_in_op(&mut op, ids.expenses)
            .await?;

        op.commit().await?;

        Ok(ProfitAndLossStatement {
            id: pl_statement_set.id.into(),
            name: pl_statement_set.name,
            description: pl_statement_set.description,
            btc_balance: pl_statement_set.btc_balance,
            usd_balance: pl_statement_set.usd_balance,
            categories: vec![
                StatementAccountSetWithAccounts {
                    id: revenue_account_set.id,
                    name: revenue_account_set.name,
                    description: revenue_account_set.description,
                    btc_balance: revenue_account_set.btc_balance,
                    usd_balance: revenue_account_set.usd_balance,
                    accounts: revenue_accounts,
                },
                StatementAccountSetWithAccounts {
                    id: expenses_account_set.id,
                    name: expenses_account_set.name,
                    description: expenses_account_set.description,
                    btc_balance: expenses_account_set.btc_balance,
                    usd_balance: expenses_account_set.usd_balance,
                    accounts: expenses_accounts,
                },
            ],
        })
    }
}
