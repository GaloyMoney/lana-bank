mod config;
pub mod error;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting_contracts::ChartLookup;
use domain_config::InternalDomainConfigs;

use crate::{CoreCreditAction, CoreCreditObject, ledger::*};

pub use config::ChartOfAccountsIntegrationConfig;
pub(crate) use config::ResolvedChartOfAccountsIntegrationConfig;
use error::ChartOfAccountsIntegrationError;

pub struct ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
{
    authz: Arc<Perms>,
    domain_configs: Arc<InternalDomainConfigs>,
    ledger: Arc<CreditLedger>,
}

impl<Perms> Clone for ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            domain_configs: self.domain_configs.clone(),
        }
    }
}

impl<Perms> ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(
        authz: Arc<Perms>,
        ledger: Arc<CreditLedger>,
        domain_configs: Arc<InternalDomainConfigs>,
    ) -> Self {
        Self {
            authz,
            ledger,
            domain_configs,
        }
    }

    pub async fn set_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &dyn ChartLookup,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::chart_of_accounts_integration(),
                CoreCreditAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        if chart.id() != config.chart_of_accounts_id {
            return Err(ChartOfAccountsIntegrationError::ChartIdMismatch);
        }

        let existing_module_config = self
            .domain_configs
            .get::<ResolvedChartOfAccountsIntegrationConfig>()
            .await?;

        if let Some(existing) = existing_module_config
            .maybe_value()
            .filter(|existing| existing.config == config)
        {
            return Ok(existing.config);
        }

        if !chart.has_accounting_base_config() {
            return Err(ChartOfAccountsIntegrationError::AccountingBaseConfigNotFound);
        }

        let mut op = self.domain_configs.begin_op().await?;

        let resolved_integration_config =
            ResolvedChartOfAccountsIntegrationConfig::try_new(config, chart)?;

        self.domain_configs
            .update_in_op::<ResolvedChartOfAccountsIntegrationConfig>(
                &mut op,
                resolved_integration_config.clone(),
            )
            .await?;

        let mut op = op.with_db_time().await?;
        self.ledger
            .attach_chart_of_accounts_account_sets_in_op(
                &mut op,
                &resolved_integration_config,
                existing_module_config.maybe_value().as_ref(),
            )
            .await?;
        op.commit().await?;

        Ok(resolved_integration_config.config)
    }

    pub async fn get_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, ChartOfAccountsIntegrationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::chart_of_accounts_integration(),
                CoreCreditAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ,
            )
            .await?;

        let config = self
            .domain_configs
            .get::<ResolvedChartOfAccountsIntegrationConfig>()
            .await?;
        Ok(config.maybe_value().map(|meta| meta.config))
    }
}
