use reqwest::Client;
use wire::Address;

mod wire;

pub struct BitgoClient {
    http_client: Client,
    access_token: String,
    coin: Coin,
    wallet_id: String,
}

impl BitgoClient {
    pub async fn init(
        access_token: String,
        ip_restrict: String,
        label: String,
        enterprise_id: String,
        wallet_id: String,
    ) -> Self {
        let http_client = Client::new();

        let access_token = todo!();
        let wallet_id = todo!();

        Self {
            http_client,
            access_token,
            coin: Coin::Testnet4Bitcoin,
            wallet_id,
        }
    }

    pub async fn create_address(&self) -> Result<(Address, serde_json::Value), reqwest::Error> {
        let json = self
            .http_client
            .post(format!(
                "/api/v2/{}/wallet/{}/address",
                self.coin, self.wallet_id
            ))
            .bearer_auth(&self.access_token)
            .json(&serde_json::Value::Object(Default::default()))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok((serde_json::from_value(json.clone()).unwrap(), json))
    }
}

pub enum Coin {
    Bitcoin,
    Testnet4Bitcoin,
}

impl core::fmt::Display for Coin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Coin::Bitcoin => write!(f, "btc"),
            Coin::Testnet4Bitcoin => write!(f, "tbtc4"),
        }
    }
}
