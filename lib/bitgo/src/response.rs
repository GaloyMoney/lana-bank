use serde::Deserialize;

use crate::TransferType;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub id: String,
    pub label: String,
    pub receive_address: Address,
    pub confirmed_balance: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Address {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub wallet: String,
    pub txid: String,
    pub confirmations: u32,
    pub value: u64,
    #[serde(rename = "type")]
    pub transfer_type: TransferType,
}
