use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{Btc, MinorUnits, Usd};

pub use super::error::CurrencyBagError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CurrencyBag {
    usd: Option<MinorUnits<Usd>>,
    btc: Option<MinorUnits<Btc>>,
}

impl CurrencyBag {
    pub fn new() -> Self {
        Self {
            usd: None,
            btc: None,
        }
    }

    pub fn with_usd(mut self) -> Self {
        self.usd = Some(MinorUnits::ZERO);
        self
    }

    pub fn with_btc(mut self) -> Self {
        self.btc = Some(MinorUnits::ZERO);
        self
    }

    pub fn with_usd_amount(mut self, amount: MinorUnits<Usd>) -> Self {
        self.usd = Some(amount);
        self
    }

    pub fn with_btc_amount(mut self, amount: MinorUnits<Btc>) -> Self {
        self.btc = Some(amount);
        self
    }

    pub fn usd(&self) -> Option<MinorUnits<Usd>> {
        self.usd
    }

    pub fn btc(&self) -> Option<MinorUnits<Btc>> {
        self.btc
    }

    pub fn supports_usd(&self) -> bool {
        self.usd.is_some()
    }

    pub fn supports_btc(&self) -> bool {
        self.btc.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.usd.map_or(true, |v| v.is_zero()) && self.btc.map_or(true, |v| v.is_zero())
    }

    pub fn supported_currencies(&self) -> Vec<&'static str> {
        let mut currencies = Vec::new();
        if self.usd.is_some() {
            currencies.push("USD");
        }
        if self.btc.is_some() {
            currencies.push("BTC");
        }
        currencies
    }

    pub fn credit_usd(&mut self, amount: MinorUnits<Usd>) -> Result<(), CurrencyBagError> {
        match &mut self.usd {
            Some(balance) => {
                *balance += amount;
                Ok(())
            }
            None => Err(CurrencyBagError::CurrencyNotSupported("USD")),
        }
    }

    pub fn credit_btc(&mut self, amount: MinorUnits<Btc>) -> Result<(), CurrencyBagError> {
        match &mut self.btc {
            Some(balance) => {
                *balance += amount;
                Ok(())
            }
            None => Err(CurrencyBagError::CurrencyNotSupported("BTC")),
        }
    }

    pub fn debit_usd(&mut self, amount: MinorUnits<Usd>) -> Result<(), CurrencyBagError> {
        match &mut self.usd {
            Some(balance) if *balance >= amount => {
                *balance -= amount;
                Ok(())
            }
            Some(_) => Err(CurrencyBagError::InsufficientBalance("USD")),
            None => Err(CurrencyBagError::CurrencyNotSupported("USD")),
        }
    }

    pub fn debit_btc(&mut self, amount: MinorUnits<Btc>) -> Result<(), CurrencyBagError> {
        match &mut self.btc {
            Some(balance) if *balance >= amount => {
                *balance -= amount;
                Ok(())
            }
            Some(_) => Err(CurrencyBagError::InsufficientBalance("BTC")),
            None => Err(CurrencyBagError::CurrencyNotSupported("BTC")),
        }
    }
}

impl Default for CurrencyBag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "sqlx")]
mod currency_bag_sqlx {
    use sqlx::{Type, postgres::*};

    use super::CurrencyBag;

    impl Type<Postgres> for CurrencyBag {
        fn type_info() -> PgTypeInfo {
            <serde_json::Value as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <serde_json::Value as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CurrencyBag {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let json = serde_json::to_value(self)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)?;
            <serde_json::Value as sqlx::Encode<'_, Postgres>>::encode(json, buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CurrencyBag {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let json = <serde_json::Value as sqlx::Decode<Postgres>>::decode(value)?;
            serde_json::from_value(json)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)
        }
    }

    impl PgHasArrayType for CurrencyBag {
        fn array_type_info() -> PgTypeInfo {
            <serde_json::Value as PgHasArrayType>::array_type_info()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bag_has_no_currencies() {
        let bag = CurrencyBag::new();
        assert!(bag.usd().is_none());
        assert!(bag.btc().is_none());
        assert!(bag.is_empty());
        assert!(bag.supported_currencies().is_empty());
    }

    #[test]
    fn with_usd_creates_zero_usd_slot() {
        let bag = CurrencyBag::new().with_usd();
        assert_eq!(bag.usd(), Some(MinorUnits::ZERO));
        assert!(bag.btc().is_none());
        assert!(bag.is_empty()); // zero is still empty
        assert_eq!(bag.supported_currencies(), vec!["USD"]);
    }

    #[test]
    fn with_both_currencies() {
        let bag = CurrencyBag::new().with_usd().with_btc();
        assert!(bag.supports_usd());
        assert!(bag.supports_btc());
        assert_eq!(bag.supported_currencies(), vec!["USD", "BTC"]);
    }

    #[test]
    fn credit_and_debit_usd() {
        let mut bag = CurrencyBag::new().with_usd();
        bag.credit_usd(MinorUnits::from(500_00u64)).unwrap();
        assert_eq!(bag.usd(), Some(MinorUnits::from(500_00u64)));
        assert!(!bag.is_empty());

        bag.debit_usd(MinorUnits::from(200_00u64)).unwrap();
        assert_eq!(bag.usd(), Some(MinorUnits::from(300_00u64)));
    }

    #[test]
    fn credit_unsupported_currency_fails() {
        let mut bag = CurrencyBag::new().with_usd();
        let result = bag.credit_btc(MinorUnits::from(100u64));
        assert!(matches!(
            result,
            Err(CurrencyBagError::CurrencyNotSupported("BTC"))
        ));
    }

    #[test]
    fn debit_insufficient_balance_fails() {
        let mut bag = CurrencyBag::new().with_usd();
        bag.credit_usd(MinorUnits::from(100u64)).unwrap();
        let result = bag.debit_usd(MinorUnits::from(200u64));
        assert!(matches!(
            result,
            Err(CurrencyBagError::InsufficientBalance("USD"))
        ));
    }

    #[test]
    fn with_amount_constructors() {
        let bag = CurrencyBag::new()
            .with_usd_amount(MinorUnits::from(500_00u64))
            .with_btc_amount(MinorUnits::from(100_000u64));
        assert_eq!(bag.usd(), Some(MinorUnits::from(500_00u64)));
        assert_eq!(bag.btc(), Some(MinorUnits::from(100_000u64)));
        assert!(!bag.is_empty());
    }

    #[test]
    fn none_vs_zero_distinction() {
        let usd_only = CurrencyBag::new().with_usd();
        let both = CurrencyBag::new().with_usd().with_btc();

        // USD-only: BTC is None (not supported)
        assert!(usd_only.btc().is_none());
        assert!(!usd_only.supports_btc());

        // Both: BTC is Some(0) (supported but empty)
        assert_eq!(both.btc(), Some(MinorUnits::ZERO));
        assert!(both.supports_btc());
    }
}
