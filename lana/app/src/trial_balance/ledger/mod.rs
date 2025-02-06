pub mod error;

use std::collections::HashMap;

use cala_ledger::{
    account_set::{AccountSetMemberId, AccountSetValues, NewAccountSet},
    balance::AccountBalance,
    AccountId, AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId,
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
        balances_by_id: &HashMap<AccountId, HashMap<Currency, AccountBalance>>,
    ) -> Result<StatementAccountSet, TrialBalanceLedgerError> {
        let values = self
            .cala
            .account_sets()
            .find(account_set_id)
            .await?
            .into_values();

        let mut btc_balance = BtcStatementAccountSetBalance::ZERO;
        let mut usd_balance = UsdStatementAccountSetBalance::ZERO;
        if let Some(balances) = balances_by_id.get(&account_set_id.into()) {
            if let Some(bal) = balances.get(&("BTC".parse()?)) {
                btc_balance = bal.clone().try_into()?;
            };
            if let Some(bal) = balances.get(&("USD".parse()?)) {
                usd_balance = bal.clone().try_into()?;
            };
        };

        Ok(StatementAccountSet {
            id: account_set_id,
            name: values.name,
            description: values.description,
            btc_balance,
            usd_balance,
        })
    }

    pub async fn get_trial_balance(
        &self,
        name: String,
    ) -> Result<StatementAccountSetWithAccounts, TrialBalanceLedgerError> {
        let statement_id = self.find_by_name(name).await?;
        let mut all_account_set_ids = vec![statement_id];

        let member_account_sets_ids = self
            .cala
            .account_sets()
            .list_members(statement_id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(TrialBalanceLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, TrialBalanceLedgerError>>()?;
        all_account_set_ids.extend(&member_account_sets_ids);

        let mut balance_ids: Vec<BalanceId> = vec![];
        for account_id in all_account_set_ids {
            balance_ids.extend([
                (self.journal_id, account_id.into(), "BTC".parse()?),
                (self.journal_id, account_id.into(), "USD".parse()?),
            ]);
        }

        let all_balances = self.cala.balances().find_all(&balance_ids).await?;
        let mut balances_by_id: HashMap<AccountId, HashMap<Currency, AccountBalance>> =
            HashMap::new();
        for ((_, account_id, currency), balance) in all_balances {
            balances_by_id
                .entry(account_id)
                .or_default()
                .insert(currency, balance);
        }

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
