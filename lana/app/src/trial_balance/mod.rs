pub mod error;
pub mod ledger;
mod statement;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{Subject, TrialBalanceAction};

use crate::{
    authorization::{Authorization, Object},
    primitives::{Currency, LedgerAccountSetId, TrialBalanceId},
};

use error::*;
use ledger::*;
pub use statement::*;

#[derive(Clone)]
pub struct TrialBalances {
    pool: sqlx::PgPool,
    authz: Authorization,
    cala: CalaLedger,
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
            cala: cala.clone(),
        })
    }

    pub async fn create_trial_balance_statement(
        &self,
        id: impl Into<TrialBalanceId>,
        name: String,
    ) -> Result<LedgerAccountSetId, TrialBalanceError> {
        let account_set_id: LedgerAccountSetId = id.into().into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::TrialBalance, TrialBalanceAction::Create)
            .await?;

        self.trial_balance_ledger
            .create(op, account_set_id, &name)
            .await?;

        Ok(account_set_id)
    }

    pub async fn find_by_name(
        &self,
        name: String,
    ) -> Result<Option<LedgerAccountSetId>, TrialBalanceError> {
        self.authz
            .audit()
            .record_system_entry(Object::TrialBalance, TrialBalanceAction::Read)
            .await?;

        let trial_balances = self
            .trial_balance_ledger
            .list_for_name(name.to_string(), Default::default())
            .await?
            .entities;

        match trial_balances.len() {
            0 => Ok(None),
            1 => Ok(Some(trial_balances[0].id)),
            _ => Err(TrialBalanceError::MultipleFound(name)),
        }
    }

    pub async fn add_to_trial_balance(
        &self,
        trial_balance_id: impl Into<TrialBalanceId>,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), TrialBalanceError> {
        let trial_balance_id = trial_balance_id.into();
        let member_id = member_id.into();

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

    pub async fn trial_balance(
        &self,
        sub: &Subject,
        name: String,
        currency: Currency,
    ) -> Result<TrialBalance, TrialBalanceError> {
        self.authz
            .enforce_permission(sub, Object::TrialBalance, TrialBalanceAction::Read)
            .await?;

        let trial_balance_id = self
            .find_by_name(name.to_string())
            .await?
            .ok_or(TrialBalanceError::NotFound(name))?;

        let trial_balance_details = self
            .trial_balance_ledger
            .get_trial_balance(trial_balance_id, currency)
            .await?;

        Ok(TrialBalance::from(trial_balance_details))
    }
}

pub struct TrialBalance {
    pub id: TrialBalanceId,
    pub name: String,
    pub description: Option<String>,
    pub balance: StatementAccountSetBalance,
    pub accounts: Vec<StatementAccountSet>,
}

impl From<LedgerAccountSetDetailsWithAccounts> for TrialBalance {
    fn from(details: LedgerAccountSetDetailsWithAccounts) -> Self {
        Self {
            id: details.values.id.into(),
            name: details.values.name,
            description: details.values.description,
            balance: details.balance.into(),
            accounts: details
                .accounts
                .into_iter()
                .map(StatementAccountSet::from)
                .collect(),
        }
    }
}
