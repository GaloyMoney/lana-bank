pub mod error;

use cala_ledger::{
    account_set::{AccountSetMemberId, NewAccountSet},
    AccountSetId, BalanceId, CalaLedger, DebitOrCredit, JournalId,
};

use crate::statement::*;

use error::*;

#[derive(Clone)]
pub struct TrialBalanceLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl TrialBalanceLedger {
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
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
        let op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(statement_id)
            .journal_id(self.journal_id)
            .external_id(reference)
            .name(reference)
            .description(reference)
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");
        self.cala.account_sets().create(new_account_set).await?;

        op.commit().await?;
        Ok(statement_id)
    }

    pub async fn find_by_name(
        &self,
        reference: String,
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
        Ok(self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id)
    }

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        statement_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), TrialBalanceLedgerError> {
        let statement_id = statement_id.into();

        let mut op = self.cala.ledger_operation_from_db_op(op);
        match self
            .cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, member)
            .await
        {
            Ok(_) | Err(cala_ledger::account_set::error::AccountSetError::MemberAlreadyAdded) => (),
            Err(e) => return Err(e.into()),
        }

        op.commit().await?;
        Ok(())
    }

    async fn get_statement_account_set(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &BalancesByAccount,
    ) -> Result<StatementAccountSet, TrialBalanceLedgerError> {
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
    ) -> Result<Vec<AccountSetId>, TrialBalanceLedgerError> {
        self.cala
            .account_sets()
            .list_members(account_set_id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(TrialBalanceLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, TrialBalanceLedgerError>>()
    }

    pub async fn get_trial_balance(
        &self,
        name: String,
    ) -> Result<StatementAccountSetWithAccounts, TrialBalanceLedgerError> {
        let statement_id = self.find_by_name(name).await?;
        let mut all_account_set_ids = vec![statement_id];

        let member_account_sets_ids = self.get_member_account_set_ids(statement_id).await?;
        all_account_set_ids.extend(&member_account_sets_ids);

        let mut balance_ids: Vec<BalanceId> = vec![];
        for account_set_id in all_account_set_ids {
            let account_set_balance_ids =
                BalanceIdsForAccountSet::from((self.journal_id, account_set_id)).balance_ids;
            balance_ids.extend(account_set_balance_ids);
        }

        let balances_by_id = self.cala.balances().find_all(&balance_ids).await?.into();

        let statement_account_set = self
            .get_statement_account_set(statement_id, &balances_by_id)
            .await?;

        let mut accounts = Vec::new();
        for account_set_id in member_account_sets_ids {
            accounts.push(
                self.get_statement_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        Ok(StatementAccountSetWithAccounts {
            id: statement_account_set.id,
            name: statement_account_set.name,
            description: statement_account_set.description,
            btc_balance: statement_account_set.btc_balance,
            usd_balance: statement_account_set.usd_balance,
            accounts,
        })
    }
}
