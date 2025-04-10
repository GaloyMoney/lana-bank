use super::error::DepositAccountError;
use sqlx::Type;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Type)]
#[sqlx(transparent)]
pub struct DepositAccountShortCodeId(i64);

impl TryFrom<i64> for DepositAccountShortCodeId {
    type Error = DepositAccountError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(DepositAccountError::NegativeShortCodeId(value));
        }

        if value > 9999999 {
            return Err(DepositAccountError::ShortCodeIdTooLarge(value));
        }

        Ok(Self(value))
    }
}

impl std::fmt::Display for DepositAccountShortCodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DepositAccountShortCodeId {
    type Err = DepositAccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: i64 = s
            .parse()
            .map_err(|_| DepositAccountError::AccountCodeParseError)?;
        inner.try_into()
    }
}
