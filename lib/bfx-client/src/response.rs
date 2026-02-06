use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct BfxErrorResponse {
    pub event: String,
    pub code: u32,
    pub description: String,
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
}
