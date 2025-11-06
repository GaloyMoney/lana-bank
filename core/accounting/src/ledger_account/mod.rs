mod cursor;
pub mod error;
mod ledger;
mod value;

use std::collections::{BTreeMap, HashMap};
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

    #[instrument(name = "core_accounting.ledger_account.history", skip(self), err)]
    pub async fn history(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<LedgerAccountId> + std::fmt::Debug,
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

    #[instrument(
        name = "core_accounting.ledger_account.complete_history",
        skip(self),
        err
    )]
    pub(crate) async fn complete_history(
        &self,
        id: impl Into<LedgerAccountId> + Copy + std::fmt::Debug,
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

    #[instrument(
        name = "core_accounting.ledger_account.find_by_id",
        skip(self, chart),
        err
    )]
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

    #[instrument(
        name = "core_accounting.ledger_account.find_by_id",
        skip(self, chart),
        err
    )]
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

    #[instrument(
        name = "core_accounting.ledger_account.find_all",
        skip(self, chart),
        err
    )]
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

    #[instrument(
        name = "core_accounting.ledger_account.list_all_account_flattened",
        skip(self, chart),
        err
    )]
    pub async fn list_all_account_flattened(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        from: chrono::NaiveDate,
        until: Option<chrono::NaiveDate>,
        filter_non_zero: bool,
    ) -> Result<Vec<LedgerAccount>, LedgerAccountError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_ledger_accounts(),
                CoreAccountingAction::LEDGER_ACCOUNT_LIST,
            )
            .await?;

        let chart_tree = chart.chart();
        let mut ordered_ids = Vec::new();
        for node in &chart_tree.children {
            ordered_ids.push(LedgerAccountId::from(node.id));
            ordered_ids.extend(node.descendants().into_iter().map(LedgerAccountId::from));
        }

        let mut entries = self
            .ledger
            .load_account_sets_in_range(&ordered_ids, from, until, filter_non_zero)
            .await?;

        for entry in &mut entries {
            self.populate_ancestors(chart, entry).await?;
            self.populate_children(chart, entry).await?;
        }

        Ok(entries)
    }

    /// Pushes into `account`'s `ancestor_ids` ancestors from the chart of account. The ancestors
    /// are pushed in ascending order, the root of the chart of accounts is pushed last. `account`
    /// itself is not pushed.
    #[instrument(
        name = "core_accounting.ledger_account.populate_ancestors",
        skip(self, chart, account),
        err
    )]
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

    #[instrument(
        name = "core_accounting.ledger_account.populate_children",
        skip(self, chart, account),
        err
    )]
    async fn populate_children(
        &self,
        chart: &Chart,
        account: &mut LedgerAccount,
    ) -> Result<(), LedgerAccountError> {
        let children: BTreeMap<_, _> = account
            .code
            .as_ref()
            .map(|code| chart.children(code).collect())
            .unwrap_or_default();

        account.children_ids = if children.is_empty() {
            self.ledger.find_leaf_children(account.id, 1).await?
        } else {
            children.into_values().map(Into::into).collect()
        };

        Ok(())
    }
}
