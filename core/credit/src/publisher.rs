use outbox::{Outbox, OutboxEventMarker};

use crate::{
    EffectiveDate,
    collateral::{Collateral, CollateralEvent, error::CollateralError},
    credit_facility::{CreditFacility, CreditFacilityEvent, error::CreditFacilityError},
    credit_facility_proposal::{
        CreditFacilityProposal, CreditFacilityProposalEvent, error::CreditFacilityProposalError,
    },
    disbursal::{Disbursal, DisbursalEvent, error::DisbursalError},
    event::*,
    interest_accrual_cycle::{
        InterestAccrualCycle, InterestAccrualCycleEvent, error::InterestAccrualCycleError,
    },
    liquidation_process::{
        LiquidationProcess, LiquidationProcessEvent, error::LiquidationProcessError,
    },
    obligation::{Obligation, ObligationEvent, error::ObligationError},
    obligation_installment::{
        ObligationInstallment, ObligationInstallmentEvent, error::ObligationInstallmentError,
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
        op: &mut impl es_entity::AtomicOperation,
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
                    state: *state,
                    recorded_at: event.recorded_at,
                    effective: event.recorded_at.date_naive(),
                    collateral: *collateral,
                    outstanding: *outstanding,
                    price: *price,
                }),

                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_proposal(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacilityProposal,
        new_events: es_entity::LastPersisted<'_, CreditFacilityProposalEvent>,
    ) -> Result<(), CreditFacilityProposalError> {
        use CreditFacilityProposalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                ApprovalProcessConcluded { approved, .. } if *approved => {
                    Some(CoreCreditEvent::FacilityProposalApproved { id: entity.id })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_collateral(
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
                    action,
                    ledger_tx_id,
                    ..
                }
                | UpdatedViaCustodianSync {
                    abs_diff,
                    action,
                    ledger_tx_id,
                    ..
                } => Some(CoreCreditEvent::FacilityCollateralUpdated {
                    ledger_tx_id: *ledger_tx_id,
                    abs_diff: *abs_diff,
                    action: *action,
                    recorded_at: event.recorded_at,
                    effective: event.recorded_at.date_naive(),
                    new_amount: entity.amount,
                    credit_facility_id: entity.credit_facility_id,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }

    pub async fn publish_disbursal(
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

    pub async fn publish_interest_accrual_cycle(
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

    pub async fn publish_obligation_installment(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &ObligationInstallment,
        new_events: es_entity::LastPersisted<'_, ObligationInstallmentEvent>,
    ) -> Result<(), ObligationInstallmentError> {
        use ObligationInstallmentEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized {
                    id,
                    obligation_id,
                    obligation_type,
                    amount,
                    effective,
                    ..
                } => CoreCreditEvent::FacilityRepaymentRecorded {
                    credit_facility_id: entity.credit_facility_id,
                    obligation_id: *obligation_id,
                    obligation_type: *obligation_type,
                    payment_id: *id,
                    amount: *amount,
                    recorded_at: event.recorded_at,
                    effective: *effective,
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_obligation(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), ObligationError> {
        use ObligationEvent::*;

        let dates = entity.lifecycle_dates();
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { effective, .. } => Some(CoreCreditEvent::ObligationCreated {
                    id: entity.id,
                    obligation_type: entity.obligation_type,
                    credit_facility_id: entity.credit_facility_id,
                    amount: entity.initial_amount,

                    due_at: dates.due,
                    overdue_at: dates.overdue,
                    defaulted_at: dates.defaulted,
                    recorded_at: event.recorded_at,
                    effective: *effective,
                }),
                DueRecorded {
                    due_amount: amount, ..
                } => Some(CoreCreditEvent::ObligationDue {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    obligation_type: entity.obligation_type,
                    amount: *amount,
                }),
                OverdueRecorded {
                    overdue_amount: amount,
                    ..
                } => Some(CoreCreditEvent::ObligationOverdue {
                    id: entity.id,
                    credit_facility_id: entity.credit_facility_id,
                    amount: *amount,
                }),
                DefaultedRecorded {
                    defaulted_amount: amount,
                    ..
                } => Some(CoreCreditEvent::ObligationDefaulted {
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
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_liquidation_process(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &LiquidationProcess,
        new_events: es_entity::LastPersisted<'_, LiquidationProcessEvent>,
    ) -> Result<(), LiquidationProcessError> {
        use LiquidationProcessEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized {
                    id,
                    obligation_id,
                    credit_facility_id,
                    ledger_tx_id,
                    initial_amount,
                    effective,
                    ..
                } => CoreCreditEvent::LiquidationProcessStarted {
                    id: *id,
                    obligation_id: *obligation_id,
                    credit_facility_id: *credit_facility_id,
                    amount: *initial_amount,
                    effective: *effective,
                    ledger_tx_id: *ledger_tx_id,
                    recorded_at: event.recorded_at,
                },
                Completed { .. } => CoreCreditEvent::LiquidationProcessConcluded {
                    id: entity.id,
                    obligation_id: entity.obligation_id,
                    credit_facility_id: entity.credit_facility_id,
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
