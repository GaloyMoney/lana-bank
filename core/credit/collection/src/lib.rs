#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
pub mod event;
pub mod ledger;
pub mod obligation;
pub mod payment;
pub mod payment_allocation;
pub mod primitives;
pub mod publisher;

pub use error::CoreCreditCollectionError;
pub use event::CoreCreditCollectionEvent;
pub use ledger::CollectionLedger;
pub use obligation::{Obligation, Obligations, error::ObligationError, obligation_cursor};
pub use payment::{Payment, Payments, error::PaymentError};
pub use payment_allocation::{PaymentAllocation, error::PaymentAllocationError};
pub use primitives::ObligationReceivableAccountIds;
pub use primitives::{
    BeneficiaryId, CoreCreditCollectionAction, CoreCreditCollectionObject, ObligationAction,
    ObligationAllOrOne, ObligationId, PERMISSION_SET_COLLECTION_PAYMENT_DATE,
    PERMISSION_SET_COLLECTION_VIEWER, PERMISSION_SET_COLLECTION_WRITER, PaymentAllocationId,
    PaymentId,
};
pub use publisher::CollectionPublisher;

#[cfg(feature = "json-schema")]
pub use obligation::ObligationEvent;
