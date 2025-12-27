use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use es_entity::*;

use crate::{CreditFacilityId, payment_allocation::NewPaymentAllocation, primitives::*};

use super::{error::ObligationError, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ObligationId")]
pub enum ObligationEvent {
    Initialized {
        id: ObligationId,
        credit_facility_id: CreditFacilityId,
        obligation_type: ObligationType,
        amount: UsdCents,
        reference: String,
        ledger_tx_id: LedgerTxId,
        receivable_account_ids: ObligationReceivableAccountIds,
        not_yet_due_accounts: ObligationAccounts,
        due_accounts: ObligationAccounts,
        overdue_accounts: ObligationAccounts,
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
    pub credit_facility_id: CreditFacilityId,
    pub reference: String,
    pub initial_amount: UsdCents,
    pub obligation_type: ObligationType,
    pub effective: chrono::NaiveDate,
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

    pub fn not_yet_due_accounts(&self) -> ObligationAccounts {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    not_yet_due_accounts,
                    ..
                } => Some(*not_yet_due_accounts),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    pub fn due_accounts(&self) -> ObligationAccounts {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized { due_accounts, .. } => Some(*due_accounts),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    pub fn overdue_accounts(&self) -> ObligationAccounts {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    overdue_accounts, ..
                } => Some(*overdue_accounts),
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
        let (not_yet_due_accounts, due_accounts, overdue_accounts) = self
            .events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized {
                    not_yet_due_accounts,
                    due_accounts,
                    overdue_accounts,
                    ..
                } => Some((*not_yet_due_accounts, *due_accounts, *overdue_accounts)),
                _ => None,
            })
            .expect("Entity was not Initialized");

        match self.status() {
            ObligationStatus::NotYetDue => Some(not_yet_due_accounts.receivable_account_id),
            ObligationStatus::Due => Some(due_accounts.receivable_account_id),
            ObligationStatus::Overdue | ObligationStatus::Defaulted => {
                Some(overdue_accounts.receivable_account_id)
            }

            ObligationStatus::Paid => None,
        }
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

    pub(crate) fn record_due(
        &mut self,
        effective: chrono::NaiveDate,
    ) -> Idempotent<ObligationDueReallocationData> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ObligationEvent::DueRecorded { .. }
        );

        match self.status() {
            ObligationStatus::NotYetDue => (),
            _ => return Idempotent::AlreadyApplied,
        }

        let res = ObligationDueReallocationData {
            tx_id: LedgerTxId::new(),
            amount: self.outstanding(),
            not_yet_due_account_id: self.not_yet_due_accounts().receivable_account_id,
            due_account_id: self.due_accounts().receivable_account_id,
            effective,
        };

        self.events.push(ObligationEvent::DueRecorded {
            ledger_tx_id: res.tx_id,
            due_amount: res.amount,
        });

        Idempotent::Executed(res)
    }

    pub(crate) fn record_overdue(
        &mut self,
        effective: chrono::NaiveDate,
    ) -> Result<Idempotent<ObligationOverdueReallocationData>, ObligationError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ObligationEvent::OverdueRecorded { .. }
        );

        match self.status() {
            ObligationStatus::NotYetDue => {
                return Err(ObligationError::InvalidStatusTransitionToOverdue);
            }
            ObligationStatus::Due => (),
            _ => return Ok(Idempotent::AlreadyApplied),
        }

        let res = ObligationOverdueReallocationData {
            tx_id: LedgerTxId::new(),
            amount: self.outstanding(),
            due_account_id: self.due_accounts().receivable_account_id,
            overdue_account_id: self.overdue_accounts().receivable_account_id,
            effective,
        };

        self.events.push(ObligationEvent::OverdueRecorded {
            ledger_tx_id: res.tx_id,
            overdue_amount: res.amount,
        });

        Ok(Idempotent::Executed(res))
    }

    pub(crate) fn record_defaulted(
        &mut self,
        effective: chrono::NaiveDate,
    ) -> Result<Idempotent<ObligationDefaultedReallocationData>, ObligationError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ObligationEvent::DefaultedRecorded { .. }
        );

        match self.status() {
            ObligationStatus::NotYetDue => {
                return Err(ObligationError::InvalidStatusTransitionToDefaulted);
            }
            ObligationStatus::Due | ObligationStatus::Overdue => (),
            _ => return Ok(Idempotent::AlreadyApplied),
        }

        let res = ObligationDefaultedReallocationData {
            tx_id: LedgerTxId::new(),
            amount: self.outstanding(),
            receivable_account_id: self.receivable_account_id().expect("Obligation is Paid"),
            defaulted_account_id: self.defaulted_account(),
            effective,
        };

        self.events.push(ObligationEvent::DefaultedRecorded {
            ledger_tx_id: res.tx_id,
            defaulted_amount: res.amount,
        });

        Ok(Idempotent::Executed(res))
    }

    pub(crate) fn allocate_payment(
        &mut self,
        amount: UsdCents,
        payment_id: PaymentId,
        payment_source_account_id: CalaAccountId,
        effective: chrono::NaiveDate,
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
            .credit_facility_id(self.credit_facility_id)
            .obligation_id(self.id)
            .payment_allocation_idx(payment_allocation_idx)
            .obligation_type(self.obligation_type)
            .receivable_account_id(
                self.receivable_account_id()
                    .expect("Obligation was already paid"),
            )
            .payment_source_account_id(payment_source_account_id)
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
                    credit_facility_id,
                    reference,
                    amount,
                    obligation_type,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .tx_id(*tx_id)
                        .credit_facility_id(*credit_facility_id)
                        .reference(reference.clone())
                        .initial_amount(*amount)
                        .obligation_type(*obligation_type)
                        .effective(*effective)
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
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) obligation_type: ObligationType,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    #[builder(setter(strip_option), default)]
    reference: Option<String>,
    receivable_account_ids: ObligationReceivableAccountIds,
    not_yet_due_accounts: ObligationAccounts,
    due_accounts: ObligationAccounts,
    overdue_accounts: ObligationAccounts,
    #[builder(setter(into))]
    defaulted_account_id: CalaAccountId,
    due_date: EffectiveDate,
    overdue_date: Option<EffectiveDate>,
    liquidation_date: Option<EffectiveDate>,
    #[builder(setter(strip_option), default)]
    defaulted_date: Option<EffectiveDate>,
    effective: chrono::NaiveDate,
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

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl IntoEvents<ObligationEvent> for NewObligation {
    fn into_events(self) -> EntityEvents<ObligationEvent> {
        EntityEvents::init(
            self.id,
            [ObligationEvent::Initialized {
                id: self.id,
                credit_facility_id: self.credit_facility_id,
                obligation_type: self.obligation_type,
                reference: self.reference(),
                amount: self.amount,
                ledger_tx_id: self.tx_id,
                receivable_account_ids: self.receivable_account_ids,
                not_yet_due_accounts: self.not_yet_due_accounts,
                due_accounts: self.due_accounts,
                overdue_accounts: self.overdue_accounts,
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

    use super::*;

    fn obligation_from(events: Vec<ObligationEvent>) -> Obligation {
        Obligation::try_from_events(EntityEvents::init(ObligationId::new(), events)).unwrap()
    }

    fn initial_events() -> Vec<ObligationEvent> {
        vec![ObligationEvent::Initialized {
            id: ObligationId::new(),
            credit_facility_id: CreditFacilityId::new(),
            obligation_type: ObligationType::Disbursal,
            amount: UsdCents::from(10),
            reference: "ref-01".to_string(),
            ledger_tx_id: LedgerTxId::new(),
            receivable_account_ids: ObligationReceivableAccountIds {
                not_yet_due: CalaAccountId::new(),
                due: CalaAccountId::new(),
                overdue: CalaAccountId::new(),
            },
            not_yet_due_accounts: ObligationAccounts {
                receivable_account_id: CalaAccountId::new(),
            },
            due_accounts: ObligationAccounts {
                receivable_account_id: CalaAccountId::new(),
            },
            overdue_accounts: ObligationAccounts {
                receivable_account_id: CalaAccountId::new(),
            },
            defaulted_account_id: CalaAccountId::new(),
            due_date: Utc::now().into(),
            overdue_date: Some(Utc::now().into()),
            defaulted_date: None,
            liquidation_date: None,
            effective: Utc::now().date_naive(),
        }]
    }

    #[test]
    fn builder_errors_for_zero_amount() {
        let res = NewObligation::builder().amount(UsdCents::ZERO).build();
        assert!(matches!(
            res,
            Err(NewObligationBuilderError::ValidationError(_))
        ));
    }

    #[test]
    fn can_record_due() {
        let mut obligation = obligation_from(initial_events());
        let res = obligation.record_due(Utc::now().date_naive()).unwrap();
        assert_eq!(res.amount, obligation.initial_amount);
    }

    #[test]
    fn ignores_due_recorded_if_after_not_yet_due() {
        let mut obligation = obligation_from(initial_events());
        let _ = obligation.record_due(Utc::now().date_naive());

        assert!(
            obligation
                .record_overdue(Utc::now().date_naive(),)
                .unwrap()
                .did_execute()
        );
        let res = obligation.record_due(Utc::now().date_naive());
        assert!(matches!(res, Idempotent::AlreadyApplied));

        assert!(
            obligation
                .record_defaulted(Utc::now().date_naive(),)
                .unwrap()
                .did_execute()
        );
        let res = obligation.record_due(Utc::now().date_naive());
        assert!(matches!(res, Idempotent::AlreadyApplied));

        let mut events = initial_events();
        events.push(ObligationEvent::Completed {
            effective: Utc::now().date_naive(),
        });
        let mut obligation = obligation_from(events);

        let res = obligation.record_due(Utc::now().date_naive());
        assert!(matches!(res, Idempotent::AlreadyApplied));
    }

    #[test]
    fn can_record_overdue() {
        let mut obligation = obligation_from(initial_events());
        let _ = obligation.record_due(Utc::now().date_naive());
        let res = obligation
            .record_overdue(Utc::now().date_naive())
            .unwrap()
            .unwrap();
        assert_eq!(res.amount, obligation.initial_amount);
    }

    #[test]
    fn ignores_overdue_recorded_if_after_due() {
        let mut obligation = obligation_from(initial_events());
        let _ = obligation.record_due(Utc::now().date_naive());
        let _ = obligation.record_defaulted(Utc::now().date_naive());
        let res = obligation.record_overdue(Utc::now().date_naive()).unwrap();
        assert!(matches!(res, Idempotent::AlreadyApplied));

        let mut events = initial_events();
        events.push(ObligationEvent::Completed {
            effective: Utc::now().date_naive(),
        });
        let mut obligation = obligation_from(events);
        let res = obligation.record_overdue(Utc::now().date_naive()).unwrap();
        assert!(matches!(res, Idempotent::AlreadyApplied));
    }

    #[test]
    fn errors_if_overdue_recorded_before_due() {
        let mut obligation = obligation_from(initial_events());
        let res = obligation.record_overdue(Utc::now().date_naive());
        assert!(matches!(
            res,
            Err(ObligationError::InvalidStatusTransitionToOverdue)
        ));
    }

    #[test]
    fn can_record_defaulted() {
        let mut obligation = obligation_from(initial_events());
        let _ = obligation.record_due(Utc::now().date_naive());
        let res = obligation
            .record_defaulted(Utc::now().date_naive())
            .unwrap()
            .unwrap();
        assert_eq!(res.amount, obligation.initial_amount);

        let mut obligation = obligation_from(initial_events());
        let _ = obligation.record_due(Utc::now().date_naive());
        let _ = obligation.record_overdue(Utc::now().date_naive());
        let res = obligation
            .record_defaulted(Utc::now().date_naive())
            .unwrap()
            .unwrap();
        assert_eq!(res.amount, obligation.initial_amount);
    }

    #[test]
    fn ignores_defaulted_recorded_if_paid() {
        let mut events = initial_events();
        events.push(ObligationEvent::Completed {
            effective: Utc::now().date_naive(),
        });
        let mut obligation = obligation_from(events);
        let res = obligation
            .record_defaulted(Utc::now().date_naive())
            .unwrap();
        assert!(matches!(res, Idempotent::AlreadyApplied));
    }

    #[test]
    fn errors_if_default_recorded_before_due() {
        let mut obligation = obligation_from(initial_events());
        let res = obligation.record_defaulted(Utc::now().date_naive());
        assert!(matches!(
            res,
            Err(ObligationError::InvalidStatusTransitionToDefaulted)
        ));
    }

    #[test]
    fn completes_on_final_payment_allocation() {
        let mut obligation = obligation_from(initial_events());
        obligation
            .allocate_payment(
                UsdCents::ONE,
                PaymentId::new(),
                CalaAccountId::new(),
                Utc::now().date_naive(),
            )
            .unwrap();
        assert_eq!(obligation.status(), ObligationStatus::NotYetDue);

        obligation
            .allocate_payment(
                obligation.outstanding(),
                PaymentId::new(),
                CalaAccountId::new(),
                Utc::now().date_naive(),
            )
            .unwrap();
        assert_eq!(obligation.status(), ObligationStatus::Paid);
    }

    mod is_status_up_to_date {

        use super::*;

        fn due_timestamp(now: DateTime<Utc>) -> DateTime<Utc> {
            now + chrono::Duration::days(1)
        }

        fn overdue_timestamp(now: DateTime<Utc>) -> DateTime<Utc> {
            now + chrono::Duration::days(2)
        }

        fn defaulted_timestamp(now: DateTime<Utc>) -> DateTime<Utc> {
            now + chrono::Duration::days(3)
        }

        fn initial_events(now: DateTime<Utc>) -> Vec<ObligationEvent> {
            vec![ObligationEvent::Initialized {
                id: ObligationId::new(),
                credit_facility_id: CreditFacilityId::new(),
                obligation_type: ObligationType::Disbursal,
                amount: UsdCents::from(10),
                reference: "ref-01".to_string(),
                ledger_tx_id: LedgerTxId::new(),
                receivable_account_ids: ObligationReceivableAccountIds {
                    not_yet_due: CalaAccountId::new(),
                    due: CalaAccountId::new(),
                    overdue: CalaAccountId::new(),
                },
                not_yet_due_accounts: ObligationAccounts {
                    receivable_account_id: CalaAccountId::new(),
                },
                due_accounts: ObligationAccounts {
                    receivable_account_id: CalaAccountId::new(),
                },
                overdue_accounts: ObligationAccounts {
                    receivable_account_id: CalaAccountId::new(),
                },
                defaulted_account_id: CalaAccountId::new(),
                due_date: due_timestamp(now).into(),
                overdue_date: Some(overdue_timestamp(now).into()),
                defaulted_date: Some(defaulted_timestamp(now).into()),
                liquidation_date: None,
                effective: Utc::now().date_naive(),
            }]
        }

        #[test]
        fn expected_not_yet_due_status_not_yet_due() {
            let now = Utc::now();
            let obligation = obligation_from(initial_events(now));
            assert_eq!(obligation.expected_status(now), ObligationStatus::NotYetDue);
            assert_eq!(obligation.status(), ObligationStatus::NotYetDue);
            assert!(obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_due_status_not_yet_due() {
            let now = Utc::now();
            let obligation = obligation_from(initial_events(now));

            let now = due_timestamp(Utc::now());
            assert_eq!(obligation.expected_status(now), ObligationStatus::Due);
            assert_eq!(obligation.status(), ObligationStatus::NotYetDue);
            assert!(!obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_due_status_due() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.push(ObligationEvent::DueRecorded {
                ledger_tx_id: LedgerTxId::new(),
                due_amount: UsdCents::from(10),
            });
            let obligation = obligation_from(events);

            let now = due_timestamp(Utc::now());
            assert_eq!(obligation.expected_status(now), ObligationStatus::Due);
            assert_eq!(obligation.status(), ObligationStatus::Due);
            assert!(obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_overdue_status_due() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.push(ObligationEvent::DueRecorded {
                ledger_tx_id: LedgerTxId::new(),
                due_amount: UsdCents::from(10),
            });
            let obligation = obligation_from(events);

            let now = overdue_timestamp(Utc::now());
            assert_eq!(obligation.expected_status(now), ObligationStatus::Overdue);
            assert_eq!(obligation.status(), ObligationStatus::Due);
            assert!(!obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_overdue_status_overdue() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.extend([
                ObligationEvent::DueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    due_amount: UsdCents::from(10),
                },
                ObligationEvent::OverdueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    overdue_amount: UsdCents::from(10),
                },
            ]);
            let obligation = obligation_from(events);

            let now = overdue_timestamp(Utc::now());
            assert_eq!(obligation.expected_status(now), ObligationStatus::Overdue);
            assert_eq!(obligation.status(), ObligationStatus::Overdue);
            assert!(obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_paid_status_paid() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.extend([
                ObligationEvent::DueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    due_amount: UsdCents::from(10),
                },
                ObligationEvent::OverdueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    overdue_amount: UsdCents::from(10),
                },
            ]);

            let now = overdue_timestamp(Utc::now());
            events.push(ObligationEvent::Completed {
                effective: now.date_naive(),
            });
            let obligation = obligation_from(events);

            assert_eq!(obligation.expected_status(now), ObligationStatus::Paid);
            assert_eq!(obligation.status(), ObligationStatus::Paid);
            assert!(obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_defaulted_status_overdue() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.extend([
                ObligationEvent::DueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    due_amount: UsdCents::from(10),
                },
                ObligationEvent::OverdueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    overdue_amount: UsdCents::from(10),
                },
            ]);
            let obligation = obligation_from(events);

            let now = defaulted_timestamp(Utc::now());
            assert_eq!(obligation.expected_status(now), ObligationStatus::Defaulted);
            assert_eq!(obligation.status(), ObligationStatus::Overdue);
            assert!(!obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_defaulted_status_defaulted() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.extend([
                ObligationEvent::DueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    due_amount: UsdCents::from(10),
                },
                ObligationEvent::OverdueRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    overdue_amount: UsdCents::from(10),
                },
                ObligationEvent::DefaultedRecorded {
                    ledger_tx_id: LedgerTxId::new(),
                    defaulted_amount: UsdCents::from(10),
                },
            ]);
            let obligation = obligation_from(events);

            let now = defaulted_timestamp(Utc::now());
            assert_eq!(obligation.expected_status(now), ObligationStatus::Defaulted);
            assert_eq!(obligation.status(), ObligationStatus::Defaulted);
            assert!(obligation.is_status_up_to_date(now));
        }
    }
}
