use async_graphql::*;

use super::primitives::{CurrencyCode, Decimal};

#[derive(SimpleObject)]
pub(super) struct Money {
    pub units: Decimal,
    pub currency: CurrencyCode,
}

impl From<crate::primitives::Money> for Money {
    fn from(crate::primitives::Money { amount, currency }: crate::primitives::Money) -> Self {
        Self {
            units: amount.into(),
            currency: currency.into(),
        }
    }
}
