use serde::Deserialize;

use crate::error::BitfinexError;

#[derive(Debug, Clone)]
pub struct WalletEntry {
    pub wallet_type: String,
    pub currency: String,
    pub balance: f64,
    pub unsettled_interest: f64,
    pub available_balance: f64,
}

impl WalletEntry {
    pub fn from_value(value: &serde_json::Value) -> Result<Self, BitfinexError> {
        let arr = value.as_array().ok_or_else(|| {
            BitfinexError::UnexpectedResponseFormat("wallet entry is not an array".to_string())
        })?;

        if arr.len() < 5 {
            return Err(BitfinexError::UnexpectedResponseFormat(format!(
                "wallet entry has {} elements, expected at least 5",
                arr.len()
            )));
        }

        let wallet_type = arr[0]
            .as_str()
            .ok_or_else(|| {
                BitfinexError::UnexpectedResponseFormat("wallet_type is not a string".to_string())
            })?
            .to_string();

        let currency = arr[1]
            .as_str()
            .ok_or_else(|| {
                BitfinexError::UnexpectedResponseFormat("currency is not a string".to_string())
            })?
            .to_string();

        let balance = arr[2].as_f64().unwrap_or(0.0);
        let unsettled_interest = arr[3].as_f64().unwrap_or(0.0);
        let available_balance = arr[4].as_f64().unwrap_or(0.0);

        Ok(Self {
            wallet_type,
            currency,
            balance,
            unsettled_interest,
            available_balance,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepositAddressResponse {
    pub address: String,
}

impl DepositAddressResponse {
    pub fn from_value(value: &serde_json::Value) -> Result<Self, BitfinexError> {
        // Bitfinex v2 deposit/address response format:
        // [MTS, TYPE, null, null, [null, METHOD, CURRENCY, null, ADDRESS, ...], ...]
        let arr = value.as_array().ok_or_else(|| {
            BitfinexError::UnexpectedResponseFormat(
                "deposit address response is not an array".to_string(),
            )
        })?;

        if arr.len() < 5 {
            return Err(BitfinexError::UnexpectedResponseFormat(format!(
                "deposit address response has {} elements, expected at least 5",
                arr.len()
            )));
        }

        let inner = arr[4].as_array().ok_or_else(|| {
            BitfinexError::UnexpectedResponseFormat(
                "deposit address inner element is not an array".to_string(),
            )
        })?;

        if inner.len() < 5 {
            return Err(BitfinexError::UnexpectedResponseFormat(format!(
                "deposit address inner array has {} elements, expected at least 5",
                inner.len()
            )));
        }

        let address = inner[4]
            .as_str()
            .ok_or_else(|| {
                BitfinexError::UnexpectedResponseFormat(
                    "deposit address is not a string".to_string(),
                )
            })?
            .to_string();

        Ok(Self { address })
    }
}
