#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
pub mod error;
mod event;
mod primitives;

use tracing::instrument;

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

    #[instrument(name = "chart_of_accounts.find_or_create", skip(self))]
    async fn find_or_create(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<ChartOfAccount, CoreChartOfAccountError> {
        let id = ChartOfAccountId::default();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart_of_account(),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_FIND_OR_CREATE,
            )
            .await?;

        if let Ok(chart_of_account) = self.chart_of_account.find_by_id(id).await {
            return Ok(chart_of_account);
        };

        let new_chart_of_account = NewChartOfAccount::builder()
            .id(id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let mut op = self.chart_of_account.begin_op().await?;
        let chart_of_account = self
            .chart_of_account
            .create_in_op(&mut op, new_chart_of_account)
            .await?;
        Ok(chart_of_account)
    }
}
