use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::{CalaAccountId, LedgerTxId, ObligationId, UsdCents};

use super::error::ObligationError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ObligationStatus {
    NotYetDue,
    Due,
    Overdue,
    _Defaulted,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ObligationId")]
pub enum ObligationEvent {
    Initialized {
        id: ObligationId,
        amount: UsdCents,
        reference: String,
        tx_id: LedgerTxId,
        account_to_be_debited_id: CalaAccountId,
        account_to_be_credited_id: CalaAccountId,
        due_date: DateTime<Utc>,
        overdue_date: Option<DateTime<Utc>>,
        defaulted_date: Option<DateTime<Utc>>,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    MarkedDue {
        audit_info: AuditInfo,
    },
    MarkedOverdue {
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Obligation {
    pub id: ObligationId,
    pub tx_id: LedgerTxId,
    pub reference: String,
    pub amount: UsdCents,
    pub account_to_be_debited_id: CalaAccountId,
    pub account_to_be_credited_id: CalaAccountId,
    pub recorded_at: DateTime<Utc>,
    pub(super) events: EntityEvents<ObligationEvent>,
}

impl Obligation {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn overdue_at(&self) -> Option<DateTime<Utc>> {
        self.events
            .iter_all()
            .find_map(|e| match e {
                ObligationEvent::Initialized { overdue_date, .. } => Some(*overdue_date),
                _ => None,
            })
            .expect("Entity was not Initialized")
    }

    pub(super) fn status(&self) -> ObligationStatus {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ObligationEvent::MarkedDue { .. } => Some(ObligationStatus::Due),
                ObligationEvent::MarkedOverdue { .. } => Some(ObligationStatus::Overdue),
                _ => None,
            })
            .unwrap_or(ObligationStatus::NotYetDue)
    }

    pub fn outstanding(&self) -> UsdCents {
        self.events
            .iter_all()
            .fold(UsdCents::from(0), |mut total_sum, event| {
                if let ObligationEvent::Initialized { amount, .. } = event {
                    total_sum += *amount;
                }
                total_sum
            })
    }

    pub(crate) fn record_due(&mut self, audit_info: AuditInfo) -> Idempotent<UsdCents> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ObligationEvent::MarkedDue { .. }
        );

        self.events.push(ObligationEvent::MarkedDue { audit_info });

        Idempotent::Executed(self.outstanding())
    }

    pub(crate) fn record_overdue(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Idempotent<UsdCents>, ObligationError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ObligationEvent::MarkedOverdue { .. }
        );

        if self.status() != ObligationStatus::Due {
            return Err(ObligationError::InvalidStatusTransitionToOverdue);
        }

        self.events
            .push(ObligationEvent::MarkedOverdue { audit_info });

        Ok(Idempotent::Executed(self.outstanding()))
    }
}

impl TryFromEvents<ObligationEvent> for Obligation {
    fn try_from_events(events: EntityEvents<ObligationEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ObligationBuilder::default();
        for event in events.iter_all() {
            match event {
                ObligationEvent::Initialized {
                    id,
                    tx_id,
                    reference,
                    amount,
                    account_to_be_debited_id,
                    account_to_be_credited_id,
                    recorded_at,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .tx_id(*tx_id)
                        .reference(reference.clone())
                        .amount(*amount)
                        .account_to_be_debited_id(*account_to_be_debited_id)
                        .account_to_be_credited_id(*account_to_be_credited_id)
                        .recorded_at(*recorded_at)
                }
                ObligationEvent::MarkedDue { .. } => (),
                ObligationEvent::MarkedOverdue { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewObligation {
    #[builder(setter(into))]
    pub(super) id: ObligationId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    #[builder(setter(strip_option), default)]
    reference: Option<String>,
    #[builder(setter(into))]
    pub(super) tx_id: LedgerTxId,
    #[builder(setter(into))]
    account_to_be_debited_id: CalaAccountId,
    #[builder(setter(into))]
    account_to_be_credited_id: CalaAccountId,
    due_date: DateTime<Utc>,
    #[builder(setter(strip_option), default)]
    overdue_date: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    defaulted_date: Option<DateTime<Utc>>,
    recorded_at: DateTime<Utc>,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewObligation {
    pub fn builder() -> NewObligationBuilder {
        NewObligationBuilder::default()
    }

    pub(crate) fn id(&self) -> ObligationId {
        self.id
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
                reference: self.reference(),
                amount: self.amount,
                tx_id: self.tx_id,
                account_to_be_debited_id: self.account_to_be_debited_id,
                account_to_be_credited_id: self.account_to_be_credited_id,
                due_date: self.due_date,
                overdue_date: self.overdue_date,
                defaulted_date: self.defaulted_date,
                recorded_at: self.recorded_at,
                audit_info: self.audit_info,
            }],
        )
    }
}
