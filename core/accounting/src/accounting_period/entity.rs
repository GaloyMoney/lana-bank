use chrono::{DateTime, Days, Duration, NaiveDate, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use cala_ledger::TransactionId as LedgerTransactionId;

use crate::{AccountingPeriodId, primitives::ChartId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Year {
    Calendar,
    Fiscal { first: NaiveDate },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Month {
    Calendar,
    OnDay(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frequency {
    Year(Year),
    Month(Month),
}

impl Frequency {
    pub const fn is_monthly(&self) -> bool {
        matches!(self, Frequency::Month(..))
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingPeriodId")]
pub enum AccountingPeriodEvent {
    Initialized {
        id: AccountingPeriodId,
        chart_id: ChartId,
        frequency: Frequency,
        period_start: NaiveDate,
        period_end: NaiveDate,
        grace_period: Duration,
    },
    Closed {
        closing_transaction: Option<LedgerTransactionId>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingPeriod {
    pub(super) id: AccountingPeriodId,
    chart_id: ChartId,
    pub(super) frequency: Frequency,
    period_start: NaiveDate,
    period_end: NaiveDate,
    grace_period: Duration,

    events: EntityEvents<AccountingPeriodEvent>,
}

impl AccountingPeriod {
    /// Unconditionally closes this Accounting Period. Returns a
    /// blueprint for the next Accounting Period or
    /// `Idempotent::Ignored` if the period has already been closed.
    ///
    /// This method does not verify any temporal conditions. Call
    /// `checked_close` if temporal conditions have to be verified.
    pub fn close(
        &mut self,
        closing_transaction: Option<LedgerTransactionId>,
    ) -> Idempotent<NewAccountingPeriod> {
        idempotency_guard!(self.events.iter_all(), AccountingPeriodEvent::Closed { .. });

        let new_period_start = self.period_end.checked_add_days(Days::new(1)).unwrap();
        let new_period_end = self.frequency.period_end(&new_period_start);

        let new_period = NewAccountingPeriod {
            id: AccountingPeriodId::new(),
            chart_id: self.chart_id,
            frequency: self.frequency.clone(),
            period_start: new_period_start,
            period_end: new_period_end,
            grace_period: self.grace_period.clone(),
        };

        self.events.push(AccountingPeriodEvent::Closed {
            closing_transaction,
        });

        Idempotent::Executed(new_period)
    }

    /// Closes this Accounting Period if all temporal conditions are
    /// met, otherwise returns an error describing the unfulfilled
    /// condition. Returns a blueprint for the next Accounting Period
    /// or `Idempotent::Ignored` if the period has already been
    /// closed.
    ///
    /// To close unconditionally call `close`.
    pub fn checked_close(
        &mut self,
        closing_transaction: Option<LedgerTransactionId>,
        date: NaiveDate,
    ) -> Result<Idempotent<NewAccountingPeriod>, String> {
        todo!("verify conditions");

        Ok(self.close(closing_transaction))
    }
}

impl TryFromEvents<AccountingPeriodEvent> for AccountingPeriod {
    fn try_from_events(events: EntityEvents<AccountingPeriodEvent>) -> Result<Self, EsEntityError> {
        todo!()
    }
}

pub struct NewAccountingPeriod {
    pub id: AccountingPeriodId,
    pub chart_id: ChartId,
    pub frequency: Frequency,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub grace_period: Duration,
}

impl IntoEvents<AccountingPeriodEvent> for NewAccountingPeriod {
    fn into_events(self) -> EntityEvents<AccountingPeriodEvent> {
        todo!()
    }
}
