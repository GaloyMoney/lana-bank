#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
pub mod event;
pub mod ledger;
pub mod primitives;
pub mod publisher;

pub mod obligation;
pub mod payment;
pub mod payment_allocation;

pub use error::*;
pub use event::*;
pub use ledger::*;
pub use primitives::*;
pub use publisher::*;

pub use obligation::Obligation;
pub use obligation::error::ObligationError;
pub use payment::Payment;
pub use payment::error::PaymentError;
pub use payment_allocation::PaymentAllocation;
pub use payment_allocation::error::PaymentAllocationError;
