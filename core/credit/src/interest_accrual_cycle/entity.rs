use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    ledger::CreditFacilityAccountIds,
    obligation::{NewObligation, ObligationAccounts},
    primitives::*,
    terms::{InterestPeriod, TermValues},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct InterestAccrualCycleAccountIds {
    pub interest_receivable_not_yet_due_account_id: CalaAccountId,
    pub interest_receivable_due_account_id: CalaAccountId,
    pub interest_receivable_overdue_account_id: CalaAccountId,
    pub interest_defaulted_account_id: CalaAccountId,
    pub interest_income_account_id: CalaAccountId,
}

impl From<CreditFacilityAccountIds> for InterestAccrualCycleAccountIds {
    fn from(credit_facility_account_ids: CreditFacilityAccountIds) -> Self {
        Self {
            interest_receivable_not_yet_due_account_id: credit_facility_account_ids
                .interest_receivable_not_yet_due_account_id,
            interest_receivable_due_account_id: credit_facility_account_ids
                .interest_receivable_due_account_id,
            interest_receivable_overdue_account_id: credit_facility_account_ids
                .interest_receivable_overdue_account_id,
            interest_defaulted_account_id: credit_facility_account_ids
                .interest_defaulted_account_id,
            interest_income_account_id: credit_facility_account_ids.interest_income_account_id,
        }
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "InterestAccrualCycleId")]
#[allow(clippy::large_enum_variant)]
pub enum InterestAccrualCycleEvent {
    Initialized {
        id: InterestAccrualCycleId,
        facility_id: CreditFacilityId,
        idx: InterestAccrualCycleIdx,
        period: InterestPeriod,
        facility_matures_at: DateTime<Utc>,
        account_ids: InterestAccrualCycleAccountIds,
        terms: TermValues,
        audit_info: AuditInfo,
    },
    InterestAccrued {
        ledger_tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        accrued_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    InterestAccrualsPosted {
        ledger_tx_id: LedgerTxId,
        tx_ref: String,
        obligation_id: ObligationId,
        total: UsdCents,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct InterestAccrualCycle {
    pub id: InterestAccrualCycleId,
    pub credit_facility_id: CreditFacilityId,
    pub account_ids: InterestAccrualCycleAccountIds,
    pub idx: InterestAccrualCycleIdx,
    pub facility_matures_at: DateTime<Utc>,
    pub terms: TermValues,
    pub period: InterestPeriod,
    events: EntityEvents<InterestAccrualCycleEvent>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualCycleData {
    pub(crate) interest: UsdCents,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
    pub(crate) effective: chrono::NaiveDate,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualData {
    pub(crate) interest: UsdCents,
    pub(crate) period: InterestPeriod,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
}

impl TryFromEvents<InterestAccrualCycleEvent> for InterestAccrualCycle {
    fn try_from_events(
        events: EntityEvents<InterestAccrualCycleEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = InterestAccrualCycleBuilder::default();
        for event in events.iter_all() {
            match event {
                InterestAccrualCycleEvent::Initialized {
                    id,
                    facility_id,
                    account_ids,
                    idx,
                    period,
                    facility_matures_at,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*facility_id)
                        .account_ids(*account_ids)
                        .idx(*idx)
                        .period(*period)
                        .facility_matures_at(*facility_matures_at)
                        .terms(*terms)
                }
                InterestAccrualCycleEvent::InterestAccrued { .. } => (),
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl InterestAccrualCycle {
    fn accrual_cycle_ends_at(&self) -> DateTime<Utc> {
        self.terms
            .accrual_cycle_interval
            .period_from(self.period.start)
            .truncate(self.facility_matures_at)
            .expect("'period.start' should be before 'facility_matures_at'")
            .end
    }

    fn total_accrued(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                InterestAccrualCycleEvent::InterestAccrued { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn last_accrual_period(&self) -> Option<InterestPeriod> {
        let mut last_accrued_at = None;
        let mut second_to_last_accrued_at = None;
        for event in self.events.iter_all() {
            if let InterestAccrualCycleEvent::InterestAccrued { accrued_at, .. } = event {
                second_to_last_accrued_at = last_accrued_at;
                last_accrued_at = Some(*accrued_at);
            }
        }
        last_accrued_at?;

        let interval = self.terms.accrual_interval;
        match second_to_last_accrued_at {
            Some(accrued_at) => interval.period_from(accrued_at).next(),
            None => interval.period_from(self.period.start),
        }
        .truncate(self.accrual_cycle_ends_at())
    }

    pub fn count_accrued(&self) -> usize {
        self.events
            .iter_all()
            .filter(|event| matches!(event, InterestAccrualCycleEvent::InterestAccrued { .. }))
            .count()
    }

    pub(crate) fn next_accrual_period(&self) -> Option<InterestPeriod> {
        let last_accrual = self.events.iter_all().rev().find_map(|event| match event {
            InterestAccrualCycleEvent::InterestAccrued { accrued_at, .. } => Some(*accrued_at),
            _ => None,
        });

        let accrual_interval = self.terms.accrual_interval;

        let untruncated_period = match last_accrual {
            Some(last_end_date) => accrual_interval.period_from(last_end_date).next(),
            None => accrual_interval.period_from(self.period.start),
        };

        untruncated_period.truncate(self.accrual_cycle_ends_at())
    }

    pub(crate) fn record_accrual(
        &mut self,
        amount: UsdCents,
        audit_info: AuditInfo,
    ) -> InterestAccrualData {
        let accrual_period = self
            .next_accrual_period()
            .expect("Accrual period should exist inside this function");

        let days_in_interest_period = accrual_period.days();
        let interest_for_period = self
            .terms
            .annual_rate
            .interest_for_time_period(amount, days_in_interest_period);

        let accrual_tx_ref = format!("{}-interest-accrual-{}", self.id, self.count_accrued() + 1);
        let interest_accrual = InterestAccrualData {
            interest: interest_for_period,
            period: accrual_period,
            tx_ref: accrual_tx_ref,
            tx_id: LedgerTxId::new(),
        };

        self.events
            .push(InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id: interest_accrual.tx_id,
                tx_ref: interest_accrual.tx_ref.to_string(),
                amount: interest_accrual.interest,
                accrued_at: interest_accrual.period.end,
                audit_info,
            });

        interest_accrual
    }

    pub(crate) fn accrual_cycle_data(&self) -> Option<InterestAccrualCycleData> {
        let last_accrual_period = self.last_accrual_period()?;

        match last_accrual_period
            .next()
            .truncate(self.accrual_cycle_ends_at())
        {
            Some(_) => None,
            None => {
                let accrual_cycle_tx_ref = format!(
                    "{}-interest-accrual-cycle-{}",
                    self.credit_facility_id, self.idx
                );
                let interest_accrual_cycle = InterestAccrualCycleData {
                    interest: self.total_accrued(),
                    tx_ref: accrual_cycle_tx_ref,
                    tx_id: LedgerTxId::new(),
                    effective: last_accrual_period.end.date_naive(),
                };

                Some(interest_accrual_cycle)
            }
        }
    }

    pub(crate) fn record_accrual_cycle(
        &mut self,
        InterestAccrualCycleData {
            interest,
            tx_ref,
            tx_id,
            effective,
            ..
        }: InterestAccrualCycleData,
        audit_info: AuditInfo,
    ) -> Idempotent<NewObligation> {
        idempotency_guard!(
            self.events.iter_all(),
            InterestAccrualCycleEvent::InterestAccrualsPosted { .. }
        );
        let obligation_id = ObligationId::new();
        self.events
            .push(InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id: tx_id,
                tx_ref: tx_ref.to_string(),
                obligation_id,
                total: interest,
                effective,
                audit_info: audit_info.clone(),
            });

        let due_date = self.accrual_cycle_ends_at();
        let overdue_date = self
            .terms
            .obligation_overdue_duration_from_due
            .map(|d| d.end_date(due_date));
        Idempotent::Executed(
            NewObligation::builder()
                .id(obligation_id)
                .credit_facility_id(self.credit_facility_id)
                .obligation_type(ObligationType::Interest)
                .reference(tx_ref.to_string())
                .amount(interest)
                .tx_id(tx_id)
                .not_yet_due_accounts(ObligationAccounts {
                    receivable_account_id: self
                        .account_ids
                        .interest_receivable_not_yet_due_account_id,
                    account_to_be_credited_id: self.account_ids.interest_income_account_id,
                })
                .due_accounts(ObligationAccounts {
                    receivable_account_id: self.account_ids.interest_receivable_due_account_id,
                    account_to_be_credited_id: self.account_ids.interest_income_account_id,
                })
                .overdue_accounts(ObligationAccounts {
                    receivable_account_id: self.account_ids.interest_receivable_overdue_account_id,
                    account_to_be_credited_id: self.account_ids.interest_income_account_id,
                })
                .defaulted_account_id(self.account_ids.interest_defaulted_account_id)
                .due_date(due_date)
                .overdue_date(overdue_date)
                .effective(effective)
                .audit_info(audit_info)
                .build()
                .expect("could not build new interest accrual cycle obligation"),
        )
    }
}

#[derive(Debug, Builder)]
pub struct NewInterestAccrualCycle {
    #[builder(setter(into))]
    pub id: InterestAccrualCycleId,
    #[builder(setter(into))]
    pub credit_facility_id: CreditFacilityId,
    pub account_ids: InterestAccrualCycleAccountIds,
    pub idx: InterestAccrualCycleIdx,
    pub period: InterestPeriod,
    pub facility_matures_at: DateTime<Utc>,
    terms: TermValues,
    #[builder(setter(into))]
    audit_info: AuditInfo,
}

impl NewInterestAccrualCycle {
    pub fn builder() -> NewInterestAccrualCycleBuilder {
        NewInterestAccrualCycleBuilder::default()
    }

    pub fn first_accrual_cycle_period(&self) -> InterestPeriod {
        self.terms.accrual_interval.period_from(self.period.start)
    }
}

impl IntoEvents<InterestAccrualCycleEvent> for NewInterestAccrualCycle {
    fn into_events(self) -> EntityEvents<InterestAccrualCycleEvent> {
        EntityEvents::init(
            self.id,
            [InterestAccrualCycleEvent::Initialized {
                id: self.id,
                facility_id: self.credit_facility_id,
                account_ids: self.account_ids,
                idx: self.idx,
                period: self.period,
                facility_matures_at: self.facility_matures_at,
                terms: self.terms,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use audit::AuditEntryId;
    use chrono::{Datelike, TimeZone, Utc};
    use rust_decimal_macros::dec;

    use crate::terms::{FacilityDuration, InterestInterval, ObligationDuration, OneTimeFeeRatePct};

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration_from_accrual(ObligationDuration::Days(0))
            .obligation_overdue_duration_from_due(None)
            .obligation_liquidation_duration_from_due(None)
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(OneTimeFeeRatePct::ZERO)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn default_started_at() -> DateTime<Utc> {
        "2024-01-15T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
    }

    fn default_period() -> InterestPeriod {
        InterestInterval::EndOfDay.period_from(default_started_at())
    }

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn accrual_from(events: Vec<InterestAccrualCycleEvent>) -> InterestAccrualCycle {
        InterestAccrualCycle::try_from_events(EntityEvents::init(
            InterestAccrualCycleId::new(),
            events,
        ))
        .unwrap()
    }

    fn initial_events() -> Vec<InterestAccrualCycleEvent> {
        let terms = default_terms();
        let started_at = default_started_at();
        vec![InterestAccrualCycleEvent::Initialized {
            id: InterestAccrualCycleId::new(),
            facility_id: CreditFacilityId::new(),
            account_ids: CreditFacilityAccountIds::new().into(),
            idx: InterestAccrualCycleIdx::FIRST,
            period: default_period(),
            facility_matures_at: terms.duration.maturity_date(started_at),
            terms,
            audit_info: dummy_audit_info(),
        }]
    }

    #[test]
    fn last_accrual_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(accrual.last_accrual_period(), None,);
    }

    #[test]
    fn last_accrual_period_in_middle() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events.clone());
        assert_eq!(
            accrual.last_accrual_period().unwrap().start,
            accrual.period.start
        );

        let second_accrual_period = first_accrual_cycle_period.next();
        let second_accrual_at = second_accrual_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events);
        assert_eq!(
            accrual.last_accrual_period().unwrap().start,
            second_accrual_period.start
        );
    }

    #[test]
    fn next_accrual_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(
            accrual.next_accrual_period().unwrap().start,
            accrual.period.start
        );
    }

    #[test]
    fn next_accrual_period_in_middle() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.extend([InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
            audit_info: dummy_audit_info(),
        }]);
        let accrual = accrual_from(events);

        assert_eq!(
            accrual.next_accrual_period().unwrap(),
            first_accrual_cycle_period.next()
        );
    }

    #[test]
    fn next_accrual_period_at_end() {
        let mut events = initial_events();

        let facility_matures_at = default_terms().duration.maturity_date(default_started_at());
        let final_accrual_period = default_terms()
            .accrual_cycle_interval
            .period_from(default_started_at())
            .truncate(facility_matures_at)
            .unwrap();
        let final_accrual_at = final_accrual_period.end;

        events.extend([InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: final_accrual_at,
            audit_info: dummy_audit_info(),
        }]);
        let accrual = accrual_from(events);

        assert_eq!(accrual.next_accrual_period(), None);
    }

    #[test]
    fn zero_amount_accrual() {
        let mut accrual = accrual_from(initial_events());
        let InterestAccrualData {
            interest, period, ..
        } = accrual.record_accrual(UsdCents::ZERO, dummy_audit_info());
        assert_eq!(interest, UsdCents::ZERO);
        let start = default_started_at();
        assert_eq!(period.start, start);
        let start = start.date_naive();
        let end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        assert_eq!(period.end, end_of_day);

        assert!(accrual.accrual_cycle_data().is_none());
    }

    fn end_of_month(start_date: DateTime<Utc>) -> DateTime<Utc> {
        let current_year = start_date.year();
        let current_month = start_date.month();

        let (year, month) = if current_month == 12 {
            (current_year + 1, 1)
        } else {
            (current_year, current_month + 1)
        };

        Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .single()
            .expect("should return a valid date time")
            - chrono::Duration::seconds(1)
    }

    #[test]
    fn accrual_is_zero_for_zero_outstanding() {
        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let start_day = start.day();
        let end = end_of_month(start);
        let end_day = end.day();
        let mut expected_end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        let mut accrual_cycle_data: Option<InterestAccrualCycleData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(accrual_cycle_data.is_none());

            let InterestAccrualData {
                interest, period, ..
            } = accrual.record_accrual(UsdCents::ZERO, dummy_audit_info());
            assert_eq!(interest, UsdCents::ZERO);
            assert_eq!(period.end, expected_end_of_day);

            accrual_cycle_data = accrual.accrual_cycle_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = UsdCents::ZERO;
        match accrual_cycle_data {
            Some(InterestAccrualCycleData { interest, .. }) => {
                assert_eq!(interest, expected_accrual_sum);
            }
            _ => panic!("Expected accrual to be returned"),
        }
    }

    #[test]
    fn accrual_is_sum_of_all_interest() {
        let disbursed_outstanding_amount = UsdCents::from(1_000_000_00);
        let expected_daily_interest = default_terms()
            .annual_rate
            .interest_for_time_period(disbursed_outstanding_amount, 1);

        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let start_day = start.day();
        let end = end_of_month(start);
        let end_day = end.day();
        let mut expected_end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        let mut accrual_cycle_data: Option<InterestAccrualCycleData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(accrual_cycle_data.is_none());

            let InterestAccrualData {
                interest, period, ..
            } = accrual.record_accrual(disbursed_outstanding_amount, dummy_audit_info());
            assert_eq!(interest, expected_daily_interest);
            assert_eq!(period.end, expected_end_of_day);

            accrual_cycle_data = accrual.accrual_cycle_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = expected_daily_interest * (end_day + 1 - start_day).into();
        match accrual_cycle_data {
            Some(InterestAccrualCycleData { interest, .. }) => {
                assert_eq!(interest, expected_accrual_sum);
            }
            _ => panic!("Expected accrual to be returned"),
        }
    }
}
