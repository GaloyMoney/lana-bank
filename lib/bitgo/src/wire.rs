use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Address {
    pub address: String,
    pub index: u64,
}
