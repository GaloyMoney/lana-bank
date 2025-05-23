use outbox::{Outbox, OutboxEventMarker};

use crate::{
    collateral::{error::CollateralError, Collateral, CollateralEvent},
    credit_facility::{error::CreditFacilityError, CreditFacility, CreditFacilityEvent},
    disbursal::{error::DisbursalError, Disbursal, DisbursalEvent},
    event::*,
    interest_accrual_cycle::{
        error::InterestAccrualCycleError, InterestAccrualCycle, InterestAccrualCycleEvent,
    },
    obligation::{error::ObligationError, Obligation, ObligationEvent},
    payment_allocation::{
        error::PaymentAllocationError, PaymentAllocation, PaymentAllocationEvent,
    },
};

pub struct CreditFacilityPublisher<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for CreditFacilityPublisher<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> CreditFacilityPublisher<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_facility(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &CreditFacility,
        new_events: es_entity::LastPersisted<'_, CreditFacilityEvent>,
    ) -> Result<(), CreditFacilityError> {
        use CreditFacilityEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { amount, terms, .. } => Some(CoreCreditEvent::FacilityCreated {
                    id: entity.id,
                    terms: *terms,
                    amount: *amount,
                    created_at: entity.created_at(),
                }),
                ApprovalProcessConcluded { approved, .. } if *approved => {
                    Some(CoreCreditEvent::FacilityApproved { id: entity.id })
                }
                Activated {
                    activated_at,
                    ledger_tx_id,
                    ..
                } => Some(CoreCreditEvent::FacilityActivated {
                    id: entity.id,
                    activation_tx_id: *ledger_tx_id,
                    activated_at: *activated_at,
                    amount: entity.amount,
                }),
                Completed { .. } => Some(CoreCreditEvent::FacilityCompleted {
                    id: entity.id,
                    completed_at: event.recorded_at,
                }),
                CollateralizationStateChanged {
                    state,
                    collateral,
                    outstanding,
                    price,
                    ..
                } => Some(CoreCreditEvent::FacilityCollateralizationChanged {
                    id: entity.id,
                    state: *state,
                    recorded_at: event.recorded_at,
                    collateral: *collateral,
                    outstanding: *outstanding,
                    price: *price,
                }),

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_collateral(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CollateralError> {
        use CollateralEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                Updated {
                    abs_diff,
                    action,
                    ledger_tx_id,
                    ..
                } => Some(CoreCreditEvent::FacilityCollateralUpdated {
                    ledger_tx_id: *ledger_tx_id,
                    abs_diff: *abs_diff,
                    action: *action,
                    recorded_at: entity.created_at(),
                    new_amount: entity.amount,
                    credit_facility_id: entity.credit_facility_id,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(db.tx(), events).await?;

        Ok(())
    }

    pub async fn publish_disbursal(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Disbursal,
        new_events: es_entity::LastPersisted<'_, DisbursalEvent>,
    ) -> Result<(), DisbursalError> {
        use DisbursalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Settled {
                    amount,
                    recorded_at,
                    ledger_tx_id,
                    ..
                } => Some(CoreCreditEvent::DisbursalSettled {
                    credit_facility_id: entity.facility_id,
                    amount: *amount,
                    recorded_at: *recorded_at,
                    ledger_tx_id: *ledger_tx_id,
                }),

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_interest_accrual_cycle(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &InterestAccrualCycle,
        new_events: es_entity::LastPersisted<'_, InterestAccrualCycleEvent>,
    ) -> Result<(), InterestAccrualCycleError> {
        use InterestAccrualCycleEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                InterestAccrualsPosted { total, tx_id, .. } => {
                    Some(CoreCreditEvent::AccrualPosted {
                        credit_facility_id: entity.credit_facility_id,
                        ledger_tx_id: *tx_id,
                        amount: *total,
                        period: entity.period,
                        recorded_at: event.recorded_at,
                    })
                }

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_payment_allocation(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &PaymentAllocation,
        new_events: es_entity::LastPersisted<'_, PaymentAllocationEvent>,
    ) -> Result<(), PaymentAllocationError> {
        use PaymentAllocationEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized {
                    id,
                    obligation_id,
                    obligation_type,
                    amount,
                    ..
                } => CoreCreditEvent::FacilityRepaymentRecorded {
                    credit_facility_id: entity.credit_facility_id,
                    obligation_id: *obligation_id,
                    obligation_type: *obligation_type,
                    payment_id: *id,
                    amount: *amount,
                    recorded_at: event.recorded_at,
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_obligation(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), ObligationError> {
        use ObligationEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreCreditEvent::ObligationCreated {
                    id: entity.id,
                    obligation_type: entity.obligation_type,
                    credit_facility_id: entity.credit_facility_id,
                    amount: entity.initial_amount,

                    due_at: entity.due_at(),
                    overdue_at: entity.overdue_at(),
                    defaulted_at: entity.defaulted_at(),
                    recorded_at: entity.recorded_at,
                }),
                DueRecorded { amount, .. } => Some(CoreCreditEvent::ObligationDue {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    obligation_type: entity.obligation_type,
                    amount: *amount,
                }),
                OverdueRecorded { amount, .. } => Some(CoreCreditEvent::ObligationOverdue {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    amount: *amount,
                }),
                DefaultedRecorded { amount, .. } => Some(CoreCreditEvent::ObligationDefaulted {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    amount: *amount,
                }),
                Completed { .. } => Some(CoreCreditEvent::ObligationCompleted {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}
