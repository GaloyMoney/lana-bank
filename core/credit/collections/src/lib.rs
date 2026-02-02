#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod event;
pub mod ledger;
pub mod obligation;
pub mod payment;
pub mod payment_allocation;
pub mod primitives;
pub mod publisher;

pub use event::CoreCreditCollectionsEvent;
pub use ledger::CollectionsLedger;
pub use obligation::{Obligation, Obligations, error::ObligationError, obligation_cursor};
pub use payment::{Payment, Payments, error::PaymentError};
pub use payment_allocation::{PaymentAllocation, error::PaymentAllocationError};
pub use primitives::{
    BeneficiaryId, CoreCreditCollectionsAction, CoreCreditCollectionsObject, ObligationAction,
    ObligationAllOrOne, ObligationId, PERMISSION_SET_COLLECTIONS_PAYMENT_DATE,
    PERMISSION_SET_COLLECTIONS_VIEWER, PERMISSION_SET_COLLECTIONS_WRITER, PaymentAllocationId,
    PaymentId,
};
pub use publisher::CollectionsPublisher;

#[cfg(feature = "json-schema")]
pub use obligation::ObligationEvent;
