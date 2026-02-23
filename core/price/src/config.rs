use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PriceConfig {
    #[serde(default = "default_providers")]
    pub providers: Vec<PriceProvider>,
}

impl Default for PriceConfig {
    fn default() -> Self {
        Self {
            providers: default_providers(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceProvider {
    Bitfinex,
}

fn default_providers() -> Vec<PriceProvider> {
    vec![PriceProvider::Bitfinex]
}
