mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use crate::{
    FiscalYearId,
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
}

impl<Perms> Clone for FiscalYears<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms> FiscalYears<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        Self {
            repo: FiscalYearRepo::new(pool),
            authz: authz.clone(),
        }
    }

    #[instrument(
        name = "core_accounting.fiscal_year.open_first_fiscal_year"
        skip(self),
        err
    )]
    pub async fn open_first_fiscal_year(
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
                CoreAccountingAction::FISCAL_YEAR_OPEN_FIRST,
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

        // TODO: Call ChartOfAccounts to close_as_of.

        Ok(fiscal_year)
    }

    #[instrument(name = "core_accounting.fiscal_year.close_month", skip(self), err)]
    pub async fn close_month(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        let _now = crate::time::now();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_CLOSE,
            )
            .await?;

        let mut latest_year = self.get_current_by_chart_id(chart_id).await?;
        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut latest_year).await?;
        // TODO: Call ChartOfAccounts to close_as_of.

        Ok(latest_year)
    }

    #[instrument(
        name = "core_accounting.fiscal_year.get_current_fiscal_year",
        skip(self),
        err
    )]
    pub async fn get_current_fiscal_year(
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
