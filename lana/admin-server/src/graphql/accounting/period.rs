use async_graphql::*;

use crate::primitives::*;

use lana_app::accounting::accounting_period::{
    AccountingPeriod as DomainAccountingPeriod, Period as DomainPeriod,
};

#[derive(SimpleObject, Clone)]
pub struct AccountingPeriod {
    id: ID,
    accounting_period_id: UUID,
    tracking_account_set_id: UUID,
    period: PeriodRange,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainAccountingPeriod>,
}

impl From<DomainAccountingPeriod> for AccountingPeriod {
    fn from(accounting_period: DomainAccountingPeriod) -> Self {
        Self {
            id: accounting_period.id.to_global_id(),
            accounting_period_id: UUID::from(accounting_period.id),
            tracking_account_set_id: UUID::from(accounting_period.tracking_account_set),
            period: accounting_period.period.into(),

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
            period_start: period.period_start.into(),
            period_end: period.period_end.into(),
        }
    }
}

#[derive(InputObject)]
pub struct AccountingPeriodCloseInput {
    pub chart_id: UUID,
    pub description: Option<String>,
}
crate::mutation_payload! { AccountingPeriodClosePayload, accounting_period: AccountingPeriod }
