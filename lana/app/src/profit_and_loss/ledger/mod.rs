pub mod error;

use std::collections::HashMap;

use cala_ledger::{
    account_set::{AccountSetMemberId, NewAccountSet},
    AccountSetId, CalaLedger, DebitOrCredit, JournalId,
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
        reference: &str,
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(statement_id)
            .journal_id(self.journal_id)
            .external_id(reference)
            .name(reference)
            .description(reference)
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
            id: statement_id,
            revenue: revenue_id,
            expenses: expenses_id,
        })
    }

    async fn get_member_account_set_ids_and_names(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<HashMap<String, AccountSetId>, ProfitAndLossStatementLedgerError> {
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

        let mut accounts: HashMap<String, AccountSetId> = HashMap::new();
        for id in member_ids {
            let account_set = self.cala.account_sets().find(id).await?.into_values();
            accounts.insert(account_set.name, id);
        }

        Ok(accounts)
    }

    pub async fn get_ids_from_reference(
        &self,
        reference: String,
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementLedgerError> {
        let statement_id = self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id;

        let statement_members = self
            .get_member_account_set_ids_and_names(statement_id)
            .await?;
        let revenue_id = statement_members.get(REVENUE_NAME).ok_or(
            ProfitAndLossStatementLedgerError::NotFound(REVENUE_NAME.to_string()),
        )?;
        let expenses_id = statement_members.get(EXPENSES_NAME).ok_or(
            ProfitAndLossStatementLedgerError::NotFound(EXPENSES_NAME.to_string()),
        )?;

        Ok(ProfitAndLossStatementIds {
            id: statement_id,
            revenue: *revenue_id,
            expenses: *expenses_id,
        })
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

    async fn get_account_set(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &BalancesByAccount,
    ) -> Result<StatementAccountSet, ProfitAndLossStatementLedgerError> {
        let values = self
            .cala
            .account_sets()
            .find(account_set_id)
            .await?
            .into_values();

        Ok(StatementAccountSet {
            id: account_set_id,
            name: values.name,
            description: values.description,
            btc_balance: balances_by_id.btc_for_account(account_set_id)?,
            usd_balance: balances_by_id.usd_for_account(account_set_id)?,
        })
    }
    async fn get_member_account_set_ids(
        &self,
        account_set_id: AccountSetId,
    ) -> Result<Vec<AccountSetId>, ProfitAndLossStatementLedgerError> {
        self.cala
            .account_sets()
            .list_members(account_set_id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(ProfitAndLossStatementLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, ProfitAndLossStatementLedgerError>>()
    }

    pub async fn get_pl_statement(
        &self,
        reference: String,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementLedgerError> {
        let ids = self.get_ids_from_reference(reference).await?;
        let mut all_account_set_ids = vec![ids.id, ids.revenue, ids.expenses];

        let revenue_member_account_sets_ids = self.get_member_account_set_ids(ids.revenue).await?;
        all_account_set_ids.extend(&revenue_member_account_sets_ids);

        let expenses_member_account_sets_ids =
            self.get_member_account_set_ids(ids.expenses).await?;
        all_account_set_ids.extend(&expenses_member_account_sets_ids);

        let balance_ids =
            BalanceIdsForAccountSets::from((self.journal_id, all_account_set_ids)).balance_ids;
        let balances_by_id = self.cala.balances().find_all(&balance_ids).await?.into();

        let statement_account_set = self.get_account_set(ids.id, &balances_by_id).await?;
        let revenue_account_set = self.get_account_set(ids.revenue, &balances_by_id).await?;
        let expenses_account_set = self.get_account_set(ids.expenses, &balances_by_id).await?;

        let mut revenue_accounts = Vec::new();
        for account_set_id in revenue_member_account_sets_ids {
            revenue_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        let mut expenses_accounts = Vec::new();
        for account_set_id in expenses_member_account_sets_ids {
            expenses_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        Ok(ProfitAndLossStatement {
            id: statement_account_set.id,
            name: statement_account_set.name,
            description: statement_account_set.description,
            btc_balance: statement_account_set.btc_balance,
            usd_balance: statement_account_set.usd_balance,
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
