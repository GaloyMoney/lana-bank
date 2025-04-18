use core_money::UsdCents;
use outbox::{Outbox, OutboxEventMarker};

use crate::{
    credit_facility::{error::CreditFacilityError, CreditFacility, CreditFacilityEvent},
    disbursal::{error::DisbursalError, Disbursal, DisbursalEvent},
    event::*,
    interest_accrual_cycle::{
        error::InterestAccrualCycleError, InterestAccrualCycle, InterestAccrualCycleEvent,
    },
    payment_allocation::{
        error::PaymentAllocationError, PaymentAllocation, PaymentAllocationEvent,
    },
    primitives::ObligationType,
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
                Initialized { .. } => Some(CoreCreditEvent::FacilityCreated {
                    id: entity.id,
                    created_at: entity.created_at(),
                }),
                ApprovalProcessConcluded { approved, .. } if *approved => {
                    Some(CoreCreditEvent::FacilityApproved { id: entity.id })
                }
                Activated { activated_at, .. } => Some(CoreCreditEvent::FacilityActivated {
                    id: entity.id,
                    activated_at: *activated_at,
                }),
                Completed { completed_at, .. } => Some(CoreCreditEvent::FacilityCompleted {
                    id: entity.id,
                    completed_at: *completed_at,
                }),
                CollateralUpdated {
                    total_collateral,
                    abs_diff,
                    action,
                    recorded_in_ledger_at,
                    ..
                } => {
                    let action = match action {
                        crate::primitives::CollateralAction::Add => {
                            FacilityCollateralUpdateAction::Add
                        }
                        crate::primitives::CollateralAction::Remove => {
                            FacilityCollateralUpdateAction::Remove
                        }
                    };

                    Some(CoreCreditEvent::FacilityCollateralUpdated {
                        id: entity.id,
                        new_amount: *total_collateral,
                        abs_diff: *abs_diff,
                        action,
                        recorded_at: *recorded_in_ledger_at,
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
                    ..
                } => Some(CoreCreditEvent::DisbursalExecuted {
                    id: entity.facility_id,
                    amount: *amount,
                    recorded_at: *recorded_at,
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
                InterestAccrualsPosted {
                    total, posted_at, ..
                } => Some(CoreCreditEvent::AccrualExecuted {
                    id: entity.credit_facility_id,
                    amount: *total,
                    posted_at: *posted_at,
                }),

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
                    obligation_type,
                    amount,
                    ..
                } => match obligation_type {
                    ObligationType::Disbursal => CoreCreditEvent::FacilityRepaymentRecorded {
                        id: entity.credit_facility_id,
                        disbursal_amount: *amount,
                        interest_amount: UsdCents::ZERO,
                        recorded_at: event.recorded_at,
                    },
                    ObligationType::Interest => CoreCreditEvent::FacilityRepaymentRecorded {
                        id: entity.credit_facility_id,
                        disbursal_amount: UsdCents::ZERO,
                        interest_amount: *amount,
                        recorded_at: event.recorded_at,
                    },
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}
