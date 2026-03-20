#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub(crate) mod chart_of_accounts_integration;
pub mod error;
pub(crate) mod ledger;
pub(crate) mod position;
mod primitives;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{CalaLedger, Currency};
use domain_config::InternalDomainConfigs;
use rust_decimal::Decimal;
use tracing_macros::record_error_severity;

use chart_of_accounts_integration::ChartOfAccountsIntegrations;
pub use chart_of_accounts_integration::{
    ChartOfAccountsIntegrationConfig, error::ChartOfAccountsIntegrationError,
};
use ledger::FxLedger;
use position::FxPositions;
pub use primitives::*;

pub struct CoreFx<Perms>
where
    Perms: PermissionCheck,
{
    ledger: Arc<FxLedger>,
    positions: Arc<FxPositions>,
    chart_of_accounts_integrations: Arc<ChartOfAccountsIntegrations<Perms>>,
}

impl<Perms> Clone for CoreFx<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            ledger: self.ledger.clone(),
            positions: self.positions.clone(),
            chart_of_accounts_integrations: self.chart_of_accounts_integrations.clone(),
        }
    }
}

impl<Perms> CoreFx<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreFxAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreFxObject>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "fx.init", skip_all, fields(journal_id = %journal_id))]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
        jobs: &job::Jobs,
        internal_domain_configs: &InternalDomainConfigs,
    ) -> Result<Self, error::CoreFxError> {
        let clock = jobs.clock().clone();
        let authz_arc = Arc::new(authz.clone());

        let ledger = Arc::new(FxLedger::init(cala, journal_id, clock.clone()).await?);
        let positions = Arc::new(FxPositions::new(pool, clock));

        let chart_of_accounts_integrations = Arc::new(ChartOfAccountsIntegrations::new(
            authz_arc,
            Arc::new(internal_domain_configs.clone()),
        ));

        Ok(Self {
            ledger,
            positions,
            chart_of_accounts_integrations,
        })
    }

    pub fn chart_of_accounts_integrations(&self) -> &ChartOfAccountsIntegrations<Perms> {
        &self.chart_of_accounts_integrations
    }

    pub fn ledger(&self) -> &FxLedger {
        &self.ledger
    }

    #[record_error_severity]
    #[tracing::instrument(name = "fx.convert_fiat_fx", skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn convert_fiat_fx(
        &self,
        op: &mut es_entity::DbOp<'_>,
        source_currency: Currency,
        target_currency: Currency,
        source_amount: Decimal,
        rate: ExchangeRate,
        source_account_id: CalaAccountId,
        target_account_id: CalaAccountId,
        trading_account_id: CalaAccountId,
        gain_account_id: CalaAccountId,
        loss_account_id: CalaAccountId,
        rounding_account_id: CalaAccountId,
        functional_currency: Currency,
        initiated_by: &impl audit::SystemSubject,
    ) -> Result<FxConversionResult, error::CoreFxError> {
        let result = self
            .ledger
            .convert_fiat_fx_with_rate_in_op(
                op,
                source_currency,
                target_currency,
                source_amount,
                rate,
                source_account_id,
                target_account_id,
                trading_account_id,
                gain_account_id,
                loss_account_id,
                rounding_account_id,
                functional_currency,
                &self.positions,
                initiated_by,
            )
            .await?;
        Ok(result)
    }
}
