mod entity;
pub mod error;
mod ledger;
mod primitives;
mod repo;

use tracing::instrument;

use crate::primitives::{CoreAccountingAction, CoreAccountingObject};
use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{AccountSetId, CalaLedger, JournalId};
use error::*;
use ledger::{ClosingLedger, EntryParams};

pub use entity::LedgerClosing;
#[cfg(feature = "json-schema")]
pub use entity::LedgerClosingEvent;
pub(super) use entity::*;
pub use repo::ledger_closing_cursor::LedgerClosingsByCreatedAtCursor;
use repo::*;

#[derive(Clone)]
pub struct LedgerClosings<Perms>
where
    Perms: PermissionCheck,
{
    ledger: ClosingLedger,
    authz: Perms,
    journal_id: JournalId,
    root_account_set_id: AccountSetId,
    repo: LedgerClosingRepo,
}

impl<Perms> LedgerClosings<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        root_account_set_id: AccountSetId,
        cala: &CalaLedger,
        journal_id: JournalId,
    ) -> Self {
        let repo = LedgerClosingRepo::new(pool);
        Self {
            ledger: ClosingLedger::new(cala, root_account_set_id),
            root_account_set_id,
            authz: authz.clone(),
            journal_id,
            repo,
        }
    }

    #[instrument(
        name = "core_accounting.ledger_closings.close_last_period",
        skip(self),
        err
    )]
    pub async fn close_last_period(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }

    async fn get_chart_of_accounts_integration_config(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }

    async fn execute_monthly_closing_operation(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }

    async fn execute_annual_closing_transaction(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }
}
