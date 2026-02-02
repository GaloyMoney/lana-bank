#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod primitives;

pub use primitives::{
    BeneficiaryId, CoreCreditCollectionsAction, CoreCreditCollectionsObject, ObligationAction,
    ObligationAllOrOne, ObligationId, PERMISSION_SET_COLLECTIONS_PAYMENT_DATE,
    PERMISSION_SET_COLLECTIONS_VIEWER, PERMISSION_SET_COLLECTIONS_WRITER, PaymentAllocationId,
    PaymentId,
};
