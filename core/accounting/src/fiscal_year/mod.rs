mod entity;
pub mod error;
mod ledger;
mod repo;

use es_entity::Idempotent;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{CalaLedger, LedgerOperation};

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
        let ledger = FiscalYearLedger::new(cala);
        Self {
            repo: FiscalYearRepo::new(pool),
            authz: authz.clone(),
            ledger,
            journal_id,
        }
    }

    #[instrument(
        name = "core_accounting.fiscal_year.add_closing_control"
        skip(self, op),
        err
    )]
    pub async fn add_closing_control_in_op(
        &self,
        op: LedgerOperation<'_>,
        tracking_account_set_id: impl Into<CalaAccountSetId> + std::fmt::Debug,
    ) -> Result<(), FiscalYearError> {
        self.ledger
            .attach_closing_controls_to_account_set_in_op(op, tracking_account_set_id)
            .await?;

        Ok(())
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
        tracking_account_set_id: impl Into<CalaAccountSetId> + std::fmt::Debug,
    ) -> Result<FiscalYear, FiscalYearError> {
        let opened_as_of = opened_as_of.into();
        let tracking_account_set_id = tracking_account_set_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_OPEN_FIRST,
            )
            .await?;

        if let Ok(fiscal_year) = self.find_current_by_chart_id(chart_id).await {
            return Ok(fiscal_year);
        }

        tracing::info!("Initializing first FiscalYear for chart ID: {}", chart_id);
        let new_fiscal_year = NewFiscalYear::builder()
            .id(FiscalYearId::new())
            .chart_id(chart_id)
            .tracking_account_set_id(tracking_account_set_id)
            .opened_as_of(opened_as_of)
            .build()
            .expect("Could not build new FiscalYear");

        let mut op = self.repo.begin_op().await?;
        let fiscal_year = self.repo.create_in_op(&mut op, new_fiscal_year).await?;

        self.ledger
            .close_month_as_of(
                op,
                opened_as_of
                    .pred_opt()
                    .expect("Date was first possible NaiveDate type value"),
                fiscal_year.tracking_account_set_id,
            )
            .await?;

        Ok(fiscal_year)
    }

    #[instrument(name = "core_accounting.fiscal_year.close_month", skip(self), err)]
    pub async fn close_month(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        let now = crate::time::now();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_CLOSE,
            )
            .await?;

        let mut latest_year = self.find_current_by_chart_id(chart_id).await?;
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
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_READ,
            )
            .await?;
        self.find_current_by_chart_id(chart_id).await
    }

    async fn find_current_by_chart_id(
        &self,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.repo
            .list_for_chart_id_by_created_at(
                chart_id,
                Default::default(),
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
