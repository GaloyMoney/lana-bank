use crate::primitives::CalaAccountId;

#[derive(Debug, Clone, Copy)]
pub struct PaymentSourceAccountId(CalaAccountId);

// Note: DO NOT implement `From<CalaAccountId> for PaymentSourceAccountId` since
//       we want to avoid trivially passing any CalaAccountId into a place that
//       expects PaymentSourceAccountId.

impl From<PaymentSourceAccountId> for CalaAccountId {
    fn from(account_id: PaymentSourceAccountId) -> Self {
        account_id.0
    }
}

impl PaymentSourceAccountId {
    pub const fn new(account_id: CalaAccountId) -> Self {
        Self(account_id)
    }
}
