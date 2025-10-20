use chrono::{DateTime, NaiveDate, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::{AccountSetId as LedgerAccountSetId, TransactionId as LedgerTransactionId};
use es_entity::*;

use crate::primitives::{AccountingPeriodId, ChartId};

use super::error::AccountingPeriodError;
use super::period::Period;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingPeriodId")]
pub enum AccountingPeriodEvent {
    Initialized {
        id: AccountingPeriodId,
        chart_id: ChartId,
        tracking_account_set: LedgerAccountSetId,
        period: Period,
    },
    Closed {
        closed_at: DateTime<Utc>,
        closing_transaction: Option<LedgerTransactionId>,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingPeriod {
    pub id: AccountingPeriodId,
    pub chart_id: ChartId,
    #[builder(default)]
    pub closed_at: Option<DateTime<Utc>>,
    pub tracking_account_set: LedgerAccountSetId,
    pub period: Period,

    events: EntityEvents<AccountingPeriodEvent>,
}

impl AccountingPeriod {
    pub const fn is_monthly(&self) -> bool {
        self.period.is_monthly()
    }

    pub const fn is_annual(&self) -> bool {
        self.period.is_annual()
    }

    pub const fn period_end(&self) -> NaiveDate {
        self.period.period_end()
    }

    /// Closes this Accounting Period if all temporal conditions are
    /// met, otherwise returns an error describing the unfulfilled
    /// condition. Returns a blueprint for the next Accounting Period
    /// or `Idempotent::Ignored` if the period has already been
    /// closed.
    ///
    /// To close unconditionally call `close_unchecked`.
    pub fn close(
        &mut self,
        closed_at: DateTime<Utc>,
        closing_transaction: Option<LedgerTransactionId>,
    ) -> Result<Idempotent<NewAccountingPeriod>, AccountingPeriodError> {
        self.check_can_close(closed_at.date_naive())?;
        Ok(self.close_unchecked(closed_at, closing_transaction))
    }

    /// Unconditionally closes this Accounting Period. Returns a
    /// blueprint for the next Accounting Period or
    /// `Idempotent::Ignored` if the period has already been closed.
    ///
    /// This method does not verify any temporal conditions. Call
    /// `close` if temporal conditions have to be verified.
    pub fn close_unchecked(
        &mut self,
        closed_at: DateTime<Utc>,
        closing_transaction: Option<LedgerTransactionId>,
    ) -> Idempotent<NewAccountingPeriod> {
        idempotency_guard!(self.events.iter_all(), AccountingPeriodEvent::Closed { .. });

        let new_accounting_period = NewAccountingPeriod {
            id: AccountingPeriodId::new(),
            chart_id: self.chart_id,
            tracking_account_set: self.tracking_account_set,
            period: self.period.next(),
            closed_at: None,
        };

        self.events.push(AccountingPeriodEvent::Closed {
            closed_at,
            closing_transaction,
        });

        self.closed_at = Some(closed_at);

        Idempotent::Executed(new_accounting_period)
    }

    /// Verifies that `closing_date` falls into allowable time range,
    /// i. e. between the end of this period and the end of grace
    /// period. Returns error otherwise.
    fn check_can_close(&self, closing_date: NaiveDate) -> Result<(), AccountingPeriodError> {
        if self.period.is_within_grace_period(closing_date) {
            Ok(())
        } else {
            Err(AccountingPeriodError::ClosingDateOutOfGracePeriod {
                closing_date,
                grace_period_start: self.period.grace_period_start(),
                grace_period_end: self.period.grace_period_end(),
            })
        }
    }
}

impl TryFromEvents<AccountingPeriodEvent> for AccountingPeriod {
    fn try_from_events(events: EntityEvents<AccountingPeriodEvent>) -> Result<Self, EsEntityError> {
        let mut builder = AccountingPeriodBuilder::default();

        for event in events.iter_all() {
            match event {
                AccountingPeriodEvent::Initialized {
                    id,
                    chart_id,
                    tracking_account_set,
                    period,
                } => {
                    builder = builder
                        .id(*id)
                        .chart_id(*chart_id)
                        .tracking_account_set(*tracking_account_set)
                        .period(period.clone())
                }
                AccountingPeriodEvent::Closed { .. } => {}
            }
        }

        builder.events(events).build()
    }
}

pub struct NewAccountingPeriod {
    pub id: AccountingPeriodId,
    pub chart_id: ChartId,
    pub tracking_account_set: LedgerAccountSetId,
    pub period: Period,
    pub closed_at: Option<DateTime<Utc>>,
}

impl IntoEvents<AccountingPeriodEvent> for NewAccountingPeriod {
    fn into_events(self) -> EntityEvents<AccountingPeriodEvent> {
        let events = vec![AccountingPeriodEvent::Initialized {
            id: self.id,
            chart_id: self.chart_id,
            tracking_account_set: self.tracking_account_set,
            period: self.period,
        }];

        EntityEvents::init(self.id, events)
    }
}
