mod csv;
mod entity;
mod primitives;
mod repo;
pub mod tree;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{account_set::NewAccountSet, CalaLedger};
use tracing::instrument;

use super::error::*;

pub(crate) use csv::CsvParseError;
pub use entity::Chart;
use entity::*;
pub use primitives::*;
use repo::*;

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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreChartOfAccountsActionNew>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreChartOfAccountsObjectNew>,
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

    #[instrument(name = "chart_of_accounts.create_chart", skip(self))]
    pub async fn create_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        reference: String,
    ) -> Result<Chart, CoreChartOfAccountsError> {
        let id = ChartId::new();

        let mut op = self.repo.begin_op().await?;
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountsObjectNew::chart(id),
                CoreChartOfAccountsActionNew::CHART_CREATE,
            )
            .await?;

        let new_chart = NewChart::builder()
            .id(id)
            .name(name)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let chart = self.repo.create_in_op(&mut op, new_chart).await?;
        op.commit().await?;

        Ok(chart)
    }

    #[instrument(name = "chart_of_account.import_from_csv", skip(self, data))]
    pub async fn import_from_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
        data: impl AsRef<str>,
    ) -> Result<Chart, CoreChartOfAccountsError> {
        let id = id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreChartOfAccountsObjectNew::chart(id),
                CoreChartOfAccountsActionNew::CHART_LIST,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let data = data.as_ref().to_string();
        let account_specs = csv::CsvParser::new(data).account_specs()?;
        let mut new_account_sets = Vec::new();
        let mut new_connections = Vec::new();
        for spec in account_specs {
            if let es_entity::Idempotent::Executed((parent, set_id)) =
                chart.create_node(&spec, audit_info.clone())
            {
                let new_account_set = NewAccountSet::builder()
                    .id(set_id)
                    .journal_id(self.journal_id)
                    .name(spec.name.to_string())
                    .description(spec.name.to_string())
                    .external_id(spec.account_set_external_id(id))
                    // .normal_balance_type()
                    .build()
                    .expect("Could not build new account set");
                new_account_sets.push(new_account_set);
                if let Some(parent) = parent {
                    new_connections.push((parent, set_id));
                }
            }
        }
        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .account_sets()
            .create_all_in_op(&mut op, new_account_sets)
            .await?;

        for (parent, child) in new_connections {
            self.cala
                .account_sets()
                .add_member_in_op(&mut op, parent, child)
                .await?;
        }
        op.commit().await?;
        Ok(chart)
    }

    #[instrument(name = "chart_of_accounts.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Chart, CoreChartOfAccountsError> {
        Ok(self.repo.find_by_id(id.into()).await?)
    }

    #[instrument(name = "chart_of_accounts.find_by_reference", skip(self))]
    pub async fn find_by_reference(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
    ) -> Result<Option<Chart>, CoreChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreChartOfAccountsObjectNew::all_charts(),
                CoreChartOfAccountsActionNew::CHART_LIST,
            )
            .await?;

        let chart = match self.repo.find_by_reference(reference).await {
            Ok(chart) => Some(chart),
            Err(e) if e.was_not_found() => None,
            Err(e) => return Err(e.into()),
        };

        Ok(chart)
    }

    #[instrument(name = "chart_of_accounts.find_all", skip(self), err)]
    pub async fn find_all<T: From<Chart>>(
        &self,
        ids: &[ChartId],
    ) -> Result<std::collections::HashMap<ChartId, T>, CoreChartOfAccountsError> {
        Ok(self.repo.find_all(ids).await?)
    }
}
