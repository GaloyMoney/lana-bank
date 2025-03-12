pub mod error;
pub mod ledger;

use chart_of_accounts::Chart;
use chrono::{DateTime, Utc};

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{Subject, TrialBalanceAction};

use crate::{
    authorization::{Authorization, Object},
    primitives::LedgerAccountSetId,
    statement::*,
};

use error::*;
use ledger::*;

#[derive(Clone)]
pub struct TrialBalances {
    pool: sqlx::PgPool,
    authz: Authorization,
    trial_balance_ledger: TrialBalanceLedger,
}

impl TrialBalances {
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Result<Self, TrialBalanceError> {
        let trial_balance_ledger = TrialBalanceLedger::new(cala, journal_id);

        Ok(Self {
            pool: pool.clone(),
            trial_balance_ledger,
            authz: authz.clone(),
        })
    }

    pub async fn create_trial_balance_statement(
        &self,
        reference: String,
    ) -> Result<(), TrialBalanceError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::TrialBalance, TrialBalanceAction::Create)
            .await?;

        match self.trial_balance_ledger.create(op, &reference).await {
            Ok(_) => Ok(()),
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn add_to_trial_balance(
        &self,
        name: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), TrialBalanceError> {
        let member_id = member_id.into();

        let trial_balance_id = self
            .trial_balance_ledger
            .get_id_from_reference(name)
            .await?;

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::TrialBalance, TrialBalanceAction::Update)
            .await?;

        self.trial_balance_ledger
            .add_member(op, trial_balance_id, member_id)
            .await?;

        Ok(())
    }

    pub async fn add_chart_to_trial_balance(
        &self,
        name: String,
        chart: Chart,
    ) -> Result<(), TrialBalanceError> {
        let trial_balance_id = self
            .trial_balance_ledger
            .get_id_from_reference(name)
            .await?;

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::TrialBalance, TrialBalanceAction::Update)
            .await?;

        self.trial_balance_ledger
            .add_members(
                op,
                trial_balance_id,
                chart.all_non_top_level_accounts().map(|(_, id)| *id),
            )
            .await?;

        Ok(())
    }

    pub async fn trial_balance(
        &self,
        sub: &Subject,
        name: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<TrialBalance, TrialBalanceError> {
        self.authz
            .enforce_permission(sub, Object::TrialBalance, TrialBalanceAction::Read)
            .await?;

        Ok(self
            .trial_balance_ledger
            .get_trial_balance(name, from, until)
            .await?
            .into())
    }
}

#[derive(Debug, Clone)]
pub struct TrialBalance {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub accounts: Vec<StatementAccountSet>,
}

impl From<StatementAccountSetWithAccounts> for TrialBalance {
    fn from(details: StatementAccountSetWithAccounts) -> Self {
        Self {
            id: details.id,
            name: details.name,
            description: details.description,
            btc_balance: details.btc_balance,
            usd_balance: details.usd_balance,
            accounts: details.accounts,
        }
    }
}
