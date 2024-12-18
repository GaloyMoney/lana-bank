#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
mod code;
pub mod error;
mod event;
mod ledger;
mod primitives;

use cala_ledger::CalaLedger;
use ledger::*;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

use chart_of_accounts::*;
use code::*;
use error::*;
pub use event::*;
pub use primitives::*;

pub struct CoreChartOfAccounts<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountEvent>,
{
    chart_of_account: ChartOfAccountRepo,
    ledger: ChartOfAccountLedger,
    authz: Perms,
    outbox: Outbox<E>,
}

impl<Perms, E> Clone for CoreChartOfAccounts<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountEvent>,
{
    fn clone(&self) -> Self {
        Self {
            chart_of_account: self.chart_of_account.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
        }
    }
}

impl<Perms, E> CoreChartOfAccounts<Perms, E>
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
        cala: &CalaLedger,
    ) -> Result<Self, CoreChartOfAccountError> {
        let chart_of_account = ChartOfAccountRepo::new(pool);
        let ledger = ChartOfAccountLedger::init(cala).await?;
        let res = Self {
            chart_of_account,
            ledger,
            authz: authz.clone(),
            outbox: outbox.clone(),
        };
        Ok(res)
    }

    #[instrument(name = "chart_of_accounts.create_chart", skip(self))]
    pub async fn create_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartOfAccountId> + std::fmt::Debug,
    ) -> Result<ChartOfAccount, CoreChartOfAccountError> {
        let id = id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart(id),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE,
            )
            .await?;

        let new_chart_of_account = NewChartOfAccount::builder()
            .id(id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let mut op = self.chart_of_account.begin_op().await?;
        let chart = self
            .chart_of_account
            .create_in_op(&mut op, new_chart_of_account)
            .await?;
        op.commit().await?;

        Ok(chart)
    }

    #[instrument(name = "core_user.list_charts", skip(self))]
    pub async fn list_charts(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Vec<ChartOfAccount>, CoreChartOfAccountError> {
        self.authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::all_charts(),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_LIST,
            )
            .await?;

        Ok(self
            .chart_of_account
            .list_by_id(Default::default(), es_entity::ListDirection::Ascending)
            .await?
            .entities)
    }

    #[instrument(name = "chart_of_accounts.create_control_account", skip(self))]
    pub async fn create_control_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartOfAccountId> + std::fmt::Debug,
        category: ChartOfAccountCode,
        name: &str,
    ) -> Result<ChartOfAccountCode, CoreChartOfAccountError> {
        let chart_id = chart_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart(chart_id),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE_CONTROL_ACCOUNT,
            )
            .await?;

        let mut chart = self.chart_of_account.find_by_id(chart_id).await?;

        let code = chart.create_control_account(category, name, audit_info)?;

        let mut op = self.chart_of_account.begin_op().await?;
        self.chart_of_account
            .update_in_op(&mut op, &mut chart)
            .await?;

        op.commit().await?;

        Ok(code)
    }

    #[instrument(name = "chart_of_accounts.create_control_sub_account", skip(self))]
    pub async fn create_control_sub_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartOfAccountId> + std::fmt::Debug,
        control_account: ChartOfAccountCode,
        name: &str,
    ) -> Result<ChartOfAccountCode, CoreChartOfAccountError> {
        let chart_id = chart_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart(chart_id),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE_CONTROL_SUB_ACCOUNT,
            )
            .await?;

        let mut chart = self.chart_of_account.find_by_id(chart_id).await?;

        let code = chart.create_control_sub_account(control_account, name, audit_info)?;

        let mut op = self.chart_of_account.begin_op().await?;
        self.chart_of_account
            .update_in_op(&mut op, &mut chart)
            .await?;

        op.commit().await?;

        Ok(code)
    }

    #[instrument(name = "chart_of_accounts.create_transaction_account", skip(self))]
    pub async fn create_transaction_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartOfAccountId> + std::fmt::Debug,
        control_sub_account: ChartOfAccountCode,
        name: &str,
        description: &str,
    ) -> Result<ChartOfAccountAccountDetails, CoreChartOfAccountError> {
        let chart_id = chart_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart(chart_id),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE_TRANSACTION_ACCOUNT,
            )
            .await?;

        let mut chart = self.chart_of_account.find_by_id(chart_id).await?;

        let account_details =
            chart.create_transaction_account(control_sub_account, name, description, audit_info)?;

        let mut op = self.chart_of_account.begin_op().await?;
        self.chart_of_account
            .update_in_op(&mut op, &mut chart)
            .await?;

        self.ledger
            .create_transaction_account(op, &account_details)
            .await?;

        Ok(account_details)
    }

    #[instrument(name = "chart_of_accounts.find_account_in_chart", skip(self))]
    pub async fn find_account_in_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartOfAccountId> + std::fmt::Debug,
        code: impl Into<ChartOfAccountCode> + std::fmt::Debug,
    ) -> Result<Option<ChartOfAccountAccountDetails>, CoreChartOfAccountError> {
        let chart_id = chart_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart(chart_id),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_FIND_TRANSACTION_ACCOUNT,
            )
            .await?;

        let chart = self.chart_of_account.find_by_id(chart_id).await?;

        let account_details = chart.find_account(code.into());

        Ok(account_details)
    }
}
