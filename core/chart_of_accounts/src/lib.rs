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

use es_entity::DbOp;

use chart_of_accounts::*;
use code::*;
use error::*;
pub use event::*;
pub use primitives::*;

pub struct CoreChartOfAccount<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreChartOfAccountEvent>,
{
    chart_of_account: ChartOfAccountRepo,
    ledger: ChartOfAccountLedger,
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
            ledger: self.ledger.clone(),
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

    async fn find_or_create(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        op: &mut DbOp<'static>,
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

        match self.chart_of_account.find_by_id(id).await {
            Ok(chart_of_account) => return Ok(chart_of_account),
            Err(ChartOfAccountError::EsEntityError(es_entity::EsEntityError::NotFound)) => (),
            Err(e) => return Err(e.into()),
        };

        if let Ok(chart_of_account) = self.chart_of_account.find_by_id(id).await {
            return Ok(chart_of_account);
        };

        let new_chart_of_account = NewChartOfAccount::builder()
            .id(id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let chart_of_account = self
            .chart_of_account
            .create_in_op(op, new_chart_of_account)
            .await?;

        Ok(chart_of_account)
    }

    #[instrument(name = "chart_of_accounts.create_control_account", skip(self))]
    pub async fn create_control_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        category: ChartOfAccountCode,
        name: &str,
    ) -> Result<ChartOfAccountCode, CoreChartOfAccountError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart_of_account(),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE_CONTROL_ACCOUNT,
            )
            .await?;

        let mut op = self.chart_of_account.begin_op().await?;

        let mut chart_of_accounts = self.find_or_create(sub, &mut op).await?;

        let code = chart_of_accounts.create_control_account(category, name, audit_info)?;

        self.chart_of_account
            .update_in_op(&mut op, &mut chart_of_accounts)
            .await?;

        op.commit().await?;

        Ok(code)
    }

    #[instrument(name = "chart_of_accounts.create_control_sub_account", skip(self))]
    pub async fn create_control_sub_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        control_account: ChartOfAccountCode,
        name: &str,
    ) -> Result<ChartOfAccountCode, CoreChartOfAccountError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart_of_account(),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE_CONTROL_SUB_ACCOUNT,
            )
            .await?;

        let mut op = self.chart_of_account.begin_op().await?;

        let mut chart_of_accounts = self.find_or_create(sub, &mut op).await?;

        let code =
            chart_of_accounts.create_control_sub_account(control_account, name, audit_info)?;

        self.chart_of_account
            .update_in_op(&mut op, &mut chart_of_accounts)
            .await?;

        op.commit().await?;

        Ok(code)
    }

    #[instrument(name = "chart_of_accounts.create_account", skip(self))]
    pub async fn create_transaction_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        control_sub_account: ChartOfAccountCode,
        name: &str,
        description: &str,
    ) -> Result<ChartOfAccountAccountDetails, CoreChartOfAccountError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart_of_account(),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_CREATE_TRANSACTION_ACCOUNT,
            )
            .await?;

        let mut op = self.chart_of_account.begin_op().await?;

        let mut chart_of_accounts = self.find_or_create(sub, &mut op).await?;

        let account_details = chart_of_accounts.create_transaction_account(
            control_sub_account,
            name,
            description,
            audit_info,
        )?;

        self.chart_of_account
            .update_in_op(&mut op, &mut chart_of_accounts)
            .await?;

        self.ledger
            .create_transaction_account(op, &account_details)
            .await?;

        Ok(account_details)
    }

    #[instrument(name = "chart_of_accounts.find_account", skip(self))]
    pub async fn find_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        code: impl Into<ChartOfAccountCode> + std::fmt::Debug,
    ) -> Result<Option<ChartOfAccountAccountDetails>, CoreChartOfAccountError> {
        self.authz
            .enforce_permission(
                sub,
                CoreChartOfAccountObject::chart_of_account(),
                CoreChartOfAccountAction::CHART_OF_ACCOUNT_FIND_TRANSACTION_ACCOUNT,
            )
            .await?;

        let mut op = self.chart_of_account.begin_op().await?;
        let chart_of_accounts = self.find_or_create(sub, &mut op).await?;
        op.commit().await?;

        let account_details = chart_of_accounts.find_account(code.into());

        Ok(account_details)
    }
}
