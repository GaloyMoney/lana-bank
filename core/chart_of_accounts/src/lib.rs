#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
pub mod error;
mod event;
mod primitives;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

use chart_of_accounts::*;
use error::*;
pub use event::*;
pub use primitives::*;

pub struct CoreChartOfAccount<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountEvent>,
{
    chart_of_account: ChartOfAccountRepo,
    authz: Perms,
    outbox: Outbox<E>,
}

impl<Perms, E> Clone for CoreChartOfAccount<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountEvent>,
{
    fn clone(&self) -> Self {
        Self {
            chart_of_account: self.chart_of_account.clone(),
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
        }
    }
}

impl<Perms, E> CoreChartOfAccount<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreChartOfAccountAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreChartOfAccountObject>,
    E: OutboxEventMarker<CoreChartOfAccountEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
    ) -> Result<Self, CoreChartOfAccountError> {
        let chart_of_account = ChartOfAccountRepo::new(pool);
        let res = Self {
            chart_of_account,
            authz: authz.clone(),
            outbox: outbox.clone(),
        };
        Ok(res)
    }
}
