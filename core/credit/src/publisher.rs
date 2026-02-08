use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    EffectiveDate,
    collateral::{Collateral, CollateralEvent, error::CollateralError},
    credit_facility::{
        CreditFacility, CreditFacilityEvent,
        error::CreditFacilityError,
        interest_accrual_cycle::{
            InterestAccrualCycle, InterestAccrualCycleEvent, error::InterestAccrualCycleError,
        },
    },
    credit_facility_proposal::{
        CreditFacilityProposal, CreditFacilityProposalEvent, error::CreditFacilityProposalError,
    },
    disbursal::{Disbursal, DisbursalEvent, error::DisbursalError},
    event::*,
    pending_credit_facility::{
        PendingCreditFacility, PendingCreditFacilityEvent, error::PendingCreditFacilityError,
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

    #[record_error_severity]
    #[instrument(name = "credit.publisher.publish_facility_in_op", skip_all)]
    pub async fn publish_facility_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacility,
        new_events: es_entity::LastPersisted<'_, CreditFacilityEvent>,
    ) -> Result<(), CreditFacilityError> {
        use CreditFacilityEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized {
                    amount,
                    ledger_tx_id,
                    ..
                } => Some(CoreCreditEvent::FacilityActivated {
                    id: entity.id,
                    activation_tx_id: *ledger_tx_id,
                    amount: *amount,
                    activated_at: entity.created_at(),
                }),
                Completed { .. } => Some(CoreCreditEvent::FacilityCompleted {
                    id: entity.id,
                    completed_at: event.recorded_at,
                }),
                CollateralizationStateChanged {
                    collateralization_state: state,
                    collateral,
                    outstanding,
                    price,
                    ..
                } => Some(CoreCreditEvent::FacilityCollateralizationChanged {
                    id: entity.id,
                    customer_id: entity.customer_id,
                    state: *state,
                    recorded_at: event.recorded_at,
                    effective: event.recorded_at.date_naive(),
                    collateral: *collateral,
                    outstanding: *outstanding,
                    price: *price,
                }),
                PartialLiquidationInitiated {
                    liquidation_id,
                    trigger_price,
                    initially_expected_to_receive,
                    initially_estimated_to_liquidate,
                } => Some(CoreCreditEvent::PartialLiquidationInitiated {
                    credit_facility_id: entity.id,
                    collateral_id: entity.collateral_id,
                    liquidation_id: *liquidation_id,
                    customer_id: entity.customer_id,
                    trigger_price: *trigger_price,
                    initially_expected_to_receive: *initially_expected_to_receive,
                    initially_estimated_to_liquidate: *initially_estimated_to_liquidate,
                }),

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "credit.publisher.publish_proposal_in_op", skip_all)]
    pub async fn publish_proposal_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacilityProposal,
        new_events: es_entity::LastPersisted<'_, CreditFacilityProposalEvent>,
    ) -> Result<(), CreditFacilityProposalError> {
        use CreditFacilityProposalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { amount, terms, .. } => {
                    Some(CoreCreditEvent::FacilityProposalCreated {
                        id: entity.id,
                        terms: *terms,
                        amount: *amount,
                        created_at: entity.created_at(),
                    })
                }
                ApprovalProcessConcluded { status, .. } => {
                    Some(CoreCreditEvent::FacilityProposalConcluded {
                        id: entity.id,
                        status: *status,
                    })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.publisher.publish_pending_credit_facility_in_op",
        skip_all
    )]
    pub async fn publish_pending_credit_facility_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PendingCreditFacility,
        new_events: es_entity::LastPersisted<'_, PendingCreditFacilityEvent>,
    ) -> Result<(), PendingCreditFacilityError> {
        use PendingCreditFacilityEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                CollateralizationStateChanged {
                    collateralization_state,
                    collateral,
                    price,
                } => Some(
                    CoreCreditEvent::PendingCreditFacilityCollateralizationChanged {
                        id: entity.id,
                        state: *collateralization_state,
                        collateral: *collateral,
                        price: *price,
                        recorded_at: event.recorded_at,
                        effective: event.recorded_at.date_naive(),
                    },
                ),
                Completed { .. } => Some(CoreCreditEvent::PendingCreditFacilityCompleted {
                    id: entity.id,
                    status: entity.status(),
                    recorded_at: event.recorded_at,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "credit.publisher.publish_collateral_in_op", skip_all)]
    pub async fn publish_collateral_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CollateralError> {
        use CollateralEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                UpdatedViaManualInput {
                    abs_diff,
                    direction,
                    ledger_tx_id,
                    ..
                }
                | UpdatedViaCustodianSync {
                    abs_diff,
                    direction,
                    ledger_tx_id,
                    ..
                } => Some(CoreCreditEvent::FacilityCollateralUpdated {
                    ledger_tx_id: *ledger_tx_id,
                    abs_diff: *abs_diff,
                    direction: *direction,
                    recorded_at: event.recorded_at,
                    effective: event.recorded_at.date_naive(),
                    new_amount: entity.amount,
                    credit_facility_id: entity.credit_facility_id,
                    pending_credit_facility_id: entity.pending_credit_facility_id,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "credit.publisher.publish_disbursal_in_op", skip_all)]
    pub async fn publish_disbursal_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Disbursal,
        new_events: es_entity::LastPersisted<'_, DisbursalEvent>,
    ) -> Result<(), DisbursalError> {
        use DisbursalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Settled {
                    amount,
                    ledger_tx_id,
                    effective,
                    ..
                } => Some(CoreCreditEvent::DisbursalSettled {
                    credit_facility_id: entity.facility_id,
                    amount: *amount,
                    recorded_at: event.recorded_at,
                    effective: *effective,
                    ledger_tx_id: *ledger_tx_id,
                }),

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.publisher.publish_interest_accrual_cycle_in_op",
        skip_all
    )]
    pub async fn publish_interest_accrual_cycle_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &InterestAccrualCycle,
        new_events: es_entity::LastPersisted<'_, InterestAccrualCycleEvent>,
    ) -> Result<(), InterestAccrualCycleError> {
        use InterestAccrualCycleEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                InterestAccrualsPosted {
                    total,
                    ledger_tx_id: tx_id,
                    effective,
                    ..
                } => Some(CoreCreditEvent::AccrualPosted {
                    credit_facility_id: entity.credit_facility_id,
                    ledger_tx_id: *tx_id,
                    amount: *total,
                    period: entity.period,
                    due_at: EffectiveDate::from(entity.period.end),
                    recorded_at: event.recorded_at,
                    effective: *effective,
                }),

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
