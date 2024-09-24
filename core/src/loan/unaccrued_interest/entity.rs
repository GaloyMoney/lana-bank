use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityError, EntityEvent, EntityEvents},
    loan::{LoanInterestAccrual, LoanUnaccruedInterestIncurred},
    primitives::{
        AuditInfo, LedgerTxId, LoanId, UnaccruedInterestId, UnaccruedInterestIdx, UsdCents,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnaccruedInterestEvent {
    Initialized {
        id: UnaccruedInterestId,
        loan_id: LoanId,
        idx: UnaccruedInterestIdx,
        audit_info: AuditInfo,
    },
    InterestIncurred {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
}

impl EntityEvent for UnaccruedInterestEvent {
    type EntityId = UnaccruedInterestId;
    fn event_table_name() -> &'static str {
        "unaccrued_interest_events"
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct UnaccruedInterest {
    pub id: UnaccruedInterestId,
    pub loan_id: LoanId,
    pub idx: UnaccruedInterestIdx,
    pub(super) events: EntityEvents<UnaccruedInterestEvent>,
}

impl Entity for UnaccruedInterest {
    type Event = UnaccruedInterestEvent;
}

impl TryFrom<EntityEvents<UnaccruedInterestEvent>> for UnaccruedInterest {
    type Error = EntityError;

    fn try_from(events: EntityEvents<UnaccruedInterestEvent>) -> Result<Self, Self::Error> {
        let mut builder = UnaccruedInterestBuilder::default();
        for event in events.iter() {
            match event {
                UnaccruedInterestEvent::Initialized {
                    id, loan_id, idx, ..
                } => builder = builder.id(*id).loan_id(*loan_id).idx(*idx),
                UnaccruedInterestEvent::InterestIncurred { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl UnaccruedInterest {
    pub fn initiate_interest(
        &self,
        outstanding_amount: UsdCents,
    ) -> Result<LoanUnaccruedInterestIncurred, UnaccruedInterestError> {
        let last_interest_payment = self
            .events
            .iter()
            .rev()
            .find_map(|event| match event {
                UnaccruedInterestEvent::InterestIncurred { recorded_at, .. } => Some(*recorded_at),
                _ => None,
            })
            .expect("Should always have interest incurred events");

        let next_idx = self.idx.next();

        let days_in_interest_period = self
            .terms
            .interval
            .period_from(last_interest_payment)
            .next()
            .truncate(expiry_date)
            .ok_or(UnaccruedInterestError::InterestPeriodStartDateInFuture)?
            .days();

        let interest_for_period = self
            .terms
            .annual_rate
            .interest_for_time_period(outstanding_amount, days_in_interest_period);

        let tx_ref = format!("{}-interest-incurred-{}", self.id, next_idx);

        self.idx = next_idx;
        Ok(LoanUnaccruedInterestIncurred {
            interest: interest_for_period,
            tx_ref,
            tx_id: LedgerTxId::new(),
            unaccrued_interest_idx: self.idx,
        })
    }

    pub fn confirm_interest(
        &mut self,
        LoanInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            ..
        }: LoanInterestAccrual,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(UnaccruedInterestEvent::InterestIncurred {
            tx_id,
            tx_ref,
            amount: interest,
            recorded_at: executed_at,
            audit_info,
        });
    }
}

#[derive(Debug)]
pub struct NewUnaccruedInterest {
    pub(super) id: UnaccruedInterestId,
    pub(super) loan_id: LoanId,
    pub(super) idx: UnaccruedInterestIdx,
    pub(super) audit_info: AuditInfo,
}

impl NewUnaccruedInterest {
    pub fn new(loan_id: LoanId, idx: UnaccruedInterestIdx, audit_info: AuditInfo) -> Self {
        Self {
            id: UnaccruedInterestId::new(),
            loan_id,
            idx,
            audit_info,
        }
    }

    pub fn initial_events(self) -> EntityEvents<UnaccruedInterestEvent> {
        EntityEvents::init(
            self.id,
            [UnaccruedInterestEvent::Initialized {
                id: self.id,
                loan_id: self.loan_id,
                idx: self.idx,
                audit_info: self.audit_info,
            }],
        )
    }
}
