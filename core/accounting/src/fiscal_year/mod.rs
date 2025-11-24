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

#[cfg(feature = "json-schema")]
pub use entity::FiscalYearEvent;
pub(super) use entity::*;
pub use entity::{FiscalMonthClosure, FiscalYear};
use error::*;
pub use repo::{fiscal_year_cursor::FiscalYearsByCreatedAtCursor, *};

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

        if let Ok(existing) = self.repo.find_by_chart_id(chart_id).await {
            return Ok(existing);
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
            .close_as_of(op, sub, chart_id, close_ledger_as_of)
            .await?;

        Ok(fiscal_year)
    }

    #[instrument(
        name = "core_accounting.fiscal_year.close_and_open_next",
        skip(self),
        err
    )]
    pub async fn close_and_open_next(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        fiscal_year_id: impl Into<FiscalYearId> + std::fmt::Debug + Copy,
    ) -> Result<FiscalYear, FiscalYearError> {
        let id = fiscal_year_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::fiscal_year(id),
                CoreAccountingAction::FISCAL_YEAR_INIT,
            )
            .await?;
        let now = crate::time::now();

        let mut fiscal_year = self.repo.find_by_id(id).await?;
        match fiscal_year.close_and_open_next(now)? {
            Idempotent::Executed(next_fiscal_year) => {
                let mut op = self.repo.begin_op().await?;
                let new_fiscal_year = self.repo.create_in_op(&mut op, next_fiscal_year).await?;
                // TODO: Use `ChartLedger` via `ChartOfAccounts` to transfer
                // net income from `ProfitAndLossStatement` to `BalanceSheet`.
                op.commit().await?;
                Ok(new_fiscal_year)
            }
            Idempotent::Ignored => return Ok(fiscal_year),
        }
    }

    #[instrument(name = "core_accounting.fiscal_year.close_month", skip(self), err)]
    pub async fn close_month(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        fiscal_year_id: impl Into<FiscalYearId> + std::fmt::Debug + Copy,
    ) -> Result<FiscalYear, FiscalYearError> {
        let id = fiscal_year_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::fiscal_year(id),
                CoreAccountingAction::FISCAL_YEAR_CLOSE_MONTH,
            )
            .await?;
        let now = crate::time::now();

        let mut fiscal_year = self.repo.find_by_id(id).await?;
        let closed_as_of_date =
            if let Idempotent::Executed(date) = fiscal_year.close_next_sequential_month(now) {
                date
            } else {
                return Ok(fiscal_year);
            };

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut fiscal_year).await?;
        self.chart_of_accounts
            .close_as_of(op, sub, fiscal_year.chart_id, closed_as_of_date)
            .await?;

        Ok(fiscal_year)
    }

    #[instrument(
        name = "core_accounting.fiscal_years.list_for_chart_id",
        skip(self),
        err
    )]
    pub async fn list_for_chart_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        query: es_entity::PaginatedQueryArgs<FiscalYearsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<FiscalYear, FiscalYearsByCreatedAtCursor>,
        FiscalYearError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_LIST,
            )
            .await?;

        self.repo
            .list_for_chart_id_by_created_at(chart_id, query, es_entity::ListDirection::Descending)
            .await
    }

    #[instrument(name = "core_accounting.fiscal_year.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        fiscal_year_id: impl Into<FiscalYearId> + std::fmt::Debug,
    ) -> Result<Option<FiscalYear>, FiscalYearError> {
        let id = fiscal_year_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::fiscal_year(id),
                CoreAccountingAction::FISCAL_YEAR_READ,
            )
            .await?;

        self.repo.maybe_find_by_id(id).await
    }

    #[instrument(name = "core_accounting.fiscal_year.find_all", skip(self), err)]
    pub async fn find_all<T: From<FiscalYear>>(
        &self,
        ids: &[FiscalYearId],
    ) -> Result<std::collections::HashMap<FiscalYearId, T>, FiscalYearError> {
        self.repo.find_all(ids).await
    }
}
