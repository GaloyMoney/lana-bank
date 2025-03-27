pub mod error;
pub mod ledger;

use chrono::{DateTime, Utc};
use core_accounting::Chart;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{Subject, TrialBalanceAction};

use crate::authorization::{Authorization, Object};

use error::*;
use ledger::*;
pub use ledger::{TrialBalanceAccountSet, TrialBalanceAccountSetsCursor, TrialBalanceRoot};

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
                chart.all_trial_balance_accounts().map(|(_, id)| *id),
            )
            .await?;

        Ok(())
    }

    pub async fn trial_balance(
        &self,
        sub: &Subject,
        name: String,
        from: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<TrialBalanceRoot, TrialBalanceError> {
        self.authz
            .enforce_permission(sub, Object::TrialBalance, TrialBalanceAction::Read)
            .await?;

        Ok(self
            .trial_balance_ledger
            .get_trial_balance(name, from, Some(until))
            .await?)
    }

    pub async fn trial_balance_accounts(
        &self,
        sub: &Subject,
        name: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
        args: es_entity::PaginatedQueryArgs<TrialBalanceAccountSetsCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<TrialBalanceAccountSet, TrialBalanceAccountSetsCursor>,
        TrialBalanceError,
    > {
        self.authz
            .enforce_permission(sub, Object::TrialBalance, TrialBalanceAction::Read)
            .await?;

        Ok(self
            .trial_balance_ledger
            .accounts(name, from, until, args)
            .await?)
    }
}
