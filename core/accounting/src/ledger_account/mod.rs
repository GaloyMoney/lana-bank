pub mod error;
mod ledger;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{AccountId, AccountSetId, CalaLedger};

use crate::journal::{JournalEntry, JournalEntryCursor};
use crate::{
    chart_of_accounts::Chart,
    primitives::{
        AccountCode, CalaAccountBalance, CalaJournalId, CoreAccountingAction, CoreAccountingObject,
        LedgerAccountId,
    },
};

use error::*;
use ledger::*;

pub struct LedgerAccount {
    pub id: LedgerAccountId,
    pub name: String,
    pub code: Option<AccountCode>,
    pub usd_balance: Option<CalaAccountBalance>,
    pub btc_balance: Option<CalaAccountBalance>,

    pub ancestor_ids: Vec<LedgerAccountId>,

    is_leaf: bool,
}

impl LedgerAccount {
    pub(crate) const fn is_leaf_account(&self) -> bool {
        self.is_leaf
    }

    pub(crate) const fn is_module_account_set(&self) -> bool {
        self.code.is_none() && !self.is_leaf
    }
}

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

        let res = self.ledger.account_set_history(id.into(), args).await?;

        // TODO if empty check account history
        Ok(res)
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
            res.insert(k, v.into());
        }
        Ok(res)
    }

    /// Pushes into `account`'s `ancestor_ids` parents of itself. Inner parents are pushed
    /// first in ascending order, the root of the chart of accounts is pushed last. `account`
    /// itself is not pushed.
    ///
    /// If `account` is a leaf account, the function assumes that there is at most one
    /// intermediate level (account set) between the leaf account and the chart of accounts.
    /// If `acccount` is account set, itself or its parent must be in the chart of accounts.
    async fn populate_ancestors(
        &self,
        chart: &Chart,
        account: &mut LedgerAccount,
    ) -> Result<(), LedgerAccountError> {
        if let Some(code) = account.code.as_ref() {
            // `account` is already in the chart
            account.ancestor_ids = chart.ancestors(code);
        } else if account.is_leaf_account() {
            // account is a leaf account whose parent may or may not be in CoA
            match self
                .ledger
                .find_parent_of_account(AccountId::from(account.id))
                .await?
            {
                // parent is already in CoA
                Some((parent, Some(code))) => {
                    account.ancestor_ids.push(parent); // because chart.ancestors excludes itself
                    account
                        .ancestor_ids
                        .extend(chart.ancestors::<LedgerAccountId>(&code));
                }
                // parent is not in CoA but its parents should be
                Some((parent, None)) => {
                    account.ancestor_ids.push(parent);
                    self.populate_parent_coa(chart, parent, &mut account.ancestor_ids)
                        .await?;
                }
                _ => {}
            }
        } else if account.is_module_account_set() {
            // account is an internal account set whose parent is expected to be in CoA
            self.populate_parent_coa(chart, account.id, &mut account.ancestor_ids)
                .await?;
        }
        Ok(())
    }

    /// Pushes into `ancestor_ids` all parents of `id` if they are in the chart of accounts, otherwise
    /// does nothing.
    async fn populate_parent_coa(
        &self,
        chart: &Chart,
        id: LedgerAccountId,
        ancestor_ids: &mut Vec<LedgerAccountId>,
    ) -> Result<(), LedgerAccountError> {
        if let Some((coa, Some(code))) = self
            .ledger
            .find_parent_of_account(AccountSetId::from(id))
            .await?
        {
            ancestor_ids.push(coa); // because chart.ancestors excludes itself
            ancestor_ids.extend(chart.ancestors::<LedgerAccountId>(&code));
        }
        Ok(())
    }
}
