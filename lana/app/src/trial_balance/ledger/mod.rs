pub mod error;

use chrono::{DateTime, Utc};

use cala_ledger::{
    account_set::{AccountSetMemberId, NewAccountSet},
    AccountSetId, CalaLedger, DebitOrCredit, JournalId, LedgerOperation,
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

    async fn create_unique_account_set(
        &self,
        op: &mut LedgerOperation<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
        let id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(id)
            .journal_id(self.journal_id)
            .external_id(reference)
            .name(reference)
            .description(reference)
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(op, new_account_set)
            .await?;

        for parent_id in parents {
            self.cala
                .account_sets()
                .add_member_in_op(op, parent_id, id)
                .await?;
        }

        Ok(id)
    }

    async fn get_account_set(
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

    async fn get_balances_by_id(
        &self,
        all_account_set_ids: Vec<AccountSetId>,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<BalancesByAccount, TrialBalanceLedgerError> {
        let balance_ids =
            BalanceIdsForAccountSets::from((self.journal_id, all_account_set_ids)).balance_ids;
        Ok(self
            .cala
            .balances()
            .find_all_in_range(&balance_ids, from, until)
            .await?
            .into())
    }

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        node_account_set_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), TrialBalanceLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.add_member_in_op(&mut op, node_account_set_id, member)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn add_members(
        &self,
        op: es_entity::DbOp<'_>,
        node_account_set_id: impl Into<AccountSetId> + Copy,
        members: impl Iterator<Item = AccountSetId>,
    ) -> Result<(), TrialBalanceLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        for member in members {
            self.add_member_in_op(&mut op, node_account_set_id, member)
                .await?;
        }

        op.commit().await?;
        Ok(())
    }

    async fn add_member_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        node_account_set_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), TrialBalanceLedgerError> {
        let node_account_set_id = node_account_set_id.into();

        match self
            .cala
            .account_sets()
            .add_member_in_op(op, node_account_set_id, member)
            .await
        {
            Ok(_) | Err(cala_ledger::account_set::error::AccountSetError::MemberAlreadyAdded) => {}
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = self
            .create_unique_account_set(&mut op, reference, DebitOrCredit::Debit, vec![])
            .await?;

        op.commit().await?;
        Ok(statement_id)
    }

    pub async fn get_id_from_reference(
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

    pub async fn get_trial_balance(
        &self,
        name: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<StatementAccountSetWithAccounts, TrialBalanceLedgerError> {
        let statement_id = self.get_id_from_reference(name).await?;
        let mut all_account_set_ids = vec![statement_id];

        let member_account_sets_ids = self.get_member_account_set_ids(statement_id).await?;
        all_account_set_ids.extend(&member_account_sets_ids);

        let balances_by_id = self
            .get_balances_by_id(all_account_set_ids, from, until)
            .await?;

        let statement_account_set = self.get_account_set(statement_id, &balances_by_id).await?;

        let mut accounts = Vec::new();
        for account_set_id in member_account_sets_ids {
            accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
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
