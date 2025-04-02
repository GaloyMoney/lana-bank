pub mod error;

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use cala_ledger::{
    account_set::{AccountSetMemberId, NewAccountSet},
    AccountSetId, CalaLedger, DebitOrCredit, JournalId, LedgerOperation,
};

use crate::statement::*;

use error::*;

use super::*;

#[derive(Clone)]
pub struct CashFlowStatementLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl CashFlowStatementLedger {
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
    ) -> Result<AccountSetId, CashFlowStatementLedgerError> {
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

    async fn create_account_set(
        &self,
        op: &mut LedgerOperation<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, CashFlowStatementLedgerError> {
        let id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(id)
            .journal_id(self.journal_id)
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

    async fn get_member_account_set_ids_and_names(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<HashMap<String, AccountSetId>, CashFlowStatementLedgerError> {
        let id = id.into();

        let member_ids = self
            .cala
            .account_sets()
            .list_members_by_created_at(id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(CashFlowStatementLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, CashFlowStatementLedgerError>>()?;

        let mut accounts: HashMap<String, AccountSetId> = HashMap::new();
        for id in member_ids {
            let account_set = self.cala.account_sets().find(id).await?.into_values();
            accounts.insert(account_set.name, id);
        }

        Ok(accounts)
    }

    async fn get_account_set(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &BalancesByAccount,
    ) -> Result<StatementAccountSet, CashFlowStatementLedgerError> {
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
    ) -> Result<Vec<AccountSetId>, CashFlowStatementLedgerError> {
        self.cala
            .account_sets()
            .list_members_by_created_at(account_set_id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(CashFlowStatementLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, CashFlowStatementLedgerError>>()
    }

    async fn get_balances_by_id(
        &self,
        all_account_set_ids: Vec<AccountSetId>,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<BalancesByAccount, CashFlowStatementLedgerError> {
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
    ) -> Result<(), CashFlowStatementLedgerError> {
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

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<CashFlowStatementIds, CashFlowStatementLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = self
            .create_unique_account_set(&mut op, reference, DebitOrCredit::Credit, vec![])
            .await?;

        let from_operations_id = self
            .create_account_set(
                &mut op,
                FROM_OPERATIONS_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;
        let from_investing_id = self
            .create_account_set(
                &mut op,
                FROM_INVESTING_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;
        let from_financing_id = self
            .create_account_set(
                &mut op,
                FROM_FINANCING_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;

        let net_income_id = self
            .create_account_set(
                &mut op,
                NET_INCOME_NAME,
                DebitOrCredit::Credit,
                vec![from_operations_id],
            )
            .await?;

        let revenue_id = self
            .create_account_set(
                &mut op,
                REVENUE_NAME,
                DebitOrCredit::Credit,
                vec![net_income_id],
            )
            .await?;
        let expenses_id = self
            .create_account_set(
                &mut op,
                EXPENSES_NAME,
                DebitOrCredit::Debit,
                vec![net_income_id],
            )
            .await?;

        let operations_non_cash_adustments_id = self
            .create_account_set(
                &mut op,
                OPERATIONS_NON_CASH_ADJUSTMENTS_NAME,
                DebitOrCredit::Debit,
                vec![from_operations_id],
            )
            .await?;
        let fee_income_non_cash_adj_id = self
            .create_account_set(
                &mut op,
                FEE_INCOME_ADJUSTMENTS_NAME,
                DebitOrCredit::Debit,
                vec![operations_non_cash_adustments_id],
            )
            .await?;

        let financing_non_cash_adustments_id = self
            .create_account_set(
                &mut op,
                FINANCING_NON_CASH_ADJUSTMENTS_NAME,
                DebitOrCredit::Debit,
                vec![from_financing_id],
            )
            .await?;
        let deposit_non_cash_adj_id = self
            .create_account_set(
                &mut op,
                DEPOSIT_ADJUSTMENTS_NAME,
                DebitOrCredit::Debit,
                vec![financing_non_cash_adustments_id],
            )
            .await?;

        op.commit().await?;

        Ok(CashFlowStatementIds {
            id: statement_id,
            from_operations: from_operations_id,
            from_investing: from_investing_id,
            from_financing: from_financing_id,
            revenue: revenue_id,
            expenses: expenses_id,
            fee_income_adjustments: fee_income_non_cash_adj_id,
            deposit_adjustments: deposit_non_cash_adj_id,
        })
    }

    pub async fn get_ids_from_reference(
        &self,
        reference: String,
    ) -> Result<CashFlowStatementIds, CashFlowStatementLedgerError> {
        let statement_id = self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id;

        let statement_members = self
            .get_member_account_set_ids_and_names(statement_id)
            .await?;
        let from_operations_id = statement_members.get(FROM_OPERATIONS_NAME).ok_or(
            CashFlowStatementLedgerError::NotFound(FROM_OPERATIONS_NAME.to_string()),
        )?;
        let from_investing_id = statement_members.get(FROM_INVESTING_NAME).ok_or(
            CashFlowStatementLedgerError::NotFound(FROM_INVESTING_NAME.to_string()),
        )?;
        let from_financing_id = statement_members.get(FROM_FINANCING_NAME).ok_or(
            CashFlowStatementLedgerError::NotFound(FROM_FINANCING_NAME.to_string()),
        )?;

        let from_operations_members = self
            .get_member_account_set_ids_and_names(*from_operations_id)
            .await?;
        let net_income_id = from_operations_members.get(NET_INCOME_NAME).ok_or(
            CashFlowStatementLedgerError::NotFound(NET_INCOME_NAME.to_string()),
        )?;
        let operations_non_cash_adjustments_id = from_operations_members
            .get(OPERATIONS_NON_CASH_ADJUSTMENTS_NAME)
            .ok_or(CashFlowStatementLedgerError::NotFound(
                OPERATIONS_NON_CASH_ADJUSTMENTS_NAME.to_string(),
            ))?;

        let net_income_members = self
            .get_member_account_set_ids_and_names(*net_income_id)
            .await?;
        let revenue_id =
            net_income_members
                .get(REVENUE_NAME)
                .ok_or(CashFlowStatementLedgerError::NotFound(
                    REVENUE_NAME.to_string(),
                ))?;
        let expenses_id =
            net_income_members
                .get(EXPENSES_NAME)
                .ok_or(CashFlowStatementLedgerError::NotFound(
                    EXPENSES_NAME.to_string(),
                ))?;

        let operations_non_cash_adjustments_members = self
            .get_member_account_set_ids_and_names(*operations_non_cash_adjustments_id)
            .await?;
        let fee_income_adj_id = operations_non_cash_adjustments_members
            .get(FEE_INCOME_ADJUSTMENTS_NAME)
            .ok_or(CashFlowStatementLedgerError::NotFound(
                FEE_INCOME_ADJUSTMENTS_NAME.to_string(),
            ))?;

        let from_financing_members = self
            .get_member_account_set_ids_and_names(*from_financing_id)
            .await?;
        let financing_non_cash_adjustments_id = from_financing_members
            .get(FINANCING_NON_CASH_ADJUSTMENTS_NAME)
            .ok_or(CashFlowStatementLedgerError::NotFound(
                FINANCING_NON_CASH_ADJUSTMENTS_NAME.to_string(),
            ))?;
        let financing_non_cash_adjustments_members = self
            .get_member_account_set_ids_and_names(*financing_non_cash_adjustments_id)
            .await?;
        let deposit_adj_id = financing_non_cash_adjustments_members
            .get(DEPOSIT_ADJUSTMENTS_NAME)
            .ok_or(CashFlowStatementLedgerError::NotFound(
                DEPOSIT_ADJUSTMENTS_NAME.to_string(),
            ))?;

        Ok(CashFlowStatementIds {
            id: statement_id,
            from_operations: *from_operations_id,
            from_investing: *from_investing_id,
            from_financing: *from_financing_id,
            revenue: *revenue_id,
            expenses: *expenses_id,
            fee_income_adjustments: *fee_income_adj_id,
            deposit_adjustments: *deposit_adj_id,
        })
    }

    pub async fn get_cash_flow_statement(
        &self,
        reference: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<CashFlowStatement, CashFlowStatementLedgerError> {
        let ids = self.get_ids_from_reference(reference).await?;
        let mut all_account_set_ids = vec![
            ids.id,
            ids.from_operations,
            ids.from_investing,
            ids.from_financing,
        ];

        let from_operations_member_account_sets_ids =
            self.get_member_account_set_ids(ids.from_operations).await?;
        all_account_set_ids.extend(&from_operations_member_account_sets_ids);

        let from_investing_member_account_sets_ids =
            self.get_member_account_set_ids(ids.from_investing).await?;
        all_account_set_ids.extend(&from_investing_member_account_sets_ids);

        let from_financing_member_account_sets_ids =
            self.get_member_account_set_ids(ids.from_financing).await?;
        all_account_set_ids.extend(&from_financing_member_account_sets_ids);

        let balances_by_id = self
            .get_balances_by_id(all_account_set_ids, from, until)
            .await?;

        let statement_account_set = self.get_account_set(ids.id, &balances_by_id).await?;
        let from_operations_account_set = self
            .get_account_set(ids.from_operations, &balances_by_id)
            .await?;
        let from_investing_account_set = self
            .get_account_set(ids.from_investing, &balances_by_id)
            .await?;
        let from_financing_account_set = self
            .get_account_set(ids.from_financing, &balances_by_id)
            .await?;

        let mut from_operations_accounts = Vec::new();
        for account_set_id in from_operations_member_account_sets_ids {
            from_operations_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        let mut from_investing_accounts = Vec::new();
        for account_set_id in from_investing_member_account_sets_ids {
            from_investing_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        let mut from_financing_accounts = Vec::new();
        for account_set_id in from_financing_member_account_sets_ids {
            from_financing_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }
        Ok(CashFlowStatement {
            id: statement_account_set.id,
            name: statement_account_set.name,
            description: statement_account_set.description,
            btc_balance: statement_account_set.btc_balance,
            usd_balance: statement_account_set.usd_balance,
            categories: vec![
                StatementAccountSetWithAccounts {
                    id: from_operations_account_set.id,
                    name: from_operations_account_set.name,
                    description: from_operations_account_set.description,
                    btc_balance: from_operations_account_set.btc_balance,
                    usd_balance: from_operations_account_set.usd_balance,
                    accounts: from_operations_accounts,
                },
                StatementAccountSetWithAccounts {
                    id: from_investing_account_set.id,
                    name: from_investing_account_set.name,
                    description: from_investing_account_set.description,
                    btc_balance: from_investing_account_set.btc_balance,
                    usd_balance: from_investing_account_set.usd_balance,
                    accounts: from_investing_accounts,
                },
                StatementAccountSetWithAccounts {
                    id: from_financing_account_set.id,
                    name: from_financing_account_set.name,
                    description: from_financing_account_set.description,
                    btc_balance: from_financing_account_set.btc_balance,
                    usd_balance: from_financing_account_set.usd_balance,
                    accounts: from_financing_accounts,
                },
            ],
        })
    }
}
