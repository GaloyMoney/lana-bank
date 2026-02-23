use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::{payment_allocation::NewPaymentAllocation, primitives::*};

pub(crate) struct ObligationDueReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub not_yet_due_account_id: CalaAccountId,
    pub due_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

pub(crate) struct ObligationOverdueReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub due_account_id: CalaAccountId,
    pub overdue_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

pub(crate) struct ObligationDefaultedReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub receivable_account_id: CalaAccountId,
    pub defaulted_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

pub(crate) enum ObligationTransition {
    Due(ObligationDueReallocationData),
    Overdue(ObligationOverdueReallocationData),
    Defaulted(ObligationDefaultedReallocationData),
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ObligationId")]
pub enum ObligationEvent {
    Initialized {
        id: ObligationId,
        beneficiary_id: BeneficiaryId,
        obligation_type: ObligationType,
        amount: UsdCents,
        reference: String,
        ledger_tx_id: LedgerTxId,
        receivable_account_ids: ObligationReceivableAccountIds,
        defaulted_account_id: CalaAccountId,
        due_date: EffectiveDate,
        overdue_date: Option<EffectiveDate>,
        defaulted_date: Option<EffectiveDate>,
        liquidation_date: Option<EffectiveDate>,
        effective: chrono::NaiveDate,
    },
    DueRecorded {
        ledger_tx_id: LedgerTxId,
        due_amount: UsdCents,
    },
    OverdueRecorded {
        ledger_tx_id: LedgerTxId,
        overdue_amount: UsdCents,
    },
    DefaultedRecorded {
        ledger_tx_id: LedgerTxId,
        defaulted_amount: UsdCents,
    },
    PaymentAllocated {
        ledger_tx_id: LedgerTxId,
        payment_id: PaymentId,
        payment_allocation_id: PaymentAllocationId,
        payment_allocation_amount: UsdCents,
    },
    Completed {
        effective: chrono::NaiveDate,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Obligation {
    pub id: ObligationId,
    pub tx_id: LedgerTxId,
    pub beneficiary_id: BeneficiaryId,
    pub reference: String,
    pub initial_amount: UsdCents,
    pub obligation_type: ObligationType,
    pub effective: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub overdue_date: Option<chrono::NaiveDate>,
    pub defaulted_date: Option<chrono::NaiveDate>,
    events: EntityEvents<ObligationEvent>,
}

impl Obligation {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn lifecycle_dates(&self) -> ObligationLifecycleDates {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    due_date,
                    overdue_date,
                    liquidation_date,
                    defaulted_date,
                    ..
                } => Some(ObligationLifecycleDates {
                    due: *due_date,
                    overdue: *overdue_date,
                    liquidation: *liquidation_date,
                    defaulted: *defaulted_date,
                }),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    fn lifecycle_timestamps(&self) -> ObligationLifecycleTimestamps {
        self.lifecycle_dates().into()
    }

    pub fn due_at(&self) -> DateTime<Utc> {
        self.lifecycle_timestamps().due
    }

    pub fn overdue_at(&self) -> Option<DateTime<Utc>> {
        self.lifecycle_timestamps().overdue
    }

    pub fn liquidation_at(&self) -> Option<DateTime<Utc>> {
        self.lifecycle_timestamps().liquidation
    }

    pub fn defaulted_at(&self) -> Option<DateTime<Utc>> {
        self.lifecycle_timestamps().defaulted
    }

    pub fn receivable_accounts(&self) -> &ObligationReceivableAccountIds {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    receivable_account_ids,
                    ..
                } => Some(receivable_account_ids),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    pub fn defaulted_account(&self) -> CalaAccountId {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    defaulted_account_id,
                    ..
                } => Some(*defaulted_account_id),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    pub fn receivable_account_id(&self) -> Option<CalaAccountId> {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    receivable_account_ids,
                    ..
                } => Some(receivable_account_ids.id_for_status(self.status())),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    fn expected_status(&self, now: DateTime<Utc>) -> ObligationStatus {
        let status = self.status();
        if status == ObligationStatus::Paid {
            return status;
        }

        let timestamps = self.lifecycle_timestamps();
        if timestamps.defaulted.is_some_and(|d| now >= d) {
            ObligationStatus::Defaulted
        } else if timestamps.overdue.is_some_and(|d| now >= d) {
            ObligationStatus::Overdue
        } else if now >= timestamps.due {
            ObligationStatus::Due
        } else {
            ObligationStatus::NotYetDue
        }
    }

    pub fn status(&self) -> ObligationStatus {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ObligationEvent::DueRecorded { .. } => Some(ObligationStatus::Due),
                ObligationEvent::OverdueRecorded { .. } => Some(ObligationStatus::Overdue),
                ObligationEvent::DefaultedRecorded { .. } => Some(ObligationStatus::Defaulted),
                ObligationEvent::Completed { .. } => Some(ObligationStatus::Paid),
                _ => None,
            })
            .unwrap_or(ObligationStatus::NotYetDue)
    }

    pub fn is_status_up_to_date(&self, now: DateTime<Utc>) -> bool {
        self.status() == self.expected_status(now)
    }

    pub fn outstanding(&self) -> UsdCents {
        self.events
            .iter_all()
            .fold(UsdCents::from(0), |mut total_sum, event| {
                match event {
                    ObligationEvent::Initialized { amount, .. } => {
                        total_sum += *amount;
                    }
                    ObligationEvent::PaymentAllocated {
                        payment_allocation_amount: amount,
                        ..
                    } => {
                        total_sum -= *amount;
                    }
                    _ => (),
                }
                total_sum
            })
    }

    pub fn has_outstanding_balance(&self) -> bool {
        !self.outstanding().is_zero()
    }

    pub fn next_transition_date(&self) -> Option<chrono::NaiveDate> {
        match self.status() {
            ObligationStatus::NotYetDue => Some(self.due_date),
            ObligationStatus::Due => self.overdue_date.or(self.defaulted_date),
            ObligationStatus::Overdue => self.defaulted_date,
            ObligationStatus::Defaulted | ObligationStatus::Paid => None,
        }
    }

    pub(crate) fn transition(
        &mut self,
        day: chrono::NaiveDate,
    ) -> Idempotent<ObligationTransition> {
        match self.status() {
            ObligationStatus::NotYetDue if self.due_date <= day => {
                Idempotent::Executed(ObligationTransition::Due(self.record_due()))
            }
            ObligationStatus::Due if self.overdue_date.is_some_and(|d| d <= day) => {
                Idempotent::Executed(ObligationTransition::Overdue(self.record_overdue()))
            }
            ObligationStatus::Due | ObligationStatus::Overdue
                if self.defaulted_date.is_some_and(|d| d <= day) =>
            {
                Idempotent::Executed(ObligationTransition::Defaulted(self.record_defaulted()))
            }
            _ => Idempotent::AlreadyApplied,
        }
    }

    fn record_due(&mut self) -> ObligationDueReallocationData {
        let res = ObligationDueReallocationData {
            tx_id: LedgerTxId::new(),
            amount: self.outstanding(),
            not_yet_due_account_id: self.receivable_accounts().not_yet_due,
            due_account_id: self.receivable_accounts().due,
            effective: self.due_date,
        };
        self.events.push(ObligationEvent::DueRecorded {
            ledger_tx_id: res.tx_id,
            due_amount: res.amount,
        });
        res
    }

    fn record_overdue(&mut self) -> ObligationOverdueReallocationData {
        let res = ObligationOverdueReallocationData {
            tx_id: LedgerTxId::new(),
            amount: self.outstanding(),
            due_account_id: self.receivable_accounts().due,
            overdue_account_id: self.receivable_accounts().overdue,
            effective: self.overdue_date.expect("overdue_date must be set"),
        };
        self.events.push(ObligationEvent::OverdueRecorded {
            ledger_tx_id: res.tx_id,
            overdue_amount: res.amount,
        });
        res
    }

    fn record_defaulted(&mut self) -> ObligationDefaultedReallocationData {
        let res = ObligationDefaultedReallocationData {
            tx_id: LedgerTxId::new(),
            amount: self.outstanding(),
            receivable_account_id: self.receivable_account_id().expect("Obligation is Paid"),
            defaulted_account_id: self.defaulted_account(),
            effective: self.defaulted_date.expect("defaulted_date must be set"),
        };
        self.events.push(ObligationEvent::DefaultedRecorded {
            ledger_tx_id: res.tx_id,
            defaulted_amount: res.amount,
        });
        res
    }

    pub(crate) fn allocate_payment(
        &mut self,
        amount: UsdCents,
        PaymentDetailsForAllocation {
            payment_id,
            facility_payment_holding_account_id: payment_holding_account_id,
            effective,
            ..
        }: PaymentDetailsForAllocation,
    ) -> Idempotent<NewPaymentAllocation> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ObligationEvent::PaymentAllocated {payment_id: id, .. }  if *id == payment_id
        );
        let pre_payment_outstanding = self.outstanding();
        if pre_payment_outstanding.is_zero() {
            return Idempotent::AlreadyApplied;
        }

        let payment_amount = std::cmp::min(pre_payment_outstanding, amount);
        let allocation_id = PaymentAllocationId::new();
        self.events.push(ObligationEvent::PaymentAllocated {
            ledger_tx_id: allocation_id.into(),
            payment_id,
            payment_allocation_id: allocation_id,
            payment_allocation_amount: payment_amount,
        });

        let payment_allocation_idx = self
            .events()
            .iter_all()
            .filter(|e| matches!(e, ObligationEvent::PaymentAllocated { .. }))
            .count();
        let allocation = NewPaymentAllocation::builder()
            .id(allocation_id)
            .payment_id(payment_id)
            .beneficiary_id(self.beneficiary_id)
            .obligation_id(self.id)
            .payment_allocation_idx(payment_allocation_idx)
            .obligation_type(self.obligation_type)
            .receivable_account_id(
                self.receivable_account_id()
                    .expect("Obligation was already paid"),
            )
            .payment_holding_account_id(payment_holding_account_id)
            .effective(effective)
            .amount(payment_amount)
            .build()
            .expect("could not build new payment allocation");

        if self.outstanding().is_zero() {
            self.events.push(ObligationEvent::Completed { effective });
        }

        Idempotent::Executed(allocation)
    }
}

impl TryFromEvents<ObligationEvent> for Obligation {
    fn try_from_events(events: EntityEvents<ObligationEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ObligationBuilder::default();
        for event in events.iter_all() {
            match event {
                ObligationEvent::Initialized {
                    id,
                    ledger_tx_id: tx_id,
                    beneficiary_id,
                    reference,
                    amount,
                    obligation_type,
                    effective,
                    due_date,
                    overdue_date,
                    defaulted_date,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .tx_id(*tx_id)
                        .beneficiary_id(*beneficiary_id)
                        .reference(reference.clone())
                        .initial_amount(*amount)
                        .obligation_type(*obligation_type)
                        .effective(*effective)
                        .due_date(chrono::NaiveDate::from(*due_date))
                        .overdue_date(overdue_date.map(chrono::NaiveDate::from))
                        .defaulted_date(defaulted_date.map(chrono::NaiveDate::from))
                }
                ObligationEvent::DueRecorded { .. } => (),
                ObligationEvent::OverdueRecorded { .. } => (),
                ObligationEvent::DefaultedRecorded { .. } => (),
                ObligationEvent::PaymentAllocated { .. } => (),
                ObligationEvent::Completed { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct NewObligation {
    #[builder(setter(into))]
    pub(crate) id: ObligationId,
    #[builder(setter(into))]
    pub(crate) tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(crate) beneficiary_id: BeneficiaryId,
    pub(crate) obligation_type: ObligationType,
    #[builder(setter(into))]
    pub(crate) amount: UsdCents,
    #[builder(setter(strip_option), default)]
    reference: Option<String>,
    pub(crate) receivable_account_ids: ObligationReceivableAccountIds,
    #[builder(setter(into))]
    pub(crate) defaulted_account_id: CalaAccountId,
    pub(crate) due_date: EffectiveDate,
    pub(crate) overdue_date: Option<EffectiveDate>,
    pub(crate) liquidation_date: Option<EffectiveDate>,
    #[builder(setter(strip_option), default)]
    pub(crate) defaulted_date: Option<EffectiveDate>,
    pub(crate) effective: chrono::NaiveDate,
}

impl NewObligationBuilder {
    fn validate(&self) -> Result<(), String> {
        match self.amount {
            Some(amount) if amount.is_zero() => Err("Obligation amount cannot be zero".to_string()),
            _ => Ok(()),
        }
    }
}

impl NewObligation {
    pub fn builder() -> NewObligationBuilder {
        NewObligationBuilder::default()
    }

    pub fn id(&self) -> ObligationId {
        self.id
    }

    pub fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }

    pub fn due_date_naive(&self) -> chrono::NaiveDate {
        chrono::NaiveDate::from(self.due_date)
    }

    pub fn overdue_date_naive(&self) -> Option<chrono::NaiveDate> {
        self.overdue_date.map(chrono::NaiveDate::from)
    }

    pub fn defaulted_date_naive(&self) -> Option<chrono::NaiveDate> {
        self.defaulted_date.map(chrono::NaiveDate::from)
    }

    pub fn next_transition_date(&self) -> Option<chrono::NaiveDate> {
        Some(self.due_date_naive())
    }
}

impl IntoEvents<ObligationEvent> for NewObligation {
    fn into_events(self) -> EntityEvents<ObligationEvent> {
        EntityEvents::init(
            self.id,
            [ObligationEvent::Initialized {
                id: self.id,
                beneficiary_id: self.beneficiary_id,
                obligation_type: self.obligation_type,
                reference: self.reference(),
                amount: self.amount,
                ledger_tx_id: self.tx_id,
                receivable_account_ids: self.receivable_account_ids,
                defaulted_account_id: self.defaulted_account_id,
                due_date: self.due_date,
                overdue_date: self.overdue_date,
                defaulted_date: self.defaulted_date,
                liquidation_date: self.liquidation_date,
                effective: self.effective,
            }],
        )
    }
}

impl Ord for Obligation {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.obligation_type, &other.obligation_type) {
            (ObligationType::Interest, ObligationType::Disbursal) => Ordering::Less,
            (ObligationType::Disbursal, ObligationType::Interest) => Ordering::Greater,
            _ => self
                .effective
                .cmp(&other.effective)
                .then_with(|| self.created_at().cmp(&other.created_at())),
        }
    }
}
impl PartialOrd for Obligation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for Obligation {}
impl PartialEq for Obligation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct ObligationLifecycleDates {
    pub due: EffectiveDate,
    pub overdue: Option<EffectiveDate>,
    pub liquidation: Option<EffectiveDate>,
    pub defaulted: Option<EffectiveDate>,
}

struct ObligationLifecycleTimestamps {
    due: DateTime<Utc>,
    overdue: Option<DateTime<Utc>>,
    liquidation: Option<DateTime<Utc>>,
    defaulted: Option<DateTime<Utc>>,
}

impl From<ObligationLifecycleDates> for ObligationLifecycleTimestamps {
    fn from(value: ObligationLifecycleDates) -> Self {
        ObligationLifecycleTimestamps {
            due: value.due.start_of_day(),
            overdue: value.overdue.map(|d| d.start_of_day()),
            liquidation: value.liquidation.map(|d| d.start_of_day()),
            defaulted: value.defaulted.map(|d| d.start_of_day()),
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use super::*;

    fn obligation_from(events: Vec<ObligationEvent>) -> Obligation {
        Obligation::try_from_events(EntityEvents::init(ObligationId::new(), events)).unwrap()
    }

    fn init_event(
        due: NaiveDate,
        overdue: Option<NaiveDate>,
        defaulted: Option<NaiveDate>,
    ) -> Vec<ObligationEvent> {
        vec![ObligationEvent::Initialized {
            id: ObligationId::new(),
            beneficiary_id: BeneficiaryId::new(),
            obligation_type: ObligationType::Disbursal,
            amount: UsdCents::from(10),
            reference: "ref-01".to_string(),
            ledger_tx_id: LedgerTxId::new(),
            receivable_account_ids: ObligationReceivableAccountIds::new(),
            defaulted_account_id: CalaAccountId::new(),
            due_date: due.into(),
            overdue_date: overdue.map(EffectiveDate::from),
            defaulted_date: defaulted.map(EffectiveDate::from),
            liquidation_date: None,
            effective: due,
        }]
    }

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn dummy_payment_details() -> PaymentDetailsForAllocation {
        PaymentDetailsForAllocation {
            payment_id: PaymentId::new(),
            amount: UsdCents::ONE,
            beneficiary_id: BeneficiaryId::new(),
            facility_payment_holding_account_id: CalaAccountId::new(),
            effective: Utc::now().date_naive(),
        }
    }

    #[test]
    fn builder_errors_for_zero_amount() {
        let res = NewObligation::builder().amount(UsdCents::ZERO).build();
        assert!(matches!(
            res,
            Err(NewObligationBuilderError::ValidationError(_))
        ));
    }

    // -- transition: NotYetDue -----------------------------------------------

    #[test]
    fn transition_not_yet_due_before_due_date_is_noop() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        assert!(matches!(
            obligation.transition(day(2025, 1, 9)),
            Idempotent::AlreadyApplied
        ));
        assert_eq!(obligation.status(), ObligationStatus::NotYetDue);
    }

    #[test]
    fn transition_not_yet_due_on_due_date_becomes_due() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        match obligation.transition(day(2025, 1, 10)).unwrap() {
            ObligationTransition::Due(data) => {
                assert_eq!(data.amount, obligation.initial_amount);
                assert_eq!(data.effective, day(2025, 1, 10));
            }
            _ => panic!("expected Due transition"),
        }
        assert_eq!(obligation.status(), ObligationStatus::Due);
    }

    #[test]
    fn transition_not_yet_due_after_due_date_becomes_due() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        match obligation.transition(day(2025, 2, 1)).unwrap() {
            ObligationTransition::Due(data) => {
                assert_eq!(data.amount, obligation.initial_amount);
                assert_eq!(data.effective, day(2025, 1, 10));
            }
            _ => panic!("expected Due transition"),
        }
        assert_eq!(obligation.status(), ObligationStatus::Due);
    }

    // -- transition: Due → Overdue -------------------------------------------

    #[test]
    fn transition_due_before_overdue_date_is_noop() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        obligation.transition(day(2025, 1, 10)).unwrap(); // NotYetDue → Due
        assert!(matches!(
            obligation.transition(day(2025, 1, 19)),
            Idempotent::AlreadyApplied
        ));
        assert_eq!(obligation.status(), ObligationStatus::Due);
    }

    #[test]
    fn transition_due_on_overdue_date_becomes_overdue() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        obligation.transition(day(2025, 1, 10)).unwrap();
        match obligation.transition(day(2025, 1, 20)).unwrap() {
            ObligationTransition::Overdue(data) => {
                assert_eq!(data.effective, day(2025, 1, 20));
            }
            _ => panic!("expected Overdue transition"),
        }
        assert_eq!(obligation.status(), ObligationStatus::Overdue);
    }

    #[test]
    fn transition_due_without_overdue_date_is_noop() {
        let mut obligation = obligation_from(init_event(day(2025, 1, 10), None, None));
        obligation.transition(day(2025, 1, 10)).unwrap(); // NotYetDue → Due
        assert!(matches!(
            obligation.transition(day(2025, 12, 31)),
            Idempotent::AlreadyApplied
        ));
        assert_eq!(obligation.status(), ObligationStatus::Due);
    }

    // -- transition: Due → Defaulted (skipping overdue) ----------------------

    #[test]
    fn transition_due_without_overdue_date_defaults_on_defaulted_date() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), None, Some(day(2025, 1, 30))));
        obligation.transition(day(2025, 1, 10)).unwrap(); // NotYetDue → Due
        match obligation.transition(day(2025, 1, 30)).unwrap() {
            ObligationTransition::Defaulted(data) => {
                assert_eq!(data.effective, day(2025, 1, 30));
            }
            _ => panic!("expected Defaulted transition"),
        }
        assert_eq!(obligation.status(), ObligationStatus::Defaulted);
    }

    // -- transition: Overdue → Defaulted -------------------------------------

    #[test]
    fn transition_overdue_before_defaulted_date_is_noop() {
        let mut obligation = obligation_from(init_event(
            day(2025, 1, 10),
            Some(day(2025, 1, 20)),
            Some(day(2025, 1, 30)),
        ));
        obligation.transition(day(2025, 1, 10)).unwrap(); // NotYetDue → Due
        obligation.transition(day(2025, 1, 20)).unwrap(); // Due → Overdue
        assert!(matches!(
            obligation.transition(day(2025, 1, 29)),
            Idempotent::AlreadyApplied
        ));
        assert_eq!(obligation.status(), ObligationStatus::Overdue);
    }

    #[test]
    fn transition_overdue_on_defaulted_date_becomes_defaulted() {
        let mut obligation = obligation_from(init_event(
            day(2025, 1, 10),
            Some(day(2025, 1, 20)),
            Some(day(2025, 1, 30)),
        ));
        obligation.transition(day(2025, 1, 10)).unwrap();
        obligation.transition(day(2025, 1, 20)).unwrap();
        match obligation.transition(day(2025, 1, 30)).unwrap() {
            ObligationTransition::Defaulted(data) => {
                assert_eq!(data.effective, day(2025, 1, 30));
            }
            _ => panic!("expected Defaulted transition"),
        }
        assert_eq!(obligation.status(), ObligationStatus::Defaulted);
    }

    #[test]
    fn transition_overdue_without_defaulted_date_is_noop() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        obligation.transition(day(2025, 1, 10)).unwrap();
        obligation.transition(day(2025, 1, 20)).unwrap();
        assert!(matches!(
            obligation.transition(day(2025, 12, 31)),
            Idempotent::AlreadyApplied
        ));
        assert_eq!(obligation.status(), ObligationStatus::Overdue);
    }

    // -- transition: terminal states -----------------------------------------

    #[test]
    fn transition_defaulted_is_always_noop() {
        let mut obligation = obligation_from(init_event(
            day(2025, 1, 10),
            Some(day(2025, 1, 20)),
            Some(day(2025, 1, 30)),
        ));
        obligation.transition(day(2025, 1, 10)).unwrap();
        obligation.transition(day(2025, 1, 20)).unwrap();
        obligation.transition(day(2025, 1, 30)).unwrap();
        assert_eq!(obligation.status(), ObligationStatus::Defaulted);
        assert!(matches!(
            obligation.transition(day(2025, 12, 31)),
            Idempotent::AlreadyApplied
        ));
    }

    #[test]
    fn transition_paid_is_always_noop() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        obligation
            .allocate_payment(obligation.outstanding(), dummy_payment_details())
            .unwrap();
        assert_eq!(obligation.status(), ObligationStatus::Paid);
        assert!(matches!(
            obligation.transition(day(2025, 12, 31)),
            Idempotent::AlreadyApplied
        ));
    }

    // -- transition: idempotency ---------------------------------------------

    #[test]
    fn transition_is_idempotent() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        obligation.transition(day(2025, 1, 10)).unwrap();
        assert!(matches!(
            obligation.transition(day(2025, 1, 10)),
            Idempotent::AlreadyApplied
        ));
        assert_eq!(obligation.status(), ObligationStatus::Due);
    }

    // -- completes_on_final_payment_allocation --------------------------------

    #[test]
    fn completes_on_final_payment_allocation() {
        let mut obligation =
            obligation_from(init_event(day(2025, 1, 10), Some(day(2025, 1, 20)), None));
        obligation
            .allocate_payment(UsdCents::ONE, dummy_payment_details())
            .unwrap();
        assert_eq!(obligation.status(), ObligationStatus::NotYetDue);

        obligation
            .allocate_payment(obligation.outstanding(), dummy_payment_details())
            .unwrap();
        assert_eq!(obligation.status(), ObligationStatus::Paid);
    }
}
