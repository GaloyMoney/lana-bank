use domain_configurations::{ConfigKey, DomainConfigurationKey, DomainConfigurationObject};

use crate::{
    balance_sheet::ChartOfAccountsIntegrationConfig as BalanceSheetChartConfig,
    credit::ChartOfAccountsIntegrationConfig as CreditChartConfig,
    deposit::ChartOfAccountsIntegrationConfig as DepositChartConfig,
    profit_and_loss::ChartOfAccountsIntegrationConfig as ProfitAndLossChartConfig,
};

pub struct DepositChartConfigKey;

impl ConfigKey<DepositChartConfig> for DepositChartConfigKey {
    fn key() -> DomainConfigurationKey {
        DomainConfigurationKey::new("deposit.chart_of_accounts")
    }

    fn object() -> DomainConfigurationObject {
        DomainConfigurationObject::DepositChart
    }
}

pub struct CreditChartConfigKey;

impl ConfigKey<CreditChartConfig> for CreditChartConfigKey {
    fn key() -> DomainConfigurationKey {
        DomainConfigurationKey::new("credit.chart_of_accounts")
    }

    fn object() -> DomainConfigurationObject {
        DomainConfigurationObject::CreditChart
    }
}

pub struct BalanceSheetChartConfigKey;

impl ConfigKey<BalanceSheetChartConfig> for BalanceSheetChartConfigKey {
    fn key() -> DomainConfigurationKey {
        DomainConfigurationKey::new("balance_sheet.chart_of_accounts")
    }

    fn object() -> DomainConfigurationObject {
        DomainConfigurationObject::BalanceSheetChart
    }
}

pub struct ProfitAndLossChartConfigKey;

impl ConfigKey<ProfitAndLossChartConfig> for ProfitAndLossChartConfigKey {
    fn key() -> DomainConfigurationKey {
        DomainConfigurationKey::new("profit_and_loss.chart_of_accounts")
    }

    fn object() -> DomainConfigurationObject {
        DomainConfigurationObject::ProfitAndLossChart
    }
}
