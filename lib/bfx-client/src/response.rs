use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct BtcUsdTick {
    pub bid: Decimal,
    pub bid_size: Decimal,
    pub ask: Decimal,
    pub ask_size: Decimal,
    pub daily_change: Decimal,
    pub daily_change_relative: Decimal,
    pub last_price: Decimal,
    pub volume: Decimal,
    pub high: Decimal,
    pub low: Decimal,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BfxErrorResponse {
    pub event: String,
    pub code: u32,
    pub description: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct BfxAuthErrorResponse(pub String, pub u32, pub String);

#[derive(Deserialize, Serialize, Debug)]
pub struct Wallet {
    pub wallet_type: String,
    pub currency: String,
    pub balance: Decimal,
    pub unsettled_interest: Decimal,
    pub available_balance: Decimal,
    pub last_change: Option<String>,
    pub trade_details: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BfxNotification<T> {
    pub mts: i64,
    pub kind: String,
    pub message_id: Option<i64>,
    pub placeholder: Option<Value>,
    pub data: T,
    pub code: Option<i64>,
    pub status: String,
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DepositAddress {
    pub placeholder_0: Option<Value>,
    pub method: String,
    pub currency_code: String,
    pub placeholder_1: Option<Value>,
    pub address: String,
    pub pool_address: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn last_price_data() {
        let response_text =
            "[16808,24.10170847,16809,55.3107456,-26,-0.0015,16809,147.2349813,16884,16769]";
        let details = serde_json::from_str::<BtcUsdTick>(response_text).unwrap();
        assert_eq!(details.last_price, dec!(16809));
    }

    #[test]
    fn wallet_array_deserializes() {
        let response_text = r#"[
          "exchange",
          "BTC",
          1.25,
          0,
          1.25,
          "Deposit 0.25 BTC",
          {"reason": "TRANSFER"}
        ]"#;
        let wallet = serde_json::from_str::<Wallet>(response_text).unwrap();

        assert_eq!(wallet.wallet_type, "exchange");
        assert_eq!(wallet.currency, "BTC");
        assert_eq!(wallet.balance, dec!(1.25));
        assert_eq!(wallet.available_balance, dec!(1.25));
        assert_eq!(wallet.last_change.as_deref(), Some("Deposit 0.25 BTC"));
    }

    #[test]
    fn deposit_address_notification_deserializes() {
        let response_text = r#"[
          1568738594687,
          "acc_dep",
          null,
          null,
          [
            null,
            "BITCOIN",
            "BTC",
            null,
            "bc1qtestaddress",
            null
          ],
          null,
          "SUCCESS",
          "success"
        ]"#;

        let notification =
            serde_json::from_str::<BfxNotification<DepositAddress>>(response_text).unwrap();

        assert_eq!(notification.kind, "acc_dep");
        assert_eq!(notification.status, "SUCCESS");
        assert_eq!(notification.data.method, "BITCOIN");
        assert_eq!(notification.data.currency_code, "BTC");
        assert_eq!(notification.data.address, "bc1qtestaddress");
        assert_eq!(notification.data.pool_address, None);
    }
}
