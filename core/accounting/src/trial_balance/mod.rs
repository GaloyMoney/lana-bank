pub mod error;
pub mod ledger;

use std::collections::HashSet;

use chrono::NaiveDate;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;

use crate::{
    chart_of_accounts::Chart,
    primitives::{
        AccountCode, BalanceRange, CalaAccountSetId, CoreAccountingAction, CoreAccountingObject,
        DebitOrCredit, LedgerAccountId,
    },
};

use error::*;
pub use ledger::TrialBalanceRoot;
use ledger::*;

#[derive(Clone)]
pub struct TrialBalances<Perms>
where
    Perms: PermissionCheck,
{
    pool: sqlx::PgPool,
    authz: Perms,
    trial_balance_ledger: TrialBalanceLedger,
}

impl<Perms> TrialBalances<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Self {
        let trial_balance_ledger = TrialBalanceLedger::new(cala, journal_id);

        Self {
            pool: pool.clone(),
            trial_balance_ledger,
            authz: authz.clone(),
        }
    }

    #[instrument(name = "core_accounting.trial_balance.create", skip(self), err)]
    pub async fn create_trial_balance_statement(
        &self,
        reference: String,
    ) -> Result<(), TrialBalanceError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreAccountingObject::all_trial_balance(),
                CoreAccountingAction::TRIAL_BALANCE_CREATE,
            )
            .await?;

        match self.trial_balance_ledger.create(op, &reference).await {
            Ok(_) => Ok(()),
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "core_accounting.trial_balance.add_new_chart", skip(self), err)]
    pub async fn add_new_chart_accounts_to_trial_balance(
        &self,
        name: &str,
        new_chart_account_set_ids: &[CalaAccountSetId],
    ) -> Result<(), TrialBalanceError> {
        let trial_balance_id = self
            .trial_balance_ledger
            .get_id_from_reference(name.to_string())
            .await?;

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreAccountingObject::all_trial_balance(),
                CoreAccountingAction::TRIAL_BALANCE_UPDATE,
            )
            .await?;

        self.trial_balance_ledger
            .add_members(op, trial_balance_id, new_chart_account_set_ids.iter())
            .await?;

        Ok(())
    }

    #[instrument(name = "core_accounting.trial_balance.trial_balance", skip(self), err)]
    pub async fn trial_balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        from: NaiveDate,
        until: NaiveDate,
    ) -> Result<TrialBalanceRoot, TrialBalanceError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_trial_balance(),
                CoreAccountingAction::TRIAL_BALANCE_READ,
            )
            .await?;

        Ok(self
            .trial_balance_ledger
            .get_trial_balance(name, from, Some(until))
            .await?)
    }

    #[instrument(
        name = "core_accounting.trial_balance.accounts_flat_for_chart",
        skip(self, chart),
        err
    )]
    pub async fn accounts_flat_for_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<Vec<TrialBalanceRow>, TrialBalanceError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_trial_balance(),
                CoreAccountingAction::TRIAL_BALANCE_READ,
            )
            .await?;
        let chart_tree = chart.chart();
        let root_ids: Vec<_> = chart_tree
            .children
            .iter()
            .map(|node| LedgerAccountId::from(node.id))
            .collect();

        let mut root_accounts_with_activity: HashSet<LedgerAccountId> = self
            .trial_balance_ledger
            .load_accounts_in_range(&root_ids, from, until)
            .await?
            .into_iter()
            .map(|account| account.id)
            .collect();

        let mut ordered_ids = Vec::new();
        for node in &chart_tree.children {
            let ledger_id = LedgerAccountId::from(node.id);
            if !root_accounts_with_activity.remove(&ledger_id) {
                continue;
            }
            ordered_ids.push(ledger_id);
            ordered_ids.extend(node.descendants().into_iter().map(LedgerAccountId::from));
        }

        Ok(self
            .trial_balance_ledger
            .load_accounts_in_range(&ordered_ids, from, until)
            .await?)
    }
}

#[derive(Debug, Clone)]
pub struct TrialBalanceRow {
    pub id: LedgerAccountId,
    pub name: String,
    pub code: Option<AccountCode>,
    pub normal_balance_type: DebitOrCredit,
    pub usd_balance_range: Option<BalanceRange>,
    pub btc_balance_range: Option<BalanceRange>,
}

impl TrialBalanceRow {
    pub fn has_non_zero_activity(&self) -> bool {
        if let Some(usd) = self.usd_balance_range.as_ref() {
            usd.has_non_zero_activity()
        } else if let Some(btc) = self.btc_balance_range.as_ref() {
            btc.has_non_zero_activity()
        } else {
            false
        }
    }
}
