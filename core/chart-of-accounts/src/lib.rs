#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
pub mod error;
mod path;
mod primitives;
mod transaction_account_factory;

use cala_ledger::CalaLedger;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use chart_of_accounts::*;
use error::*;
use path::ControlAccountPath;
pub use path::ControlSubAccountPath;
pub use primitives::*;
pub use transaction_account_factory::*;

pub struct CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    repo: ChartRepo,
    cala: CalaLedger,
    authz: Perms,
}

impl<Perms> Clone for CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            cala: self.cala.clone(),
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
    ) -> Result<Self, CoreChartOfAccountsError> {
        let chart_of_account = ChartRepo::new(pool);
        let res = Self {
            repo: chart_of_account,
            cala: cala.clone(),
            authz: authz.clone(),
        };
        Ok(res)
    }

    pub fn transaction_account_factory(
        &self,
        chart_id: ChartId,
        control_sub_account: ControlSubAccountPath,
    ) -> TransactionAccountFactory {
        TransactionAccountFactory::new(&self.repo, &self.cala, chart_id, control_sub_account)
    }

    #[instrument(name = "chart_of_accounts.create_chart", skip(self))]
    pub async fn create_chart(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
        reference: String,
    ) -> Result<Chart, CoreChartOfAccountsError> {
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

        let new_chart_of_account = NewChart::builder()
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
    ) -> Result<Option<Chart>, CoreChartOfAccountsError> {
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
    ) -> Result<Vec<Chart>, CoreChartOfAccountsError> {
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

    pub async fn find_control_account_by_reference(
        &self,
        chart_id: impl Into<ChartId>,
        reference: String,
    ) -> Result<Option<ControlAccountPath>, CoreChartOfAccountsError> {
        let chart_id = chart_id.into();

        let mut op = self.repo.begin_op().await?;
        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreChartOfAccountsObject::chart(chart_id),
                CoreChartOfAccountsAction::CHART_FIND_CONTROL_ACCOUNT,
            )
            .await?;
        op.commit().await?;

        let chart = self.repo.find_by_id(chart_id).await?;

        Ok(chart.find_control_account_by_reference(reference))
    }

    pub async fn create_control_account(
        &self,
        chart_id: impl Into<ChartId>,
        category: ChartCategory,
        name: String,
        reference: String,
    ) -> Result<ControlAccountPath, CoreChartOfAccountsError> {
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

        let path = chart.create_control_account(category, name, reference, audit_info)?;

        self.repo.update_in_op(&mut op, &mut chart).await?;

        op.commit().await?;

        Ok(path)
    }

    pub async fn find_control_sub_account_by_reference(
        &self,
        chart_id: impl Into<ChartId>,
        reference: String,
    ) -> Result<Option<ControlSubAccountPath>, CoreChartOfAccountsError> {
        let chart_id = chart_id.into();

        let mut op = self.repo.begin_op().await?;
        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreChartOfAccountsObject::chart(chart_id),
                CoreChartOfAccountsAction::CHART_FIND_CONTROL_SUB_ACCOUNT,
            )
            .await?;
        op.commit().await?;

        let chart = self.repo.find_by_id(chart_id).await?;

        Ok(chart.find_control_sub_account_by_reference(reference))
    }

    pub async fn create_control_sub_account(
        &self,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        control_account: ControlAccountPath,
        name: String,
        reference: String,
    ) -> Result<ControlSubAccountPath, CoreChartOfAccountsError> {
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

        let path =
            chart.create_control_sub_account(control_account, name, reference, audit_info)?;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        op.commit().await?;

        Ok(path)
    }

    #[instrument(name = "chart_of_accounts.find_account_in_chart", skip(self))]
    pub async fn find_account_in_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        encoded_path: String,
    ) -> Result<Option<ChartAccountDetails>, CoreChartOfAccountsError> {
        let chart_id = chart_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreChartOfAccountsObject::chart(chart_id),
                CoreChartOfAccountsAction::CHART_FIND_TRANSACTION_ACCOUNT,
            )
            .await?;

        let chart = self.repo.find_by_id(chart_id).await?;

        let account_details = chart.find_account_by_encoded_path(encoded_path);

        Ok(account_details)
    }
}
