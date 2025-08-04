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
    pub in_liquidation_account_id: CalaAccountId,
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
            in_liquidation_account_id: credit_facility_account_ids.in_liquidation_account_id,
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
        accrual_idx: usize,
        tx_ref: String,
        amount: UsdCents,
        accrued_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    },
    AccruedInterestReverted {
        ledger_tx_id: LedgerTxId,
        accrued_ledger_tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    },
    InterestAccrualsPosted {
        ledger_tx_id: LedgerTxId,
        tx_ref: String,
        obligation_id: Option<ObligationId>,
        total: UsdCents,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    },
    PostedInterestAccrualsReverted {
        ledger_tx_id: LedgerTxId,
        posted_ledger_tx_id: LedgerTxId,
        tx_ref: String,
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
    reverted_ledger_tx_ids: Vec<LedgerTxId>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualCycleData {
    pub(crate) interest: UsdCents,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
    pub(crate) effective: chrono::NaiveDate,
}

#[derive(Debug, Clone)]
pub(crate) struct NewInterestAccrualCycleData {
    pub(crate) id: InterestAccrualCycleId,
    pub(crate) first_accrual_end_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualData {
    pub(crate) interest: UsdCents,
    pub(crate) period: InterestPeriod,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
}

#[derive(Debug, Clone)]
struct InterestAccruedEventData {
    ledger_tx_id: LedgerTxId,
    accrual_idx: usize,
    amount: UsdCents,
    effective: chrono::NaiveDate,
}

#[derive(Debug, Clone)]
pub(crate) struct RevertedInterestAccrualData {
    pub(crate) tx_id: LedgerTxId,
    pub(crate) tx_ref: String,
    pub(crate) interest: UsdCents,
    pub(crate) effective: chrono::NaiveDate,
}

#[derive(Debug, Clone)]
struct PostedCycleEventData {
    ledger_tx_id: LedgerTxId,
    amount: UsdCents,
    effective: chrono::NaiveDate,
}

pub(crate) struct RevertedInterestCycleData {
    pub(crate) tx_id: LedgerTxId,
    pub(crate) tx_ref: String,
    pub(crate) total: UsdCents,
    pub(crate) effective: chrono::NaiveDate,
}

pub(crate) enum RevertedInterestEventData {
    Accrued(RevertedInterestAccrualData),
    PostedCycle(RevertedInterestCycleData),
}

impl TryFromEvents<InterestAccrualCycleEvent> for InterestAccrualCycle {
    fn try_from_events(
        events: EntityEvents<InterestAccrualCycleEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = InterestAccrualCycleBuilder::default();
        let mut reverted_ledger_tx_ids = vec![];
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
                InterestAccrualCycleEvent::AccruedInterestReverted {
                    accrued_ledger_tx_id,
                    ..
                } => reverted_ledger_tx_ids.push(*accrued_ledger_tx_id),
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. } => (),
                InterestAccrualCycleEvent::PostedInterestAccrualsReverted {
                    posted_ledger_tx_id,
                    ..
                } => reverted_ledger_tx_ids.push(*posted_ledger_tx_id),
            }
        }
        builder
            .reverted_ledger_tx_ids(reverted_ledger_tx_ids)
            .events(events)
            .build()
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

    pub(crate) fn is_completed(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .find(|event| match event {
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. }
                | InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. }
                | InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                _ => false,
            })
            .is_some()
    }

    fn is_reverted_event(&self, ledger_tx_id: &LedgerTxId) -> bool {
        self.reverted_ledger_tx_ids.contains(ledger_tx_id)
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

        let accrual_idx = self.count_accrued() + 1;
        let accrual_tx_ref = format!("{}-interest-accrual-{}", self.id, accrual_idx);
        let interest_accrual = InterestAccrualData {
            interest: interest_for_period,
            period: accrual_period,
            tx_ref: accrual_tx_ref,
            tx_id: LedgerTxId::new(),
        };

        self.events
            .push(InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id: interest_accrual.tx_id,
                accrual_idx,
                tx_ref: interest_accrual.tx_ref.to_string(),
                amount: interest_accrual.interest,
                accrued_at: interest_accrual.period.end,
                effective: interest_accrual.period.end.date_naive(),
                audit_info,
            });

        interest_accrual
    }

    fn last_unreverted_accrual(&self) -> Option<InterestAccruedEventData> {
        self.events.iter_all().rev().find_map(|event| match event {
            InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id,
                accrual_idx,
                amount,
                effective,
                ..
            } if !self.is_reverted_event(ledger_tx_id) => Some(InterestAccruedEventData {
                ledger_tx_id: *ledger_tx_id,
                accrual_idx: *accrual_idx,
                amount: *amount,
                effective: *effective,
            }),
            _ => None,
        })
    }

    fn has_unreverted_cycle_posting(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => Some(false),
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. } => Some(true),
                _ => None,
            })
            .unwrap_or(false)
    }

    fn revert_accrual(&mut self, audit_info: AuditInfo) -> Idempotent<RevertedInterestAccrualData> {
        if self.has_unreverted_cycle_posting() {
            return Idempotent::Ignored;
        }

        let InterestAccruedEventData {
            ledger_tx_id: accrued_ledger_tx_id,
            accrual_idx,
            amount,
            effective,
        } = match self.last_unreverted_accrual() {
            Some(a) => a,
            None => return Idempotent::Ignored,
        };

        let reverted_interest_accrual = RevertedInterestAccrualData {
            tx_id: LedgerTxId::new(),
            tx_ref: format!(
                "{}-reverted-interest-accrual-{}-{}",
                self.id, accrual_idx, accrued_ledger_tx_id
            ),
            interest: amount,
            effective,
        };

        self.events
            .push(InterestAccrualCycleEvent::AccruedInterestReverted {
                ledger_tx_id: reverted_interest_accrual.tx_id,
                accrued_ledger_tx_id,
                tx_ref: reverted_interest_accrual.tx_ref.to_string(),
                amount: reverted_interest_accrual.interest,
                effective: reverted_interest_accrual.effective,
                audit_info,
            });
        self.reverted_ledger_tx_ids.push(accrued_ledger_tx_id);

        Idempotent::Executed(reverted_interest_accrual)
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
    ) -> Idempotent<Option<NewObligation>> {
        idempotency_guard!(
            self.events.iter_all(),
            InterestAccrualCycleEvent::InterestAccrualsPosted { .. }
        );

        if interest.is_zero() {
            self.events
                .push(InterestAccrualCycleEvent::InterestAccrualsPosted {
                    ledger_tx_id: tx_id,
                    tx_ref: tx_ref.to_string(),
                    obligation_id: None,
                    total: interest,
                    effective,
                    audit_info: audit_info.clone(),
                });

            return Idempotent::Executed(None);
        }

        let due_date = self.accrual_cycle_ends_at();
        let overdue_date = self
            .terms
            .obligation_overdue_duration_from_due
            .map(|d| d.end_date(due_date));
        let liquidation_date = self
            .terms
            .obligation_liquidation_duration_from_due
            .map(|d| d.end_date(due_date));
        let new_obligation = NewObligation::builder()
            .id(ObligationId::new())
            .credit_facility_id(self.credit_facility_id)
            .obligation_type(ObligationType::Interest)
            .reference(tx_ref.to_string())
            .amount(interest)
            .tx_id(tx_id)
            .not_yet_due_accounts(ObligationAccounts {
                receivable_account_id: self.account_ids.interest_receivable_not_yet_due_account_id,
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
            .in_liquidation_account_id(self.account_ids.in_liquidation_account_id)
            .defaulted_account_id(self.account_ids.interest_defaulted_account_id)
            .due_date(due_date)
            .overdue_date(overdue_date)
            .liquidation_date(liquidation_date)
            .effective(effective)
            .audit_info(audit_info.clone())
            .build()
            .expect("could not build new interest accrual cycle obligation");

        self.events
            .push(InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id: tx_id,
                tx_ref: tx_ref.to_string(),
                obligation_id: Some(new_obligation.id),
                total: interest,
                effective,
                audit_info,
            });

        Idempotent::Executed(Some(new_obligation))
    }

    fn last_unreverted_accrual_cycle(&self) -> Option<PostedCycleEventData> {
        self.events.iter_all().rev().find_map(|event| match event {
            InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id,
                total: amount,
                effective,
                ..
            } if !self.is_reverted_event(ledger_tx_id) => Some(PostedCycleEventData {
                ledger_tx_id: *ledger_tx_id,
                amount: *amount,
                effective: *effective,
            }),
            _ => None,
        })
    }

    fn revert_accrual_cycle(
        &mut self,
        audit_info: AuditInfo,
    ) -> Idempotent<RevertedInterestCycleData> {
        let PostedCycleEventData {
            ledger_tx_id: posted_ledger_tx_id,
            amount,
            effective,
        } = match self.last_unreverted_accrual_cycle() {
            Some(a) => a,
            None => return Idempotent::Ignored,
        };

        let reverted_interest_accrual_cycle = RevertedInterestCycleData {
            tx_id: LedgerTxId::new(),
            tx_ref: format!(
                "{}-reverted-interest-accrual-cycle-{}-{}",
                self.id, self.idx, posted_ledger_tx_id
            ),
            total: amount,
            effective,
        };

        self.events
            .push(InterestAccrualCycleEvent::PostedInterestAccrualsReverted {
                ledger_tx_id: reverted_interest_accrual_cycle.tx_id,
                posted_ledger_tx_id,
                tx_ref: reverted_interest_accrual_cycle.tx_ref.to_string(),
                total: reverted_interest_accrual_cycle.total,
                effective: reverted_interest_accrual_cycle.effective,
                audit_info,
            });
        self.reverted_ledger_tx_ids.push(posted_ledger_tx_id);

        Idempotent::Executed(reverted_interest_accrual_cycle)
    }

    pub(crate) fn revert_on_or_after(
        &mut self,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    ) -> Idempotent<Vec<RevertedInterestEventData>> {
        let mut all_reverted_data = vec![];

        let posted_entry_after_effective_exists = self
            .last_unreverted_accrual_cycle()
            .filter(|cycle| cycle.effective >= effective)
            .is_some();
        if posted_entry_after_effective_exists {
            if let Idempotent::Executed(reverted_cycle_data) =
                self.revert_accrual_cycle(audit_info.clone())
            {
                all_reverted_data.push(RevertedInterestEventData::PostedCycle(reverted_cycle_data))
            }
        }

        while let Some(event) = self.last_unreverted_accrual() {
            if event.effective < effective {
                break;
            }

            match self.revert_accrual(audit_info.clone()) {
                Idempotent::Executed(accrued_data) => {
                    all_reverted_data.push(RevertedInterestEventData::Accrued(accrued_data))
                }
                _ => break,
            }
        }

        if all_reverted_data.is_empty() {
            Idempotent::Ignored
        } else {
            Idempotent::Executed(all_reverted_data)
        }
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
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
            effective: first_accrual_at.date_naive(),
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
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
            effective: second_accrual_at.date_naive(),
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events);
        assert_eq!(
            accrual.last_accrual_period().unwrap().start,
            second_accrual_period.start
        );
    }

    #[test]
    fn count_accrued_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(accrual.count_accrued(), 0);
    }

    #[test]
    fn count_multiple_accrued() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
            effective: first_accrual_at.date_naive(),
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events.clone());
        assert_eq!(accrual.count_accrued(), 1);

        let second_accrual_period = first_accrual_cycle_period.next();
        let second_accrual_at = second_accrual_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            accrual_idx: 1,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
            effective: second_accrual_at.date_naive(),
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events);
        assert_eq!(accrual.count_accrued(), 2);
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
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
            effective: first_accrual_at.date_naive(),
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
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: final_accrual_at,
            effective: final_accrual_at.date_naive(),
            audit_info: dummy_audit_info(),
        }]);
        let accrual = accrual_from(events);

        assert_eq!(accrual.next_accrual_period(), None);
    }

    #[test]
    fn total_accrued() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
            effective: first_accrual_at.date_naive(),
            audit_info: dummy_audit_info(),
        });

        let second_accrual_period = first_accrual_cycle_period.next();
        let second_accrual_at = second_accrual_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            accrual_idx: 1,
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
            effective: second_accrual_at.date_naive(),
            audit_info: dummy_audit_info(),
        });

        let accrual = accrual_from(events);
        assert_eq!(accrual.total_accrued(), UsdCents::from(2));
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
    fn record_accrual_returns_correct_period() {
        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let end = end_of_month(start);
        let start_day = start.day();
        let end_day = end.day();

        let mut expected_end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        for _ in start_day..(end_day + 1) {
            let InterestAccrualData { period, .. } =
                accrual.record_accrual(UsdCents::ONE, dummy_audit_info());
            assert_eq!(period.end, expected_end_of_day);

            expected_end_of_day += chrono::Duration::days(1);
        }
    }

    #[test]
    fn can_revert_accrual() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;

        let ledger_tx_id = LedgerTxId::new();
        let amount = UsdCents::ONE;
        let effective = first_accrual_at.date_naive();
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id,
            accrual_idx: 0,
            tx_ref: "".to_string(),
            amount,
            accrued_at: first_accrual_at,
            effective,
            audit_info: dummy_audit_info(),
        });
        let mut accrual = accrual_from(events);

        assert!(accrual.revert_accrual(dummy_audit_info()).did_execute());

        let (tx_id, interest, reverted_effective) = match accrual.events.iter_all().last() {
            Some(InterestAccrualCycleEvent::AccruedInterestReverted {
                accrued_ledger_tx_id,
                amount,
                effective,
                ..
            }) => (*accrued_ledger_tx_id, *amount, *effective),
            _ => panic!("Expected last event to be AccruedInterestReverted"),
        };
        assert_eq!(ledger_tx_id, tx_id);
        assert_eq!(amount, interest);
        assert_eq!(effective, reverted_effective);
    }

    #[test]
    fn revert_accrual_ignored_if_last_accrual_reverted() {
        let mut events = initial_events();

        let accrued_ledger_tx_id = LedgerTxId::new();
        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.extend([
            InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id: accrued_ledger_tx_id,
                accrual_idx: 0,
                tx_ref: "".to_string(),
                amount: UsdCents::ONE,
                accrued_at: first_accrual_at,
                effective: first_accrual_at.date_naive(),
                audit_info: dummy_audit_info(),
            },
            InterestAccrualCycleEvent::AccruedInterestReverted {
                ledger_tx_id: LedgerTxId::new(),
                accrued_ledger_tx_id,
                tx_ref: "".to_string(),
                amount: UsdCents::ONE,
                effective: first_accrual_at.date_naive(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let mut accrual = accrual_from(events);

        assert!(accrual.revert_accrual(dummy_audit_info()).was_ignored());
    }

    #[test]
    fn can_revert_accrual_if_last_posted_event_not_reverted() {
        let mut events = initial_events();

        let posted_ledger_tx_id = LedgerTxId::new();
        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.extend([
            InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id: LedgerTxId::new(),
                accrual_idx: 0,
                tx_ref: "".to_string(),
                amount: UsdCents::ONE,
                accrued_at: first_accrual_at,
                effective: first_accrual_at.date_naive(),
                audit_info: dummy_audit_info(),
            },
            InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id: posted_ledger_tx_id,
                tx_ref: "".to_string(),
                obligation_id: Some(ObligationId::new()),
                total: UsdCents::ONE,
                effective: Utc::now().date_naive(),
                audit_info: dummy_audit_info(),
            },
            InterestAccrualCycleEvent::PostedInterestAccrualsReverted {
                ledger_tx_id: LedgerTxId::new(),
                posted_ledger_tx_id,
                tx_ref: "".to_string(),
                total: UsdCents::ONE,
                effective: Utc::now().date_naive(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let mut accrual = accrual_from(events);

        assert!(accrual.revert_accrual(dummy_audit_info()).did_execute());
    }

    #[test]
    fn revert_accrual_ignored_if_last_posted_event_is_reverted() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.extend([
            InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id: LedgerTxId::new(),
                accrual_idx: 0,
                tx_ref: "".to_string(),
                amount: UsdCents::ONE,
                accrued_at: first_accrual_at,
                effective: first_accrual_at.date_naive(),
                audit_info: dummy_audit_info(),
            },
            InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                obligation_id: Some(ObligationId::new()),
                total: UsdCents::ONE,
                effective: Utc::now().date_naive(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let mut accrual = accrual_from(events);

        assert!(accrual.revert_accrual(dummy_audit_info()).was_ignored());
    }

    #[test]
    fn can_revert_accrual_cycle() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;

        let ledger_tx_id = LedgerTxId::new();
        let amount = UsdCents::ONE;
        let effective = first_accrual_at.date_naive();
        events.push(InterestAccrualCycleEvent::InterestAccrualsPosted {
            ledger_tx_id,
            tx_ref: "".to_string(),
            obligation_id: Some(ObligationId::new()),
            total: amount,
            effective,
            audit_info: dummy_audit_info(),
        });
        let mut accrual = accrual_from(events);

        assert!(
            accrual
                .revert_accrual_cycle(dummy_audit_info())
                .did_execute()
        );

        let (tx_id, total, reverted_effective) = match accrual.events.iter_all().last() {
            Some(InterestAccrualCycleEvent::PostedInterestAccrualsReverted {
                posted_ledger_tx_id,
                total,
                effective,
                ..
            }) => (*posted_ledger_tx_id, *total, *effective),
            _ => panic!("Expected last event to be AccruedInterestReverted"),
        };
        assert_eq!(ledger_tx_id, tx_id);
        assert_eq!(amount, total);
        assert_eq!(effective, reverted_effective);
    }

    #[test]
    fn revert_accrual_cycle_ignored_if_last_posted_is_reverted() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;

        let ledger_tx_id = LedgerTxId::new();
        let amount = UsdCents::ONE;
        let effective = first_accrual_at.date_naive();
        events.extend([
            InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id,
                tx_ref: "".to_string(),
                obligation_id: Some(ObligationId::new()),
                total: amount,
                effective,
                audit_info: dummy_audit_info(),
            },
            InterestAccrualCycleEvent::PostedInterestAccrualsReverted {
                ledger_tx_id: LedgerTxId::new(),
                posted_ledger_tx_id: ledger_tx_id,
                tx_ref: "".to_string(),
                total: amount,
                effective,
                audit_info: dummy_audit_info(),
            },
        ]);
        let mut accrual = accrual_from(events);

        assert!(
            accrual
                .revert_accrual_cycle(dummy_audit_info())
                .was_ignored()
        );
    }

    mod revert_on_or_after {
        use super::*;

        mod with_posted {
            use super::*;

            fn events_and_effective_dates()
            -> (Vec<InterestAccrualCycleEvent>, DateTime<Utc>, DateTime<Utc>) {
                let mut events = initial_events();

                let first_accrual_period = default_terms()
                    .accrual_interval
                    .period_from(default_started_at());
                let first_accrual_at = first_accrual_period.end;
                let second_accrual_period = first_accrual_period.next();
                let second_accrual_at = second_accrual_period.end;

                events.extend([
                    InterestAccrualCycleEvent::InterestAccrued {
                        ledger_tx_id: LedgerTxId::new(),
                        accrual_idx: 0,
                        tx_ref: "".to_string(),
                        amount: UsdCents::ONE,
                        accrued_at: first_accrual_at,
                        effective: first_accrual_at.date_naive(),
                        audit_info: dummy_audit_info(),
                    },
                    InterestAccrualCycleEvent::InterestAccrued {
                        ledger_tx_id: LedgerTxId::new(),
                        accrual_idx: 0,
                        tx_ref: "".to_string(),
                        amount: UsdCents::ONE,
                        accrued_at: second_accrual_at,
                        effective: second_accrual_at.date_naive(),
                        audit_info: dummy_audit_info(),
                    },
                    InterestAccrualCycleEvent::InterestAccrualsPosted {
                        ledger_tx_id: LedgerTxId::new(),
                        tx_ref: "".to_string(),
                        obligation_id: Some(ObligationId::new()),
                        total: UsdCents::from(3),
                        effective: second_accrual_at.date_naive(),
                        audit_info: dummy_audit_info(),
                    },
                ]);

                (events, first_accrual_at, second_accrual_at)
            }

            #[test]
            fn can_revert_on_or_after() {
                let (events, first_accrual_at, _) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_before_first_accrual =
                    first_accrual_at.date_naive() - chrono::Duration::days(1);
                let res = accrual
                    .revert_on_or_after(date_before_first_accrual, dummy_audit_info())
                    .unwrap();

                let (n_posted, n_accrued) =
                    res.iter()
                        .fold((0, 0), |(posted, accrued), event| match event {
                            RevertedInterestEventData::PostedCycle(_) => (posted + 1, accrued),
                            RevertedInterestEventData::Accrued(_) => (posted, accrued + 1),
                        });
                assert_eq!(n_posted, 1);
                assert_eq!(n_accrued, 2);
            }

            #[test]
            fn before_all_accruals() {
                let (events, first_accrual_at, _) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_before_first_accrual =
                    first_accrual_at.date_naive() - chrono::Duration::days(1);
                accrual
                    .revert_on_or_after(date_before_first_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 1);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 2);
            }

            #[test]
            fn on_first_accrual() {
                let (events, first_accrual_at, _) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_on_first_accrual = first_accrual_at.date_naive();
                accrual
                    .revert_on_or_after(date_on_first_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 1);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 2);
            }

            #[test]
            fn on_second_accrual() {
                let (events, _, second_accrual_at) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_on_second_accrual = second_accrual_at.date_naive();
                accrual
                    .revert_on_or_after(date_on_second_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 1);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 1);
            }

            #[test]
            fn after_all_accruals() {
                let (events, _, second_accrual_at) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_after_second_accrual =
                    second_accrual_at.date_naive() + chrono::Duration::days(1);
                accrual
                    .revert_on_or_after(date_after_second_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 0);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 0);
            }
        }

        mod no_posted {

            use super::*;

            fn events_and_effective_dates()
            -> (Vec<InterestAccrualCycleEvent>, DateTime<Utc>, DateTime<Utc>) {
                let mut events = initial_events();

                let first_accrual_period = default_terms()
                    .accrual_interval
                    .period_from(default_started_at());
                let first_accrual_at = first_accrual_period.end;
                let second_accrual_period = first_accrual_period.next();
                let second_accrual_at = second_accrual_period.end;

                events.extend([
                    InterestAccrualCycleEvent::InterestAccrued {
                        ledger_tx_id: LedgerTxId::new(),
                        accrual_idx: 0,
                        tx_ref: "".to_string(),
                        amount: UsdCents::ONE,
                        accrued_at: first_accrual_at,
                        effective: first_accrual_at.date_naive(),
                        audit_info: dummy_audit_info(),
                    },
                    InterestAccrualCycleEvent::InterestAccrued {
                        ledger_tx_id: LedgerTxId::new(),
                        accrual_idx: 0,
                        tx_ref: "".to_string(),
                        amount: UsdCents::ONE,
                        accrued_at: second_accrual_at,
                        effective: second_accrual_at.date_naive(),
                        audit_info: dummy_audit_info(),
                    },
                ]);

                (events, first_accrual_at, second_accrual_at)
            }

            #[test]
            fn can_revert_on_or_after() {
                let (events, first_accrual_at, _) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_before_first_accrual =
                    first_accrual_at.date_naive() - chrono::Duration::days(1);
                let res = accrual
                    .revert_on_or_after(date_before_first_accrual, dummy_audit_info())
                    .unwrap();

                let (n_posted, n_accrued) =
                    res.iter()
                        .fold((0, 0), |(posted, accrued), event| match event {
                            RevertedInterestEventData::PostedCycle(_) => (posted + 1, accrued),
                            RevertedInterestEventData::Accrued(_) => (posted, accrued + 1),
                        });
                assert_eq!(n_posted, 0);
                assert_eq!(n_accrued, 2);
            }

            #[test]
            fn before_all_accruals() {
                let (events, first_accrual_at, _) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_before_first_accrual =
                    first_accrual_at.date_naive() - chrono::Duration::days(1);
                accrual
                    .revert_on_or_after(date_before_first_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 0);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 2);
            }

            #[test]
            fn on_first_accrual() {
                let (events, first_accrual_at, _) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_on_first_accrual = first_accrual_at.date_naive();
                accrual
                    .revert_on_or_after(date_on_first_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 0);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 2);
            }

            #[test]
            fn on_second_accrual() {
                let (events, _, second_accrual_at) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_on_second_accrual = second_accrual_at.date_naive();
                accrual
                    .revert_on_or_after(date_on_second_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 0);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 1);
            }

            #[test]
            fn after_all_accruals() {
                let (events, _, second_accrual_at) = events_and_effective_dates();
                let mut accrual = accrual_from(events);

                let date_after_second_accrual =
                    second_accrual_at.date_naive() + chrono::Duration::days(1);
                accrual
                    .revert_on_or_after(date_after_second_accrual, dummy_audit_info())
                    .did_execute();

                let n_posted_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::PostedInterestAccrualsReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_posted_reverted, 0);

                let n_accrued_reverted = accrual
                    .events()
                    .iter_all()
                    .filter(|event| match event {
                        InterestAccrualCycleEvent::AccruedInterestReverted { .. } => true,
                        _ => false,
                    })
                    .count();
                assert_eq!(n_accrued_reverted, 0);
            }
        }
    }

    #[test]
    fn accrual_cycle_data_exists_at_end_of_cycle() {
        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let end = end_of_month(start);
        let start_day = start.day();
        let end_day = end.day();

        let mut accrual_cycle_data: Option<InterestAccrualCycleData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(accrual_cycle_data.is_none());

            accrual.record_accrual(UsdCents::ONE, dummy_audit_info());

            accrual_cycle_data = accrual.accrual_cycle_data();
        }
        assert!(accrual_cycle_data.is_some());
    }

    #[test]
    fn accrual_is_sum_of_all_interest() {
        let disbursed_outstanding_amount = UsdCents::from(1_000_000_00);
        let expected_daily_interest = default_terms()
            .annual_rate
            .interest_for_time_period(disbursed_outstanding_amount, 1);

        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let end = end_of_month(start);
        let start_day = start.day();
        let end_day = end.day();

        for _ in start_day..(end_day + 1) {
            let InterestAccrualData { interest, .. } =
                accrual.record_accrual(disbursed_outstanding_amount, dummy_audit_info());
            assert_eq!(interest, expected_daily_interest);
        }

        let expected_accrual_sum = expected_daily_interest * (end_day + 1 - start_day).into();
        let InterestAccrualCycleData { interest, .. } = accrual.accrual_cycle_data().unwrap();
        assert_eq!(interest, expected_accrual_sum);
    }
}
