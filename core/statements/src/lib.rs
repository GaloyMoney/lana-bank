#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod auth;
pub mod error;
mod primitives;
mod trial_balance;

use cala_ledger::CalaLedger;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use auth::*;
use error::*;
pub use primitives::*;
use trial_balance::*;

pub struct CoreStatements<Perms>
where
    Perms: PermissionCheck,
{
    trial_balance_repo: TrialBalanceStatementRepo,
    trial_balance_ledger: TrialBalanceStatementLedger,
    authz: Perms,
    journal_id: LedgerJournalId,
}

impl<Perms> Clone for CoreStatements<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            trial_balance_repo: self.trial_balance_repo.clone(),
            trial_balance_ledger: self.trial_balance_ledger.clone(),
            authz: self.authz.clone(),
            journal_id: self.journal_id,
        }
    }
}

impl<Perms> CoreStatements<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreStatementsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreStatementsObject>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
    ) -> Result<Self, CoreStatementsError> {
        let trial_balance_repo = TrialBalanceStatementRepo::new(pool);
        let trial_balance_ledger = TrialBalanceStatementLedger::new(cala, journal_id);

        let res = Self {
            trial_balance_repo,
            trial_balance_ledger,
            authz: authz.clone(),
            journal_id,
        };
        Ok(res)
    }

    pub async fn create_trial_balance_statement(
        &self,
        id: impl Into<TrialBalanceStatementId>,
        name: String,
        reference: String,
    ) -> Result<TrialBalanceStatement, CoreStatementsError> {
        let id = id.into();
        let statement_id: StatementId = id.into();
        let account_set_id: LedgerAccountSetId = id.into();

        let mut op = self.trial_balance_repo.begin_op().await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreStatementsObject::statement(statement_id),
                CoreStatementsAction::STATEMENT_CREATE,
            )
            .await?;

        let new_trial_balance = NewTrialBalanceStatement::builder()
            .id(id)
            .name(name.to_string())
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let trial_balance = self
            .trial_balance_repo
            .create_in_op(&mut op, new_trial_balance)
            .await?;

        self.trial_balance_ledger
            .create(op, account_set_id, &name)
            .await?;

        Ok(trial_balance)
    }

    pub async fn find_by_reference(
        &self,
        reference: String,
    ) -> Result<Option<TrialBalanceStatement>, CoreStatementsError> {
        let mut op = self.trial_balance_repo.begin_op().await?;
        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreStatementsObject::all_statements(),
                CoreStatementsAction::STATEMENT_LIST,
            )
            .await?;

        let statement = match self.trial_balance_repo.find_by_reference(reference).await {
            Ok(statement) => Some(statement),
            Err(e) if e.was_not_found() => None,
            Err(e) => return Err(e.into()),
        };
        op.commit().await?;

        Ok(statement)
    }
}
