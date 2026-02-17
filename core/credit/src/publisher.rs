use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
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
    pending_credit_facility::{
        PendingCreditFacility, PendingCreditFacilityEvent, error::PendingCreditFacilityError,
    },
    public::*,
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
                Initialized { .. } => Some(CoreCreditEvent::FacilityActivated {
                    entity: PublicCreditFacility::from(entity),
                }),
                Completed { .. } => Some(CoreCreditEvent::FacilityCompleted {
                    entity: PublicCreditFacility::from(entity),
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
                PartialLiquidationInitiated { .. } => {
                    Some(CoreCreditEvent::PartialLiquidationInitiated {
                        entity: PublicCreditFacility::from(entity),
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
                Initialized { .. } => Some(CoreCreditEvent::FacilityProposalCreated {
                    entity: PublicCreditFacilityProposal::from(entity),
                }),
                ApprovalProcessConcluded { .. } => {
                    Some(CoreCreditEvent::FacilityProposalConcluded {
                        entity: PublicCreditFacilityProposal::from(entity),
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
                    entity: PublicPendingCreditFacility::from(entity),
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
            .flat_map(|event| match &event.event {
                UpdatedViaManualInput { .. } | UpdatedViaCustodianSync { .. } => {
                    vec![CoreCreditEvent::FacilityCollateralUpdated {
                        entity: PublicCollateral::from(entity),
                    }]
                }
                UpdatedViaLiquidation {
                    abs_diff,
                    ledger_tx_id,
                    liquidation_id,
                    ..
                } => vec![
                    CoreCreditEvent::FacilityCollateralUpdated {
                        entity: PublicCollateral::from(entity),
                    },
                    CoreCreditEvent::PartialLiquidationCollateralSentOut {
                        liquidation_id: *liquidation_id,
                        credit_facility_id: entity.credit_facility_id,
                        amount: *abs_diff,
                        ledger_tx_id: *ledger_tx_id,
                        recorded_at: event.recorded_at,
                        effective: event.recorded_at.date_naive(),
                    },
                ],
                LiquidationProceedsReceived {
                    liquidation_id,
                    amount,
                    ledger_tx_id,
                    payment_id,
                } => {
                    vec![CoreCreditEvent::PartialLiquidationProceedsReceived {
                        liquidation_id: *liquidation_id,
                        credit_facility_id: entity.credit_facility_id,
                        amount: *amount,
                        payment_id: *payment_id,
                        ledger_tx_id: *ledger_tx_id,
                        recorded_at: event.recorded_at,
                        effective: event.recorded_at.date_naive(),
                    }]
                }
                LiquidationCompleted { liquidation_id } => {
                    vec![CoreCreditEvent::PartialLiquidationCompleted {
                        liquidation_id: *liquidation_id,
                        credit_facility_id: entity.credit_facility_id,
                    }]
                }
                _ => vec![],
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
                Settled { .. } => Some(CoreCreditEvent::DisbursalSettled {
                    entity: PublicDisbursal::from(entity),
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
                InterestAccrualsPosted { .. } => Some(CoreCreditEvent::AccrualPosted {
                    entity: PublicInterestAccrualCycle::from(entity),
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
