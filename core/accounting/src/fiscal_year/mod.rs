pub mod config;
mod entity;
pub mod error;
mod repo;

use chrono::NaiveDate;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use domain_config::DomainConfigs;
use es_entity::{Idempotent, PaginatedQueryArgs, clock::Clock};
use tracing_macros::record_error_severity;

use crate::{
    FiscalYearId,
    chart_of_accounts::ChartOfAccounts,
    primitives::{ChartId, CoreAccountingAction, CoreAccountingObject},
};

pub use config::FiscalYearConfig;
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
    domain_configs: DomainConfigs,
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
            domain_configs: self.domain_configs.clone(),
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
        domain_configs: &DomainConfigs,
        chart_of_accounts: &ChartOfAccounts<Perms>,
    ) -> Self {
        Self {
            repo: FiscalYearRepo::new(pool),
            authz: authz.clone(),
            domain_configs: domain_configs.clone(),
            chart_of_accounts: chart_of_accounts.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.fiscal_year.init_for_chart"
        skip(self),
    )]
    pub async fn init_for_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        opened_as_of: impl Into<NaiveDate> + std::fmt::Debug,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        let opened_as_of = opened_as_of.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_CREATE,
            )
            .await?;
        let latest_fiscal_year = self.find_latest_for_chart(chart_id).await?;
        if let Some(latest_fiscal_year) = latest_fiscal_year {
            return Ok(latest_fiscal_year);
        }

        tracing::info!(
            chart_id = %chart_id,
            "Initializing first fiscal year for chart"

        );
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
            .close_as_of_in_op(&mut op, sub, chart_id, close_ledger_as_of)
            .await?;
        op.commit().await?;

        Ok(fiscal_year)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_year.open_next", skip(self))]
    pub async fn open_next(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        fiscal_year_id: impl Into<FiscalYearId> + std::fmt::Debug + Copy,
    ) -> Result<FiscalYear, FiscalYearError> {
        let id = fiscal_year_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::fiscal_year(id),
                CoreAccountingAction::FISCAL_YEAR_CREATE,
            )
            .await?;
        let now = Clock::now();

        let fiscal_year = self.repo.find_by_id(id).await?;
        let new_fiscal_year = fiscal_year.next(now)?;
        let next_fiscal_year = self.repo.create(new_fiscal_year).await?;
        Ok(next_fiscal_year)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_year.close", skip(self))]
    pub async fn close(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        fiscal_year_id: impl Into<FiscalYearId> + std::fmt::Debug + Copy,
    ) -> Result<FiscalYear, FiscalYearError> {
        let id = fiscal_year_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::fiscal_year(id),
                CoreAccountingAction::FISCAL_YEAR_CLOSE,
            )
            .await?;
        let mut fiscal_year = self.repo.find_by_id(id).await?;
        let now = Clock::now();

        match fiscal_year.close(now)? {
            Idempotent::Executed(tx_details) => {
                let mut op = self.repo.begin_op().await?;
                self.repo.update_in_op(&mut op, &mut fiscal_year).await?;

                let config = self
                    .domain_configs
                    .get::<config::FiscalYearConfig>()
                    .await?;
                let Some(fiscal_year_conf) = config.value() else {
                    return Err(domain_config::DomainConfigError::NotConfigured.into());
                };

                self.chart_of_accounts
                    .post_closing_transaction(
                        op,
                        fiscal_year.chart_id,
                        &fiscal_year_conf,
                        tx_details,
                    )
                    .await?;

                Ok(fiscal_year)
            }
            Idempotent::AlreadyApplied => Ok(fiscal_year),
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_year.close_month", skip(self))]
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
        let now = Clock::now();

        let mut fiscal_year = self.repo.find_by_id(id).await?;
        if let Idempotent::Executed(date) = fiscal_year.close_next_sequential_month(now)? {
            let mut op = self.repo.begin_op().await?;
            self.repo.update_in_op(&mut op, &mut fiscal_year).await?;
            self.chart_of_accounts
                .close_as_of_in_op(&mut op, sub, fiscal_year.chart_id, date)
                .await?;
            op.commit().await?;
        }
        Ok(fiscal_year)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_years.list_for_chart_id", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_year.find_by_id", skip(self))]
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

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.fiscal_year.find_by_chart_id_and_year",
        skip(self)
    )]
    pub async fn find_by_chart_id_and_year(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        year: &str,
    ) -> Result<Option<FiscalYear>, FiscalYearError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_fiscal_years(),
                CoreAccountingAction::FISCAL_YEAR_READ,
            )
            .await?;

        self.repo
            .maybe_find_by_chart_id_and_year(chart_id, year)
            .await
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_year.find_all", skip(self))]
    pub async fn find_all<T: From<FiscalYear>>(
        &self,
        ids: &[FiscalYearId],
    ) -> Result<std::collections::HashMap<FiscalYearId, T>, FiscalYearError> {
        self.repo.find_all(ids).await
    }

    async fn find_latest_for_chart(
        &self,
        chart_id: ChartId,
    ) -> Result<Option<FiscalYear>, FiscalYearError> {
        let query = PaginatedQueryArgs::<FiscalYearsByCreatedAtCursor> {
            first: 1,
            after: None,
        };
        let result = self
            .repo
            .list_for_chart_id_by_created_at(chart_id, query, es_entity::ListDirection::Descending)
            .await?;

        Ok(result.entities.into_iter().next())
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.fiscal_year.configure", skip(self))]
    pub async fn configure(&self, cfg: FiscalYearConfig) -> Result<(), FiscalYearError> {
        let config = self.domain_configs.get::<FiscalYearConfig>().await?;
        if config.value().is_some() {
            return Err(FiscalYearError::FiscalYearConfigAlreadyExists);
        }

        self.domain_configs.update::<FiscalYearConfig>(cfg).await?;

        Ok(())
    }
}
