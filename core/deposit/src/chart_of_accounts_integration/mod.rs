mod config;
pub mod error;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::Chart;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    CoreDepositAction, CoreDepositObject,
    ledger::{ChartOfAccountsIntegrationMeta, DepositLedger},
};

pub use config::ChartOfAccountsIntegrationConfig;
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

        if self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?
            .is_some()
        {
            return Err(ChartOfAccountsIntegrationError::ConfigAlreadyExists);
        }

        let individual_deposit_accounts_parent_account_set_id = chart.account_set_id_from_code(
            &config.chart_of_accounts_individual_deposit_accounts_parent_code,
        )?;
        let government_entity_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_accounts_government_entity_deposit_accounts_parent_code,
            )?;
        let private_company_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_private_company_deposit_accounts_parent_code,
            )?;
        let bank_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(&config.chart_of_account_bank_deposit_accounts_parent_code)?;
        let financial_institution_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_financial_institution_deposit_accounts_parent_code,
            )?;
        let non_domiciled_individual_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let frozen_individual_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            )?;
        let frozen_government_entity_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            )?;
        let frozen_private_company_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            )?;
        let frozen_bank_deposit_accounts_parent_account_set_id = chart.account_set_id_from_code(
            &config.chart_of_account_frozen_bank_deposit_accounts_parent_code,
        )?;
        let frozen_financial_institution_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            )?;
        let frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let omnibus_parent_account_set_id =
            chart.account_set_id_from_code(&config.chart_of_accounts_omnibus_parent_code)?;

        let charts_integration_meta = ChartOfAccountsIntegrationMeta {
            audit_info,
            config: config.clone(),
            omnibus_parent_account_set_id,
            individual_deposit_accounts_parent_account_set_id,
            government_entity_deposit_accounts_parent_account_set_id,
            private_company_deposit_accounts_parent_account_set_id,
            bank_deposit_accounts_parent_account_set_id,
            financial_institution_deposit_accounts_parent_account_set_id,
            non_domiciled_individual_deposit_accounts_parent_account_set_id,
            frozen_individual_deposit_accounts_parent_account_set_id,
            frozen_government_entity_deposit_accounts_parent_account_set_id,
            frozen_private_company_deposit_accounts_parent_account_set_id,
            frozen_bank_deposit_accounts_parent_account_set_id,
            frozen_financial_institution_deposit_accounts_parent_account_set_id,
            frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id,
        };

        self.ledger
            .attach_chart_of_accounts_account_sets(charts_integration_meta)
            .await?;

        Ok(config)
    }
}
