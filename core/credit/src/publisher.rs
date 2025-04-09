use outbox::{Outbox, OutboxEventMarker};

use crate::{
    credit_facility::{error::CreditFacilityError, CreditFacility, CreditFacilityEvent},
    event::*,
    obligation::{error::ObligationError, Obligation, ObligationEvent, ObligationType},
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
                PaymentRecorded {
                    disbursal_amount,
                    interest_amount,
                    recorded_at: recorded_in_ledger_at,
                    ..
                } => Some(CoreCreditEvent::FacilityRepaymentRecorded {
                    id: entity.id,
                    disbursal_amount: *disbursal_amount,
                    interest_amount: *interest_amount,
                    recorded_at: *recorded_in_ledger_at,
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

    pub async fn publish_obligation(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), ObligationError> {
        use ObligationEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized {
                    obligation_type, ..
                } => match obligation_type {
                    ObligationType::Disbursal => Some(CoreCreditEvent::DisbursalExecuted {
                        id: entity.credit_facility_id,
                        amount: entity.initial_amount,
                        recorded_at: entity.recorded_at,
                    }),
                    ObligationType::Interest => Some(CoreCreditEvent::AccrualExecuted {
                        id: entity.credit_facility_id,
                        amount: entity.initial_amount,
                        posted_at: entity.recorded_at,
                    }),
                },

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}
