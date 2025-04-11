use chrono::{DateTime, Utc};

use crate::{primitives::*, terms::CollateralizationState};

use super::{BalanceUpdatedSource, BalanceUpdatedType, CreditFacilityEvent};

pub struct IncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub payment_id: LedgerTxId, // TODO: change to PaymentAllocationId
}

pub struct CollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: DateTime<Utc>,
    pub action: CollateralAction,
    pub tx_id: LedgerTxId,
}

pub struct CreditFacilityOrigination {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct CollateralizationUpdated {
    pub state: CollateralizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursal: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub price: PriceOfOneBTC,
}

pub struct DisbursalExecuted {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct InterestAccrualsPosted {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub days: i64,
    pub tx_id: LedgerTxId,
}

pub enum CreditFacilityHistoryEntry {
    Payment(IncrementalPayment),
    Collateral(CollateralUpdated),
    Origination(CreditFacilityOrigination),
    Collateralization(CollateralizationUpdated),
    Disbursal(DisbursalExecuted),
    Interest(InterestAccrualsPosted),
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a CreditFacilityEvent>,
) -> Vec<CreditFacilityHistoryEntry> {
    let mut history = vec![];
    let mut disbursals = std::collections::HashMap::new();
    let mut interest_accruals_started_at = std::collections::HashMap::new();
    let mut interest_accruals = std::collections::HashMap::new();

    let mut initial_facility = None;
    for event in events {
        match event {
            CreditFacilityEvent::Initialized { facility, .. } => initial_facility = Some(*facility),
            CreditFacilityEvent::CollateralUpdated {
                abs_diff,
                action,
                recorded_in_ledger_at,
                tx_id,
                ..
            } => match action {
                CollateralAction::Add => {
                    history.push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        action: *action,
                        recorded_at: *recorded_in_ledger_at,
                        tx_id: *tx_id,
                    }));
                }
                CollateralAction::Remove => {
                    history.push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        action: *action,
                        recorded_at: *recorded_in_ledger_at,
                        tx_id: *tx_id,
                    }));
                }
            },

            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(payment_id),
                amount,
                updated_at: recorded_in_ledger_at,
                ..
            } => {
                history.push(CreditFacilityHistoryEntry::Payment(IncrementalPayment {
                    cents: *amount,
                    recorded_at: *recorded_in_ledger_at,
                    payment_id: *payment_id,
                }));
            }

            CreditFacilityEvent::Activated {
                ledger_tx_id,
                activated_at,
                ..
            } => {
                history.push(CreditFacilityHistoryEntry::Origination(
                    CreditFacilityOrigination {
                        cents: initial_facility
                            .expect("CreditFacility must have initial facility amount"),
                        recorded_at: *activated_at,
                        tx_id: *ledger_tx_id,
                    },
                ));
            }

            CreditFacilityEvent::CollateralizationChanged {
                state,
                collateral,
                outstanding,
                price,
                recorded_at,
                ..
            } => {
                history.push(CreditFacilityHistoryEntry::Collateralization(
                    CollateralizationUpdated {
                        state: *state,
                        collateral: *collateral,
                        outstanding_interest: outstanding.interest,
                        outstanding_disbursal: outstanding.disbursed,
                        price: *price,
                        recorded_at: *recorded_at,
                    },
                ));
            }
            CreditFacilityEvent::DisbursalConcluded {
                tx_id,
                obligation_id: Some(obligation_id),
                ..
            } => {
                disbursals.insert(*obligation_id, *tx_id);
            }
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(obligation_id),
                balance_type: BalanceUpdatedType::Disbursal,
                amount,
                updated_at,
                ..
            } => {
                history.push(CreditFacilityHistoryEntry::Disbursal(DisbursalExecuted {
                    cents: *amount,
                    recorded_at: *updated_at,
                    tx_id: disbursals
                        .remove(obligation_id)
                        .expect("ObligationId was not found"),
                }));
            }
            CreditFacilityEvent::InterestAccrualCycleStarted {
                idx, started_at, ..
            } => {
                interest_accruals_started_at.insert(*idx, *started_at);
            }
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx,
                tx_id,
                obligation_id,
                ..
            } => {
                let started_at = interest_accruals_started_at
                    .remove(idx)
                    .expect("Accrual not found");
                interest_accruals.insert(*obligation_id, (started_at, *tx_id));
            }
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(obligation_id),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount,
                updated_at: posted_at,
                ..
            } => {
                let (started_at, tx_id) = interest_accruals
                    .remove(obligation_id)
                    .expect("Accrual must have been initiated");
                let days = (*posted_at - started_at).num_days();
                history.push(CreditFacilityHistoryEntry::Interest(
                    InterestAccrualsPosted {
                        cents: *amount,
                        tx_id,
                        days,
                        recorded_at: *posted_at,
                    },
                ));
            }

            _ => {}
        }
    }
    history.reverse();
    history
}
