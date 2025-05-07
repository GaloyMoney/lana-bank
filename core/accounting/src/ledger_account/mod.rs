mod cursor;
pub mod error;
mod ledger;
mod value;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;

use crate::journal::{JournalEntry, JournalEntryCursor};
use crate::{
    chart_of_accounts::Chart,
    primitives::{
        AccountCode, CalaJournalId, CoreAccountingAction, CoreAccountingObject, LedgerAccountId,
    },
};

pub use cursor::LedgerAccountChildrenCursor;
use error::*;
use ledger::*;
pub use value::*;

#[derive(Clone)]
pub struct LedgerAccounts<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    ledger: LedgerAccountLedger,
}

impl<Perms> LedgerAccounts<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(authz: &Perms, cala: &CalaLedger, journal_id: CalaJournalId) -> Self {
        Self {
            authz: authz.clone(),
            ledger: LedgerAccountLedger::new(cala, journal_id),
        }
    }

    pub async fn history(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<LedgerAccountId>,
        args: es_entity::PaginatedQueryArgs<JournalEntryCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<JournalEntry, JournalEntryCursor>, LedgerAccountError>
    {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::ledger_account(id),
                CoreAccountingAction::LEDGER_ACCOUNT_READ_HISTORY,
            )
            .await?;

        Ok(self.ledger.ledger_account_history(id, args).await?)
    }

    pub(crate) async fn complete_history(
        &self,
        id: impl Into<LedgerAccountId> + Copy,
    ) -> Result<Vec<JournalEntry>, LedgerAccountError> {
        let id = id.into();

        let mut all_entries = Vec::new();
        let mut cursor: Option<JournalEntryCursor> = None;
        let page_size = 100;

        loop {
            let query_args = es_entity::PaginatedQueryArgs {
                first: page_size,
                after: cursor,
            };

            let result = self.ledger.ledger_account_history(id, query_args).await?;
            all_entries.extend(result.entities);

            if !result.has_next_page {
                break;
            }

            cursor = result.end_cursor;
        }

        Ok(all_entries)
    }

    #[instrument(name = "accounting.ledger_account.find_by_id", skip(self, chart), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Option<LedgerAccount>, LedgerAccountError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::ledger_account(id),
                CoreAccountingAction::LEDGER_ACCOUNT_READ,
            )
            .await?;
        let mut accounts = self.find_all(chart, &[id]).await?;
        Ok(accounts.remove(&id))
    }

    #[instrument(name = "accounting.ledger_account.find_by_id", skip(self, chart), err)]
    pub async fn find_by_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        code: AccountCode,
    ) -> Result<Option<LedgerAccount>, LedgerAccountError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_ledger_accounts(),
                CoreAccountingAction::LEDGER_ACCOUNT_LIST,
            )
            .await?;
        if let Some(mut account) = self
            .ledger
            .load_ledger_account_by_external_id(code.account_set_external_id(chart.id))
            .await?
        {
            self.populate_ancestors(chart, &mut account).await?;
            self.populate_children(chart, &mut account).await?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    pub async fn find_all<T: From<LedgerAccount>>(
        &self,
        chart: &Chart,
        ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, T>, LedgerAccountError> {
        let accounts = self.ledger.load_ledger_accounts(ids).await?;
        let mut res = HashMap::new();
        for (k, mut v) in accounts.into_iter() {
            self.populate_ancestors(chart, &mut v).await?;
            self.populate_children(chart, &mut v).await?;
            res.insert(k, v.into());
        }
        Ok(res)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_account_children(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        id: cala_ledger::AccountSetId,
        mut args: es_entity::PaginatedQueryArgs<LedgerAccountChildrenCursor>,
        from: chrono::NaiveDate,
        until: Option<chrono::NaiveDate>,
        filter_non_zero: bool,
    ) -> Result<
        es_entity::PaginatedQueryRet<LedgerAccount, LedgerAccountChildrenCursor>,
        LedgerAccountError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_ledger_accounts(),
                CoreAccountingAction::LEDGER_ACCOUNT_LIST,
            )
            .await?;
        let mut entities = Vec::with_capacity(args.first);
        loop {
            let res = self
                .ledger
                .list_children(
                    id,
                    es_entity::PaginatedQueryArgs {
                        first: args.first,
                        after: args.after.take(),
                    },
                    from,
                    until,
                )
                .await?;

            for mut account in res.entities {
                if filter_non_zero && !account.has_non_zero_balance() {
                    continue;
                }
                self.populate_ancestors(chart, &mut account).await?;
                self.populate_children(chart, &mut account).await?;
                entities.push(account);
                if entities.len() >= args.first {
                    break;
                }
            }

            if !res.has_next_page || entities.len() >= args.first {
                return Ok(es_entity::PaginatedQueryRet {
                    entities,
                    has_next_page: res.has_next_page,
                    end_cursor: res.end_cursor,
                });
            }

            args.after = res.end_cursor;
        }
    }

    /// Pushes into `account`'s `ancestor_ids` ancestors from the chart of account. The ancestors
    /// are pushed in ascending order, the root of the chart of accounts is pushed last. `account`
    /// itself is not pushed.
    async fn populate_ancestors(
        &self,
        chart: &Chart,
        account: &mut LedgerAccount,
    ) -> Result<(), LedgerAccountError> {
        if let Some(code) = account.code.as_ref() {
            account.ancestor_ids = chart.ancestors(code);
        } else if let Some((id, code)) = self
            .ledger
            .find_parent_with_account_code(account.account_set_member_id(), 1)
            .await?
        {
            let mut ancestors = chart.ancestors(&code);
            ancestors.insert(0, id.into());
            account.ancestor_ids = ancestors;
        }
        Ok(())
    }

    async fn populate_children(
        &self,
        chart: &Chart,
        account: &mut LedgerAccount,
    ) -> Result<(), LedgerAccountError> {
        let children = if let Some(code) = &account.code {
            chart.children::<LedgerAccountId>(code)
        } else {
            Vec::new()
        };
        account.children_ids = if children.is_empty() {
            self.ledger.find_leaf_children(account.id, 1).await?
        } else {
            children
        };
        Ok(())
    }
}
