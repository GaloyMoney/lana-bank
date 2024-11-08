use chrono::{DateTime, Utc};

use crate::{primitives::*, terms::CollateralizationState};

use super::CreditFacilityEvent;

pub struct IncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
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

pub enum CreditFacilityHistoryEntry {
    Payment(IncrementalPayment),
    Collateral(CollateralUpdated),
    Origination(CreditFacilityOrigination),
    Collateralization(CollateralizationUpdated),
    Disbursal(DisbursalExecuted),
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a CreditFacilityEvent>,
) -> Vec<CreditFacilityHistoryEntry> {
    let mut history = vec![];
    let mut disbursals = std::collections::HashMap::new();

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

            CreditFacilityEvent::PaymentRecorded {
                disbursal_amount,
                interest_amount,
                recorded_in_ledger_at,
                tx_id,
                ..
            } => {
                history.push(CreditFacilityHistoryEntry::Payment(IncrementalPayment {
                    cents: *disbursal_amount + *interest_amount,
                    recorded_at: *recorded_in_ledger_at,
                    tx_id: *tx_id,
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
            CreditFacilityEvent::DisbursalInitiated { idx, amount, .. } => {
                disbursals.insert(*idx, *amount);
            }
            CreditFacilityEvent::DisbursalConcluded {
                idx,
                recorded_at,
                tx_id: Some(tx_id),
                ..
            } => {
                history.push(CreditFacilityHistoryEntry::Disbursal(DisbursalExecuted {
                    cents: disbursals
                        .remove(idx)
                        .expect("Disbursal must have been initiated"),
                    recorded_at: *recorded_at,
                    tx_id: *tx_id,
                }));
            }
            _ => {}
        }
    }
    history.reverse();
    history
}
