use chrono::{DateTime, Utc};

use crate::primitives::*;

use super::{LoanCollaterizationState, LoanEvent};

pub struct IncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct InterestAccrued {
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

pub struct LoanOrigination {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct CollateralizationUpdated {
    pub state: LoanCollaterizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursements: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub price: PriceOfOneBTC,
}

pub enum LoanHistoryEntry {
    Payment(IncrementalPayment),
    Interest(InterestAccrued),
    Collateral(CollateralUpdated),
    Origination(LoanOrigination),
    Collateralization(CollateralizationUpdated),
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a LoanEvent>,
) -> Vec<LoanHistoryEntry> {
    let mut history = vec![];

    let mut initial_facility = None;
    for event in events {
        match event {
            LoanEvent::Initialized { facility, .. } => initial_facility = Some(*facility),
            LoanEvent::CollateralUpdated {
                abs_diff,
                action,
                recorded_at,
                tx_id,
                ..
            } => match action {
                CollateralAction::Add => {
                    history.push(LoanHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        action: *action,
                        recorded_at: *recorded_at,
                        tx_id: *tx_id,
                    }));
                }
                CollateralAction::Remove => {
                    history.push(LoanHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        action: *action,
                        recorded_at: *recorded_at,
                        tx_id: *tx_id,
                    }));
                }
            },

            LoanEvent::InterestIncurred {
                amount,
                recorded_at,
                tx_id,
                ..
            } => {
                history.push(LoanHistoryEntry::Interest(InterestAccrued {
                    cents: *amount,
                    recorded_at: *recorded_at,
                    tx_id: *tx_id,
                }));
            }

            LoanEvent::PaymentRecorded {
                disbursements_amount,
                interest_amount,
                recorded_at: transaction_recorded_at,
                tx_id,
                ..
            } => {
                history.push(LoanHistoryEntry::Payment(IncrementalPayment {
                    cents: *disbursements_amount + *interest_amount,
                    recorded_at: *transaction_recorded_at,
                    tx_id: *tx_id,
                }));
            }

            LoanEvent::Approved {
                tx_id, recorded_at, ..
            } => {
                history.push(LoanHistoryEntry::Origination(LoanOrigination {
                    cents: initial_facility.expect("Loan must have initial facility"),
                    recorded_at: *recorded_at,
                    tx_id: *tx_id,
                }));
            }

            LoanEvent::CollateralizationChanged {
                state,
                collateral,
                outstanding,
                price,
                recorded_at,
                ..
            } => {
                history.push(LoanHistoryEntry::Collateralization(
                    CollateralizationUpdated {
                        state: *state,
                        collateral: *collateral,
                        outstanding_interest: outstanding.interest,
                        outstanding_disbursements: outstanding.disbursements,
                        price: *price,
                        recorded_at: *recorded_at,
                    },
                ));
            }
            _ => {}
        }
    }
    history.reverse();
    history
}
