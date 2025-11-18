mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::Idempotent;

use crate::{
    FiscalYearId,
    chart_of_accounts::ChartOfAccounts,
    primitives::{ChartId, CoreAccountingAction, CoreAccountingObject},
};

pub use entity::FiscalYear;
#[cfg(feature = "json-schema")]
pub use entity::FiscalYearEvent;
pub(super) use entity::*;
use error::*;
pub(super) use repo::{fiscal_year_cursor::FiscalYearsByCreatedAtCursor, *};

pub struct FiscalYears<Perms>
where
    Perms: PermissionCheck,
{
    repo: FiscalYearRepo,
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
}

impl<Perms> Clone for FiscalYears<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            chart_of_accounts: self.chart_of_accounts.clone(),
        }
    }
}

impl<Perms> FiscalYears<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        chart_of_accounts: &ChartOfAccounts<Perms>,
    ) -> Self {
        Self {
            repo: FiscalYearRepo::new(pool),
            authz: authz.clone(),
            chart_of_accounts: chart_of_accounts.clone(),
        }
    }

    #[instrument(
        name = "core_accounting.fiscal_year.init_fiscal_year_for_chart"
        skip(self),
        err
    )]
    pub async fn init_fiscal_year_for_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        opened_as_of: impl Into<chrono::NaiveDate> + std::fmt::Debug,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        let opened_as_of = opened_as_of.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_INIT,
            )
            .await?;

        if let Ok(fiscal_year) = self.get_current_by_chart_id(chart_id).await {
            return Ok(fiscal_year);
        }

        tracing::info!("Initializing first FiscalYear for chart ID: {}", chart_id);
        let new_fiscal_year = NewFiscalYear::builder()
            .id(FiscalYearId::new())
            .chart_id(chart_id)
            .opened_as_of(opened_as_of)
            .build()
            .expect("Could not build new FiscalYear");

        let mut op = self.repo.begin_op().await?;
        let fiscal_year = self.repo.create_in_op(&mut op, new_fiscal_year).await?;
        let close_ledger_as_of = opened_as_of
            .pred_opt()
            .expect("Date was first possible NaiveDate type value");
        self.chart_of_accounts
            .close_by_chart_id_as_of(op, sub, chart_id, close_ledger_as_of)
            .await?;

        Ok(fiscal_year)
    }

    #[instrument(
        name = "core_accounting.fiscal_year.close_month_on_open_fiscal_year_for_chart",
        skip(self),
        err
    )]
    pub async fn close_month_on_open_fiscal_year_for_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_CLOSE_MONTH,
            )
            .await?;
        let now = crate::time::now();
        let mut latest_year = self.get_current_by_chart_id(chart_id).await?;
        let closed_as_of_date =
            if let Idempotent::Executed(date) = latest_year.close_last_month(now) {
                date
            } else {
                return Ok(latest_year);
            };

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut latest_year).await?;
        self.chart_of_accounts
            .close_by_chart_id_as_of(op, sub, chart_id, closed_as_of_date)
            .await?;

        Ok(latest_year)
    }

    #[instrument(
        name = "core_accounting.fiscal_year.get_current_fiscal_year_by_chart",
        skip(self),
        err
    )]
    pub async fn get_current_fiscal_year_by_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_READ,
            )
            .await?;
        self.get_current_by_chart_id(chart_id).await
    }

    async fn get_current_by_chart_id(
        &self,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.repo
            .list_for_chart_id_by_created_at(
                chart_id,
                es_entity::PaginatedQueryArgs::<FiscalYearsByCreatedAtCursor> {
                    first: 1,
                    after: None,
                },
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities
            .first()
            .cloned()
            .ok_or(FiscalYearError::CurrentYearNotFoundByChartId(chart_id))
    }

    #[instrument(name = "core_accounting.fiscal_year.find_all", skip(self), err)]
    pub async fn find_all<T: From<FiscalYear>>(
        &self,
        ids: &[FiscalYearId],
    ) -> Result<std::collections::HashMap<FiscalYearId, T>, FiscalYearError> {
        self.repo.find_all(ids).await
    }
}
