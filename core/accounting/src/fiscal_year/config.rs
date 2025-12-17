use domain_config::{DomainConfigError, DomainConfigKey, DomainConfigValue, DomainConfigs};
use es_entity::EsEntityError;
use serde::{Deserialize, Serialize};

use crate::{AccountCode, ChartId};

use super::error::FiscalYearError;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiscalYearClosingCoaMappingConfig {
    pub chart_of_accounts_id: ChartId,
    pub revenue_code: AccountCode,
    pub cost_of_revenue_code: AccountCode,
    pub expenses_code: AccountCode,
    pub retained_earnings_gain_code: AccountCode,
    pub retained_earnings_loss_code: AccountCode,
}

impl FiscalYearClosingCoaMappingConfig {
    fn ensure_code_is_present(code: &AccountCode, field: &str) -> Result<(), DomainConfigError> {
        if code.len_sections() == 0 {
            return Err(DomainConfigError::InvalidState(format!(
                "{field} is required"
            )));
        }

        Ok(())
    }
}

impl Default for FiscalYearClosingCoaMappingConfig {
    fn default() -> Self {
        unreachable!("Default is intentionally unsupported for FiscalYearClosingCoaMappingConfig")
    }
}

impl DomainConfigValue for FiscalYearClosingCoaMappingConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("fiscal-year-closing-coa-mapping");

    fn validate(&self) -> Result<(), DomainConfigError> {
        Self::ensure_code_is_present(&self.revenue_code, "revenue_code")?;
        Self::ensure_code_is_present(&self.cost_of_revenue_code, "cost_of_revenue_code")?;
        Self::ensure_code_is_present(&self.expenses_code, "expenses_code")?;
        Self::ensure_code_is_present(
            &self.retained_earnings_gain_code,
            "retained_earnings_gain_code",
        )?;
        Self::ensure_code_is_present(
            &self.retained_earnings_loss_code,
            "retained_earnings_loss_code",
        )?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct FiscalYearClosingConfigService {
    domain_configs: DomainConfigs,
}

impl FiscalYearClosingConfigService {
    pub fn new(domain_configs: &DomainConfigs) -> Self {
        Self {
            domain_configs: domain_configs.clone(),
        }
    }

    pub async fn load(
        &self,
        chart_id: ChartId,
    ) -> Result<FiscalYearClosingCoaMappingConfig, FiscalYearError> {
        let config = self
            .domain_configs
            .get::<FiscalYearClosingCoaMappingConfig>()
            .await
            .map_err(|err| match err {
                DomainConfigError::EsEntityError(EsEntityError::NotFound) => {
                    FiscalYearError::FiscalYearClosingMappingNotConfigured
                }
                other => other.into(),
            })?;

        if config.chart_of_accounts_id != chart_id {
            return Err(FiscalYearError::FiscalYearClosingMappingChartMismatch {
                config_chart_id: config.chart_of_accounts_id,
                fiscal_year_chart_id: chart_id,
            });
        }

        Ok(config)
    }
}
