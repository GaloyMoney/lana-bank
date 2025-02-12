pub mod error;

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use cala_ledger::{
    account_set::{AccountSetMemberId, NewAccountSet},
    AccountSetId, CalaLedger, DebitOrCredit, JournalId, LedgerOperation,
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
    async fn create_unique_account_set(
        &self,
        op: &mut LedgerOperation<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, BalanceSheetLedgerError> {
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
    ) -> Result<AccountSetId, BalanceSheetLedgerError> {
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

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = self
            .create_unique_account_set(&mut op, reference, DebitOrCredit::Debit, vec![])
            .await?;

        let assets_id = self
            .create_account_set(
                &mut op,
                ASSETS_NAME,
                DebitOrCredit::Debit,
                vec![statement_id],
            )
            .await?;
        let liabilities_id = self
            .create_account_set(
                &mut op,
                LIABILITIES_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;
        let equity_id = self
            .create_account_set(
                &mut op,
                EQUITY_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;

        let net_income_id = self
            .create_account_set(
                &mut op,
                NET_INCOME_NAME,
                DebitOrCredit::Credit,
                vec![equity_id],
            )
            .await?;

        let revenue_id = self
            .create_account_set(
                &mut op,
                NI_REVENUE_NAME,
                DebitOrCredit::Credit,
                vec![net_income_id],
            )
            .await?;
        let expenses_id = self
            .create_account_set(
                &mut op,
                NI_EXPENSES_NAME,
                DebitOrCredit::Debit,
                vec![net_income_id],
            )
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

    async fn get_member_account_set_ids_and_names(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<HashMap<String, AccountSetId>, BalanceSheetLedgerError> {
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
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let statement_id = self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id;

        let statement_members = self
            .get_member_account_set_ids_and_names(statement_id)
            .await?;
        let assets_id = statement_members
            .get(ASSETS_NAME)
            .ok_or(BalanceSheetLedgerError::NotFound(ASSETS_NAME.to_string()))?;
        let liabilities_id =
            statement_members
                .get(LIABILITIES_NAME)
                .ok_or(BalanceSheetLedgerError::NotFound(
                    LIABILITIES_NAME.to_string(),
                ))?;
        let equity_id = statement_members
            .get(EQUITY_NAME)
            .ok_or(BalanceSheetLedgerError::NotFound(EQUITY_NAME.to_string()))?;

        let equity_members = self
            .get_member_account_set_ids_and_names(*equity_id)
            .await?;
        let net_income_id =
            equity_members
                .get(NET_INCOME_NAME)
                .ok_or(BalanceSheetLedgerError::NotFound(
                    NET_INCOME_NAME.to_string(),
                ))?;

        let net_income_members = self
            .get_member_account_set_ids_and_names(*net_income_id)
            .await?;
        let revenue_id =
            net_income_members
                .get(NI_REVENUE_NAME)
                .ok_or(BalanceSheetLedgerError::NotFound(
                    NI_REVENUE_NAME.to_string(),
                ))?;
        let expenses_id =
            net_income_members
                .get(NI_EXPENSES_NAME)
                .ok_or(BalanceSheetLedgerError::NotFound(
                    NI_EXPENSES_NAME.to_string(),
                ))?;

        Ok(BalanceSheetIds {
            id: statement_id,
            assets: *assets_id,
            liabilities: *liabilities_id,
            equity: *equity_id,
            revenue: *revenue_id,
            expenses: *expenses_id,
        })
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

    async fn get_account_set(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &BalancesByAccount,
    ) -> Result<StatementAccountSet, BalanceSheetLedgerError> {
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
    ) -> Result<Vec<AccountSetId>, BalanceSheetLedgerError> {
        self.cala
            .account_sets()
            .list_members(account_set_id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(BalanceSheetLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, BalanceSheetLedgerError>>()
    }

    pub async fn get_balance_sheet(
        &self,
        reference: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<BalanceSheet, BalanceSheetLedgerError> {
        let ids = self.get_ids_from_reference(reference).await?;
        let mut all_account_set_ids = vec![ids.id, ids.assets, ids.liabilities, ids.equity];

        let assets_member_account_sets_ids = self.get_member_account_set_ids(ids.assets).await?;
        all_account_set_ids.extend(&assets_member_account_sets_ids);

        let liabilities_member_account_sets_ids =
            self.get_member_account_set_ids(ids.liabilities).await?;
        all_account_set_ids.extend(&liabilities_member_account_sets_ids);

        let equity_member_account_sets_ids = self.get_member_account_set_ids(ids.equity).await?;
        all_account_set_ids.extend(&equity_member_account_sets_ids);

        let balance_ids =
            BalanceIdsForAccountSets::from((self.journal_id, all_account_set_ids)).balance_ids;
        let balances_by_id = self
            .cala
            .balances()
            .find_all_in_range(&balance_ids, from, until)
            .await?
            .into();

        let statement_account_set = self.get_account_set(ids.id, &balances_by_id).await?;
        let assets_account_set = self.get_account_set(ids.assets, &balances_by_id).await?;
        let liabilities_account_set = self
            .get_account_set(ids.liabilities, &balances_by_id)
            .await?;
        let equity_account_set = self.get_account_set(ids.equity, &balances_by_id).await?;

        let mut assets_accounts = Vec::new();
        for account_set_id in assets_member_account_sets_ids {
            assets_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        let mut liabilities_accounts = Vec::new();
        for account_set_id in liabilities_member_account_sets_ids {
            liabilities_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }

        let mut equity_accounts = Vec::new();
        for account_set_id in equity_member_account_sets_ids {
            equity_accounts.push(
                self.get_account_set(account_set_id, &balances_by_id)
                    .await?,
            );
        }
        Ok(BalanceSheet {
            id: statement_account_set.id,
            name: statement_account_set.name,
            description: statement_account_set.description,
            btc_balance: statement_account_set.btc_balance,
            usd_balance: statement_account_set.usd_balance,
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
