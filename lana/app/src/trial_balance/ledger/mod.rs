pub mod error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use cala_ledger::{
    account_set::{AccountSetMember, AccountSetMemberId, AccountSetMembersCursor, NewAccountSet},
    AccountSetId, CalaLedger, DebitOrCredit, JournalId, LedgerOperation,
};

use chart_of_accounts::AccountCode;

use crate::statement::*;

use error::*;

#[derive(Clone)]
pub struct TrialBalanceAccountSet {
    pub id: AccountSetId,
    pub name: String,
    pub code: AccountCode,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub member_created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TrialBalanceRoot {
    pub id: AccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub from: DateTime<Utc>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceAccountSetsCursor {
    pub created_at: DateTime<Utc>,
}

impl From<TrialBalanceAccountSetsCursor> for AccountSetMembersCursor {
    fn from(cursor: TrialBalanceAccountSetsCursor) -> Self {
        Self {
            member_created_at: cursor.created_at,
        }
    }
}

impl From<AccountSetMembersCursor> for TrialBalanceAccountSetsCursor {
    fn from(cursor: AccountSetMembersCursor) -> Self {
        Self {
            created_at: cursor.member_created_at,
        }
    }
}

impl es_entity::graphql::async_graphql::connection::CursorType for TrialBalanceAccountSetsCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

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

    async fn trial_balance_root(
        &self,
        account_set_id: AccountSetId,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
        balances_by_id: &BalancesByAccount,
    ) -> Result<TrialBalanceRoot, TrialBalanceLedgerError> {
        let values = self
            .cala
            .account_sets()
            .find(account_set_id)
            .await?
            .into_values();

        Ok(TrialBalanceRoot {
            id: account_set_id,
            name: values.name,
            description: values.description,
            btc_balance: balances_by_id.btc_for_account(account_set_id)?,
            usd_balance: balances_by_id.usd_for_account(account_set_id)?,
            from,
            until,
        })
    }

    async fn get_member_account_set(
        &self,
        account_set_id: AccountSetId,
        member_created_at: DateTime<Utc>,
        balances_by_id: &BalancesByAccount,
    ) -> Result<TrialBalanceAccountSet, TrialBalanceLedgerError> {
        let values = self
            .cala
            .account_sets()
            .find(account_set_id)
            .await?
            .into_values();

        let code = values
            .external_id
            .expect("external_id should exist")
            .parse()?;

        Ok(TrialBalanceAccountSet {
            id: account_set_id,
            name: values.name,
            description: values.description,
            btc_balance: balances_by_id.btc_for_account(account_set_id)?,
            usd_balance: balances_by_id.usd_for_account(account_set_id)?,
            code,
            member_created_at,
        })
    }

    async fn get_member_account_sets<U>(
        &self,
        account_set_id: AccountSetId,
        cursor: es_entity::PaginatedQueryArgs<U>,
    ) -> Result<es_entity::PaginatedQueryRet<AccountSetMember, U>, TrialBalanceLedgerError>
    where
        U: std::fmt::Debug + From<AccountSetMembersCursor> + Into<AccountSetMembersCursor>,
    {
        let cala_cursor = es_entity::PaginatedQueryArgs {
            after: cursor.after.map(|u| u.into()),
            first: cursor.first,
        };

        let ret = self
            .cala
            .account_sets()
            .list_members(account_set_id, cala_cursor)
            .await?;

        Ok(es_entity::PaginatedQueryRet {
            entities: ret.entities,
            has_next_page: ret.has_next_page,
            end_cursor: ret.end_cursor.map(|c| c.into()),
        })
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
    ) -> Result<TrialBalanceRoot, TrialBalanceLedgerError> {
        let statement_id = self.get_id_from_reference(name).await?;

        let balances_by_id = self
            .get_balances_by_id(vec![statement_id], from, until)
            .await?;

        let statement_account_set = self
            .trial_balance_root(statement_id, from, until, &balances_by_id)
            .await?;

        Ok(TrialBalanceRoot {
            id: statement_account_set.id,
            name: statement_account_set.name,
            description: statement_account_set.description,
            btc_balance: statement_account_set.btc_balance,
            usd_balance: statement_account_set.usd_balance,
            from,
            until,
        })
    }

    pub async fn accounts(
        &self,
        name: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
        query: es_entity::PaginatedQueryArgs<TrialBalanceAccountSetsCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<TrialBalanceAccountSet, TrialBalanceAccountSetsCursor>,
        TrialBalanceLedgerError,
    > {
        let statement_id = self.get_id_from_reference(name).await?;

        let member_account_sets = self
            .get_member_account_sets::<TrialBalanceAccountSetsCursor>(statement_id, query)
            .await?;
        let member_account_sets_tuples = member_account_sets
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok((id, m.created_at)),
                _ => Err(TrialBalanceLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<(AccountSetId, DateTime<Utc>)>, TrialBalanceLedgerError>>()?;

        let member_account_sets_ids = member_account_sets_tuples
            .iter()
            .map(|&(id, _)| id)
            .collect();
        let balances_by_id = self
            .get_balances_by_id(member_account_sets_ids, from, until)
            .await?;

        let mut accounts = Vec::new();
        for (account_set_id, created_at) in member_account_sets_tuples {
            accounts.push(
                self.get_member_account_set(account_set_id, created_at, &balances_by_id)
                    .await?,
            );
        }

        Ok(es_entity::PaginatedQueryRet {
            entities: accounts,
            has_next_page: member_account_sets.has_next_page,
            end_cursor: member_account_sets.end_cursor,
        })
    }
}
