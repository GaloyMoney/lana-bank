mod config;
pub mod error;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting_primitives::{AccountCategory, AccountCode, CalaAccountSetId, ChartId};
use domain_config::InternalDomainConfigs;
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
    domain_configs: Arc<InternalDomainConfigs>,
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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDepositAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDepositObject>,
{
    pub fn new(
        authz: Arc<Perms>,
        ledger: Arc<DepositLedger>,
        domain_configs: Arc<InternalDomainConfigs>,
    ) -> Self {
        Self {
            authz,
            ledger,
            domain_configs,
        }
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
        let config = self
            .domain_configs
            .get::<ResolvedChartOfAccountsIntegrationConfig>()
            .await?;
        Ok(config.maybe_value().map(|meta| meta.config))
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit.chart_of_accounts_integrations.set_config",
        skip(self, account_set_lookup)
    )]
    pub async fn set_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        has_accounting_base_config: bool,
        account_set_lookup: &impl Fn(&AccountCode, AccountCategory) -> Option<CalaAccountSetId>,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::chart_of_accounts_integration(),
                CoreDepositAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        if chart_id != config.chart_of_accounts_id {
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

        if !has_accounting_base_config {
            return Err(ChartOfAccountsIntegrationError::AccountingBaseConfigNotFound);
        }

        let resolved_integration_config =
            ResolvedChartOfAccountsIntegrationConfig::try_new(config, account_set_lookup)?;

        let mut op = self.domain_configs.begin_op().await?;
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
}
