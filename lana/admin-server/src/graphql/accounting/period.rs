use async_graphql::*;

use crate::primitives::*;

use lana_app::accounting::accounting_period::{
    AccountingPeriod as DomainAccountingPeriod,
    ChartOfAccountsIntegrationConfig as DomainChartOfAccountsIntegrationConfig,
    Period as DomainPeriod,
};

#[derive(SimpleObject, Clone)]
pub struct AccountingPeriod {
    id: ID,
    accounting_period_id: UUID,
    period: PeriodRange,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainAccountingPeriod>,
}

impl From<DomainAccountingPeriod> for AccountingPeriod {
    fn from(accounting_period: DomainAccountingPeriod) -> Self {
        Self {
            id: accounting_period.id.to_global_id(),
            accounting_period_id: UUID::from(accounting_period.id),
            period: accounting_period.period.clone().into(),

            entity: Arc::new(accounting_period),
        }
    }
}

#[derive(SimpleObject, Clone, Copy)]
pub struct PeriodRange {
    period_start: Date,
    period_end: Date,
}

impl From<DomainPeriod> for PeriodRange {
    fn from(period: DomainPeriod) -> Self {
        Self {
            period_start: period.period_start().into(),
            period_end: period.period_end().into(),
        }
    }
}

#[derive(InputObject)]
pub struct AccountingPeriodCloseInput {
    pub chart_id: UUID,
    pub description: Option<String>,
}
crate::mutation_payload! { AccountingPeriodClosePayload, accounting_period: AccountingPeriod }

#[derive(SimpleObject, Clone)]
pub struct AccountingPeriodModuleConfig {
    chart_of_accounts_id: Option<UUID>,
    revenue_code: Option<String>,
    cost_of_revenue_code: Option<String>,
    expenses_code: Option<String>,
    equity_retained_earnings_code: Option<String>,
    equity_retained_losses_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChartOfAccountsIntegrationConfig>,
}

impl From<DomainChartOfAccountsIntegrationConfig> for AccountingPeriodModuleConfig {
    fn from(values: DomainChartOfAccountsIntegrationConfig) -> Self {
        Self {
            chart_of_accounts_id: Some(values.chart_of_accounts_id.into()),
            revenue_code: Some(values.revenue_code.to_string()),
            cost_of_revenue_code: Some(values.cost_of_revenue_code.to_string()),
            expenses_code: Some(values.expenses_code.to_string()),
            equity_retained_earnings_code: Some(values.equity_retained_earnings_code.to_string()),
            equity_retained_losses_code: Some(values.equity_retained_losses_code.to_string()),

            _entity: Arc::new(values),
        }
    }
}

#[derive(InputObject)]
pub struct AccountingPeriodModuleConfigureInput {
    pub revenue_code: String,
    pub cost_of_revenue_code: String,
    pub expenses_code: String,
    pub equity_retained_earnings_code: String,
    pub equity_retained_losses_code: String,
}
crate::mutation_payload! { AccountingPeriodModuleConfigurePayload, accounting_period_config: AccountingPeriodModuleConfig }
