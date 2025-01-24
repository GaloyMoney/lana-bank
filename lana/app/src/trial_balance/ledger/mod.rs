pub mod error;

pub(super) use cala_ledger::balance::AccountBalance;
use cala_ledger::{
    account_set::{
        AccountSet, AccountSetMemberId, AccountSetValues, AccountSetsByCreatedAtCursor,
        NewAccountSet,
    },
    AccountSetId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
};

use error::*;

#[derive(Clone)]
pub struct TrialBalanceLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

pub struct LedgerAccountSetDetails {
    pub values: AccountSetValues,
    pub btc_balance: AccountBalance,
    pub usd_balance: AccountBalance,
}

pub struct LedgerAccountSetDetailsWithAccounts {
    pub values: AccountSetValues,
    pub btc_balance: AccountBalance,
    pub usd_balance: AccountBalance,
    pub accounts: Vec<LedgerAccountSetDetails>,
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
        statement_id: impl Into<AccountSetId>,
        name: &str,
    ) -> Result<(), TrialBalanceLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

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

        op.commit().await?;
        Ok(())
    }

    pub async fn list_for_name(
        &self,
        name: String,
        args: es_entity::PaginatedQueryArgs<AccountSetsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<AccountSet, AccountSetsByCreatedAtCursor>,
        TrialBalanceLedgerError,
    > {
        Ok(self.cala.account_sets().list_for_name(name, args).await?)
    }

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        statement_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), TrialBalanceLedgerError> {
        let statement_id = statement_id.into();

        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, member)
            .await?;

        op.commit().await?;
        Ok(())
    }

    async fn get_account_set(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<LedgerAccountSetDetails, TrialBalanceLedgerError> {
        let id = id.into();

        let values = self.cala.account_sets().find(id).await?.into_values();

        let btc_currency =
            Currency::try_from("BTC".to_string()).expect("Cannot deserialize 'BTC' as Currency");
        let btc_balance = self
            .cala
            .balances()
            .find_in_op(op, self.journal_id, id, btc_currency)
            .await?;

        let usd_currency =
            Currency::try_from("BTC".to_string()).expect("Cannot deserialize 'BTC' as Currency");
        let usd_balance = self
            .cala
            .balances()
            .find_in_op(op, self.journal_id, id, usd_currency)
            .await?;

        Ok(LedgerAccountSetDetails {
            values,
            btc_balance,
            usd_balance,
        })
    }

    async fn get_member_account_sets(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<Vec<LedgerAccountSetDetails>, TrialBalanceLedgerError> {
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
                _ => Err(TrialBalanceLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, TrialBalanceLedgerError>>()?;

        let mut accounts: Vec<LedgerAccountSetDetails> = vec![];
        for id in member_ids {
            accounts.push(self.get_account_set(op, id).await?);
        }

        Ok(accounts)
    }

    pub async fn get_trial_balance(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<LedgerAccountSetDetailsWithAccounts, TrialBalanceLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let trial_balance_set = self.get_account_set(&mut op, id).await?;

        let accounts = self.get_member_account_sets(&mut op, id).await?;

        op.commit().await?;

        Ok(LedgerAccountSetDetailsWithAccounts {
            values: trial_balance_set.values,
            btc_balance: trial_balance_set.btc_balance,
            usd_balance: trial_balance_set.usd_balance,
            accounts,
        })
    }
}
