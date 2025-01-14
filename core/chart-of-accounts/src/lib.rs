#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod chart_of_accounts;
pub mod error;
mod path;
mod primitives;
mod transaction_account_factory;

use cala_ledger::{account_set::NewAccountSet, CalaLedger};
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
    journal_id: LedgerJournalId,
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
            journal_id: self.journal_id,
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
        journal_id: LedgerJournalId,
    ) -> Result<Self, CoreChartOfAccountsError> {
        let chart_of_account = ChartRepo::new(pool);
        let res = Self {
            repo: chart_of_account,
            cala: cala.clone(),
            authz: authz.clone(),
            journal_id,
        };
        Ok(res)
    }

    pub fn transaction_account_factory(
        &self,
        chart_id: ChartId,
        control_sub_account: ControlSubAccountDetails,
    ) -> TransactionAccountFactory {
        TransactionAccountFactory::new(&self.repo, &self.cala, chart_id, control_sub_account)
    }

    #[instrument(name = "chart_of_accounts.create_chart", skip(self))]
    pub async fn create_chart(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
        name: String,
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
            .name(name)
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
    ) -> Result<Option<ControlSubAccountDetails>, CoreChartOfAccountsError> {
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
        id: impl Into<LedgerAccountSetId> + std::fmt::Debug,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        control_account: ControlAccountPath,
        name: String,
        reference: String,
    ) -> Result<ControlSubAccountDetails, CoreChartOfAccountsError> {
        let id = id.into();
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

        let account_set_details =
            chart.create_control_sub_account(id, control_account, name, reference, audit_info)?;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self.cala.ledger_operation_from_db_op(op);
        let new_account_set = NewAccountSet::builder()
            .id(account_set_details.account_set_id)
            .journal_id(self.journal_id)
            .name(account_set_details.name.to_string())
            .description(account_set_details.name.to_string())
            .normal_balance_type(account_set_details.path.normal_balance_type())
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;

        op.commit().await?;

        Ok(account_set_details)
    }
}
