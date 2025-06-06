use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

use super::{error::LiquidationObligationError, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LiquidationObligationId")]
pub enum LiquidationObligationEvent {
    Initialized {
        id: LiquidationObligationId,
        parent_obligation_id: ObligationId,
        credit_facility_id: CreditFacilityId,
        tx_id: LedgerTxId,
        receivable_account_id: CalaAccountId,
        defaulted_account_id: CalaAccountId,
        amount: UsdCents,
        defaulted_date: Option<DateTime<Utc>>,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    },
    DefaultedRecorded {
        tx_id: LedgerTxId,
        amount: UsdCents,
        audit_info: AuditInfo,
    },
    PaymentAllocated {
        amount: UsdCents,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LiquidationObligation {
    pub id: LiquidationObligationId,
    pub parent_obligation_id: ObligationId,
    pub credit_facility_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub receivable_account_id: CalaAccountId,
    pub initial_amount: UsdCents,
    pub defaulted_date: Option<DateTime<Utc>>,
    pub effective: chrono::NaiveDate,
    events: EntityEvents<LiquidationObligationEvent>,
}

impl LiquidationObligation {
    pub fn defaulted_at(&self) -> Option<DateTime<Utc>> {
        self.events.iter_all().find_map(|e| match e {
            LiquidationObligationEvent::Initialized { defaulted_date, .. } => *defaulted_date,
            _ => None,
        })
    }

    pub fn defaulted_account(&self) -> CalaAccountId {
        self.events
            .iter_all()
            .find_map(|e| match e {
                LiquidationObligationEvent::Initialized {
                    defaulted_account_id,
                    ..
                } => Some(*defaulted_account_id),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    fn expected_status(&self, now: DateTime<Utc>) -> LiquidationObligationStatus {
        if let Some(defaulted_date) = self.defaulted_at() {
            if now >= defaulted_date {
                return LiquidationObligationStatus::Defaulted;
            }
        }

        LiquidationObligationStatus::Unpaid
    }

    pub fn status(&self) -> LiquidationObligationStatus {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                LiquidationObligationEvent::DefaultedRecorded { .. } => {
                    Some(LiquidationObligationStatus::Defaulted)
                }
                _ => None,
            })
            .unwrap_or(LiquidationObligationStatus::Unpaid)
    }

    pub fn is_status_up_to_date(&self, now: DateTime<Utc>) -> bool {
        self.status() == self.expected_status(now)
    }

    pub fn outstanding(&self) -> UsdCents {
        self.events
            .iter_all()
            .fold(UsdCents::from(0), |mut total_sum, event| {
                match event {
                    LiquidationObligationEvent::Initialized { amount, .. } => {
                        total_sum += *amount;
                    }
                    LiquidationObligationEvent::PaymentAllocated { amount, .. } => {
                        total_sum -= *amount;
                    }
                    _ => (),
                }
                total_sum
            })
    }

    pub(crate) fn record_defaulted(
        &mut self,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    ) -> Result<
        Idempotent<LiquidationObligationDefaultedReallocationData>,
        LiquidationObligationError,
    > {
        idempotency_guard!(
            self.events.iter_all().rev(),
            LiquidationObligationEvent::DefaultedRecorded { .. }
        );

        let amount = self.outstanding();
        if amount.is_zero() {
            return Ok(Idempotent::Ignored);
        }

        let res = LiquidationObligationDefaultedReallocationData {
            tx_id: LedgerTxId::new(),
            amount,
            receivable_account_id: self.receivable_account_id,
            defaulted_account_id: self.defaulted_account(),
            effective,
        };

        self.events
            .push(LiquidationObligationEvent::DefaultedRecorded {
                tx_id: res.tx_id,
                amount: res.amount,
                audit_info,
            });

        Ok(Idempotent::Executed(res))
    }
}

impl TryFromEvents<LiquidationObligationEvent> for LiquidationObligation {
    fn try_from_events(
        events: EntityEvents<LiquidationObligationEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = LiquidationObligationBuilder::default();
        for event in events.iter_all() {
            match event {
                LiquidationObligationEvent::Initialized {
                    id,
                    parent_obligation_id,
                    credit_facility_id,
                    tx_id,
                    receivable_account_id,
                    amount,
                    defaulted_date,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .parent_obligation_id(*parent_obligation_id)
                        .credit_facility_id(*credit_facility_id)
                        .tx_id(*tx_id)
                        .receivable_account_id(*receivable_account_id)
                        .initial_amount(*amount)
                        .defaulted_date(*defaulted_date)
                        .effective(*effective)
                }
                LiquidationObligationEvent::DefaultedRecorded { .. } => (),
                LiquidationObligationEvent::PaymentAllocated { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLiquidationObligation {
    #[builder(setter(into))]
    pub(crate) id: LiquidationObligationId,
    #[builder(setter(into))]
    pub(crate) parent_obligation_id: ObligationId,
    #[builder(setter(into))]
    pub(crate) credit_facility_id: CreditFacilityId,
    #[builder(setter(into))]
    pub(crate) tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(crate) receivable_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(crate) defaulted_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(crate) amount: UsdCents,
    pub(crate) defaulted_date: Option<DateTime<Utc>>,
    pub(crate) effective: chrono::NaiveDate,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewLiquidationObligation {
    pub fn builder() -> NewLiquidationObligationBuilder {
        NewLiquidationObligationBuilder::default()
    }
}

impl IntoEvents<LiquidationObligationEvent> for NewLiquidationObligation {
    fn into_events(self) -> EntityEvents<LiquidationObligationEvent> {
        EntityEvents::init(
            self.id,
            [LiquidationObligationEvent::Initialized {
                id: self.id,
                parent_obligation_id: self.parent_obligation_id,
                credit_facility_id: self.credit_facility_id,
                tx_id: self.tx_id,
                receivable_account_id: self.receivable_account_id,
                defaulted_account_id: self.defaulted_account_id,
                amount: self.amount,
                defaulted_date: self.defaulted_date,
                effective: self.effective,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use audit::{AuditEntryId, AuditInfo};

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn liquidation_obligation_from(
        events: Vec<LiquidationObligationEvent>,
    ) -> LiquidationObligation {
        LiquidationObligation::try_from_events(EntityEvents::init(
            LiquidationObligationId::new(),
            events,
        ))
        .unwrap()
    }

    fn initial_events() -> Vec<LiquidationObligationEvent> {
        vec![LiquidationObligationEvent::Initialized {
            id: LiquidationObligationId::new(),
            parent_obligation_id: ObligationId::new(),
            credit_facility_id: CreditFacilityId::new(),
            tx_id: LedgerTxId::new(),
            receivable_account_id: CalaAccountId::new(),
            defaulted_account_id: CalaAccountId::new(),
            amount: UsdCents::from(10),
            defaulted_date: None,
            effective: Utc::now().date_naive(),
            audit_info: dummy_audit_info(),
        }]
    }

    #[test]
    fn can_record_defaulted() {
        let mut liquidation_obligation = liquidation_obligation_from(initial_events());
        let res = liquidation_obligation
            .record_defaulted(Utc::now().date_naive(), dummy_audit_info())
            .unwrap()
            .unwrap();
        assert_eq!(res.amount, liquidation_obligation.initial_amount);
    }

    mod is_status_up_to_date {

        use super::*;

        fn defaulted_timestamp(now: DateTime<Utc>) -> DateTime<Utc> {
            now + chrono::Duration::days(1)
        }

        fn initial_events(now: DateTime<Utc>) -> Vec<LiquidationObligationEvent> {
            vec![LiquidationObligationEvent::Initialized {
                id: LiquidationObligationId::new(),
                parent_obligation_id: ObligationId::new(),
                credit_facility_id: CreditFacilityId::new(),
                tx_id: LedgerTxId::new(),
                receivable_account_id: CalaAccountId::new(),
                defaulted_account_id: CalaAccountId::new(),
                amount: UsdCents::from(10),
                defaulted_date: Some(defaulted_timestamp(now)),
                effective: Utc::now().date_naive(),
                audit_info: dummy_audit_info(),
            }]
        }

        #[test]
        fn expected_unpaid_status_unpaid() {
            let now = Utc::now();
            let obligation = liquidation_obligation_from(initial_events(now));
            assert_eq!(
                obligation.expected_status(now),
                LiquidationObligationStatus::Unpaid
            );
            assert_eq!(obligation.status(), LiquidationObligationStatus::Unpaid);
            assert!(obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_defaulted_status_unpaid() {
            let now = Utc::now();
            let obligation = liquidation_obligation_from(initial_events(now));

            let now = defaulted_timestamp(Utc::now());
            assert_eq!(
                obligation.expected_status(now),
                LiquidationObligationStatus::Defaulted
            );
            assert_eq!(obligation.status(), LiquidationObligationStatus::Unpaid);
            assert!(!obligation.is_status_up_to_date(now));
        }

        #[test]
        fn expected_defaulted_status_defaulted() {
            let now = Utc::now();
            let mut events = initial_events(now);
            events.push(LiquidationObligationEvent::DefaultedRecorded {
                tx_id: LedgerTxId::new(),
                amount: UsdCents::from(10),
                audit_info: dummy_audit_info(),
            });
            let obligation = liquidation_obligation_from(events);

            let now = defaulted_timestamp(Utc::now());
            assert_eq!(
                obligation.expected_status(now),
                LiquidationObligationStatus::Defaulted
            );
            assert_eq!(obligation.status(), LiquidationObligationStatus::Defaulted);
            assert!(obligation.is_status_up_to_date(now));
        }
    }
}
