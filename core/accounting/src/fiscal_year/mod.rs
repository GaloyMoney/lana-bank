mod entity;
pub mod error;
mod ledger;
mod repo;

use es_entity::Idempotent;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use chrono::{Datelike, NaiveDate};

use cala_ledger::CalaLedger;

use crate::{
    FiscalYearId,
    fiscal_year::ledger::FiscalYearLedger,
    primitives::{
        CalaAccountSetId, CalaJournalId, ChartId, CoreAccountingAction, CoreAccountingObject,
    },
};

pub use entity::FiscalYear;
#[cfg(feature = "json-schema")]
pub use entity::FiscalYearEvent;
pub(super) use entity::*;
use error::*;
pub(super) use repo::*;

pub struct FiscalYears<Perms>
where
    Perms: PermissionCheck,
{
    repo: FiscalYearRepo,
    authz: Perms,
    ledger: FiscalYearLedger,
    journal_id: CalaJournalId,
}

impl<Perms> Clone for FiscalYears<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            journal_id: self.journal_id,
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
        cala: &CalaLedger,
        journal_id: CalaJournalId,
    ) -> Self {
        let ledger = FiscalYearLedger::new(cala, journal_id);
        Self {
            repo: FiscalYearRepo::new(pool),
            authz: authz.clone(),
            ledger,
            journal_id,
        }
    }

    #[instrument(
        name = "core_accounting.fiscal_year.init_first_fiscal_year"
        skip(self),
        err
    )]
    pub async fn init_first_fiscal_year(
        &self,
        opened_as_of: NaiveDate,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
        tracking_account_set_id: impl Into<CalaAccountSetId> + std::fmt::Debug,
    ) -> Result<(), FiscalYearError> {
        let id = chart_id.into();
        let fiscal_years = self
            .repo
            .list_for_chart_id_by_created_at(
                id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?;
        if !fiscal_years.entities.is_empty() {
            return Ok(());
        }
        tracing::info!("Initializing first FiscalYear for chart ID: {}", id);
        let now = crate::time::now();
        let init_fiscal_year = NewFiscalYear::builder()
            .id(FiscalYearId::new())
            .chart_id(id)
            .tracking_account_set_id(tracking_account_set_id.into())
            .first_period_opened_at(now)
            .build()
            .expect("Could not build new FiscalYear");
        self.repo.create(init_fiscal_year).await?;

        Ok(())
    }

    #[instrument(name = "core_accounting.fiscal_year.close_month", skip(self), err)]
    pub async fn close_month(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_CLOSE,
            )
            .await?;
        let now = crate::time::now();
        let fiscal_years = self
            .repo
            .list_for_chart_id_by_created_at(
                chart_id.into(),
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?;
        let mut latest_year = fiscal_years
            .entities
            .first()
            .cloned()
            .ok_or(FiscalYearError::CurrentYearNotFound)?;

        if latest_year.first_period_opened_as_of.year() != now.year() {
            return Err(FiscalYearError::CurrentYearNotFound);
        }

        let closed_as_of_date =
            if let Idempotent::Executed(date) = latest_year.close_last_month(now)? {
                date
            } else {
                return Err(FiscalYearError::FiscalYearMonthAlreadyClosed);
            };

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut latest_year).await?;
        self.ledger
            .close_month_as_of(op, closed_as_of_date, latest_year.tracking_account_set_id)
            .await?;

        Ok(latest_year)
    }

    #[instrument(
        name = "core_accounting.fiscal_year.find_current_fiscal_year",
        skip(self),
        err
    )]
    pub async fn find_current_fiscal_year(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Option<FiscalYear>, FiscalYearError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_READ,
            )
            .await?;
        self.repo.find_current_by_chart_id(chart_id.into()).await
    }

    #[instrument(name = "core_accounting.fiscal_year.find_all", skip(self), err)]
    pub async fn find_all<T: From<FiscalYear>>(
        &self,
        ids: &[FiscalYearId],
    ) -> Result<std::collections::HashMap<FiscalYearId, T>, FiscalYearError> {
        self.repo.find_all(ids).await
    }
}
