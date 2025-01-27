pub mod error;

use cala_ledger::{
    account_set::{AccountSet, AccountSetMemberId, AccountSetsByCreatedAtCursor, NewAccountSet},
    balance::error::BalanceError,
    AccountSetId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
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
        match self
            .cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, member)
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
    ) -> Result<StatementAccountSet, TrialBalanceLedgerError> {
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

    async fn get_member_account_sets_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<Vec<StatementAccountSet>, TrialBalanceLedgerError> {
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

        let mut accounts: Vec<StatementAccountSet> = vec![];
        for id in member_ids {
            accounts.push(self.get_account_set_in_op(op, id).await?);
        }

        Ok(accounts)
    }

    pub async fn get_trial_balance(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<StatementAccountSetWithAccounts, TrialBalanceLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let trial_balance_set = self.get_account_set_in_op(&mut op, id).await?;

        let accounts = self.get_member_account_sets_in_op(&mut op, id).await?;

        op.commit().await?;

        Ok(StatementAccountSetWithAccounts {
            id: trial_balance_set.id,
            name: trial_balance_set.name,
            description: trial_balance_set.description,
            btc_balance: trial_balance_set.btc_balance,
            usd_balance: trial_balance_set.usd_balance,
            accounts,
        })
    }
}
