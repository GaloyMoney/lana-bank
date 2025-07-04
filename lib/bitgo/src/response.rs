use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub id: String,
    pub label: String,
    pub receive_address: Address,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Address {
    pub address: String,
}
