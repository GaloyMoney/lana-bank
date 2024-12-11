#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
pub mod error;
mod event;
mod ledger;
mod primitives;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use outbox::{Outbox, OutboxEventMarker};

use chart_of_accounts::*;
use error::*;
use ledger::*;
pub use primitives::*;

pub struct CoreChartOfAccounts<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountsEvent>,
{
    chart_of_accounts: ChartOfAccountsRepo,
    ledger: ChartOfAccountsLedger,
    authz: Perms,
    outbox: Outbox<E>,
}

impl<Perms, E> Clone for CoreChartOfAccounts<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountsEvent>,
{
    fn clone(&self) -> Self {
        Self {
            chart_of_accounts: self.chart_of_accounts.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
        }
    }
}

impl<Perms, E> CoreChartOfAccounts<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreChartOfAccountsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreChartOfAccountsObject>,
    E: OutboxEventMarker<CoreChartOfAccountsEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
    ) -> Result<Self, CoreChartOfAccountsError> {
        let chart_of_accounts = ChartOfAccountsRepo::new(pool);
        let ledger = ChartOfAccountsLedger::init(cala, journal_id).await?;
        let res = Self {
            chart_of_accounts,
            authz: authz.clone(),
            outbox: outbox.clone(),
            ledger,
        };
        Ok(res)
    }
}
