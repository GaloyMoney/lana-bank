mod config;
pub mod error;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::Chart;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{CoreDepositAction, CoreDepositObject, ledger::DepositLedger};

pub use config::ChartOfAccountsIntegrationConfig;
pub(crate) use config::ResolvedChartOfAccountsIntegrationConfig;
use error::ChartOfAccountsIntegrationError;

pub struct ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
{
    authz: Arc<Perms>,
    ledger: Arc<DepositLedger>,
}

impl<Perms> Clone for ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms> ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDepositAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDepositObject>,
{
    pub fn new(authz: Arc<Perms>, ledger: Arc<DepositLedger>) -> Self {
        Self { authz, ledger }
    }

    #[record_error_severity]
    #[instrument(name = "deposit.chart_of_accounts_integrations.get_config", skip(self))]
    pub async fn get_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, ChartOfAccountsIntegrationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::chart_of_accounts_integration(),
                CoreDepositAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ,
            )
            .await?;
        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit.chart_of_accounts_integrations.set_config",
        skip(self, chart)
    )]
    pub async fn set_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrationError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::chart_of_accounts_integration(),
                CoreDepositAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        if chart.id != config.chart_of_accounts_id {
            return Err(ChartOfAccountsIntegrationError::ChartIdMismatch);
        }

        if chart.accounting_base_config().is_none() {
            return Err(ChartOfAccountsIntegrationError::AccountingBaseConfigNotFound);
        }

        if self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?
            .is_some()
        {
            return Err(ChartOfAccountsIntegrationError::ConfigAlreadyExists);
        }

        let resolved_integration_config =
            ResolvedChartOfAccountsIntegrationConfig::try_new(config.clone(), chart, audit_info)?;

        self.ledger
            .attach_chart_of_accounts_account_sets(resolved_integration_config)
            .await?;

        Ok(config)
    }
}
