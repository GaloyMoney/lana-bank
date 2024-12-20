#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
mod code;
pub mod config;
pub mod error;
mod event;
mod ledger;
mod primitives;
mod transaction_account_factory;

use cala_ledger::CalaLedger;
use ledger::*;
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;

use chart_of_accounts::*;
use error::*;
pub use event::*;
pub use primitives::*;

pub struct CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    repo: ChartOfAccountRepo,
    ledger: ChartOfAccountLedger,
    authz: Perms,
}

impl<Perms> Clone for CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms> CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreChartOfAccountsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreChartOfAccountsObject>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
    ) -> Result<Self, CoreChartOfAccountError> {
        let chart_of_account = ChartOfAccountRepo::new(pool);
        let ledger = ChartOfAccountLedger::init(cala).await?;
        let res = Self {
            repo: chart_of_account,
            ledger,
            authz: authz.clone(),
        };
        Ok(res)
    }

    #[instrument(name = "chart_of_accounts.create_chart", skip(self))]
    pub async fn create_chart(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
        reference: String,
    ) -> Result<ChartOfAccount, CoreChartOfAccountError> {
        let id = id.into();

        let mut op = self.repo.begin_op().await?;
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreChartOfAccountsObject::chart(id),
                CoreChartOfAccountsAction::CHART_CREATE,
            )
            .await?;

        let new_chart_of_account = NewChartOfAccount::builder()
            .id(id)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let chart = self
            .repo
            .create_in_op(&mut op, new_chart_of_account)
            .await?;
        op.commit().await?;

        Ok(chart)
    }

    #[instrument(name = "chart_of_accounts.find_by_reference", skip(self))]
    pub async fn find_by_reference(
        &self,
        reference: String,
    ) -> Result<Option<ChartOfAccount>, CoreChartOfAccountError> {
        let mut op = self.repo.begin_op().await?;
        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreChartOfAccountsObject::all_charts(),
                CoreChartOfAccountsAction::CHART_LIST,
            )
            .await?;

        let chart = match self.repo.find_by_reference(reference).await {
            Ok(chart) => Some(chart),
            Err(e) if e.was_not_found() => None,
            Err(e) => return Err(e.into()),
        };
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
                CoreChartOfAccountsObject::all_charts(),
                CoreChartOfAccountsAction::CHART_LIST,
            )
            .await?;

        Ok(self
            .repo
            .list_by_id(Default::default(), es_entity::ListDirection::Ascending)
            .await?
            .entities)
    }

    pub async fn create_control_account(
        &self,
        chart_id: impl Into<ChartId>,
        category: ChartOfAccountCode,
        name: &str,
    ) -> Result<ChartOfAccountCode, CoreChartOfAccountError> {
        let chart_id = chart_id.into();

        let mut op = self.repo.begin_op().await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreChartOfAccountsObject::chart(chart_id),
                CoreChartOfAccountsAction::CHART_CREATE_CONTROL_ACCOUNT,
            )
            .await?;

        let mut chart = self.repo.find_by_id(chart_id).await?;

        let code = chart.create_control_account(category, name, audit_info)?;

        self.repo.update_in_op(&mut op, &mut chart).await?;

        op.commit().await?;

        Ok(code)
    }

    pub async fn create_control_sub_account(
        &self,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        control_account: ChartOfAccountCode,
        name: &str,
    ) -> Result<ChartOfAccountCode, CoreChartOfAccountError> {
        let chart_id = chart_id.into();

        let mut op = self.repo.begin_op().await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreChartOfAccountsObject::chart(chart_id),
                CoreChartOfAccountsAction::CHART_CREATE_CONTROL_SUB_ACCOUNT,
            )
            .await?;

        let mut chart = self.repo.find_by_id(chart_id).await?;

        let code = chart.create_control_sub_account(control_account, name, audit_info)?;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        op.commit().await?;

        Ok(code)
    }

    pub async fn create_transaction_account_in_op(
        &self,
        mut op: es_entity::DbOp<'_>,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        account_id: impl Into<LedgerAccountId>,
        control_sub_account: ChartOfAccountCode,
        name: &str,
        description: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountAccountDetails, CoreChartOfAccountError> {
        let chart_id = chart_id.into();

        let mut chart = self.repo.find_by_id(chart_id).await?;

        let account_details = chart.create_transaction_account(
            account_id,
            control_sub_account,
            name,
            description,
            audit_info,
        )?;

        self.repo.update_in_op(&mut op, &mut chart).await?;

        self.ledger
            .create_transaction_account(op, &account_details)
            .await?;

        Ok(account_details)
    }

    #[instrument(name = "chart_of_accounts.find_account_in_chart", skip(self))]
    pub async fn find_account_in_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        code: impl Into<ChartOfAccountCode> + std::fmt::Debug,
    ) -> Result<Option<ChartOfAccountAccountDetails>, CoreChartOfAccountError> {
        let chart_id = chart_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreChartOfAccountsObject::chart(chart_id),
                CoreChartOfAccountsAction::CHART_FIND_TRANSACTION_ACCOUNT,
            )
            .await?;

        let chart = self.repo.find_by_id(chart_id).await?;

        let account_details = chart.find_account(code.into());

        Ok(account_details)
    }
}
