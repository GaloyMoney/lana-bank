use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Notification {
    Transfer(TransferNotification),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferNotification {
    pub hash: String,
    pub transfer: String,
    pub wallet: String,
    pub coin: String,
    pub state: TransferState,
    pub transfer_type: TransferType,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub id: String,
    pub label: String,
    pub receive_address: Address,
    pub confirmed_balance: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateWalletResponse {
    pub wallet: Wallet,
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
    pub state: TransferState,
}

#[derive(Clone, Debug, Deserialize)]
pub enum TransferType {
    Receive,
    Send,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransferState {
    Confirmed,
    Failed,
    Initialized,
    PendingApproval,
    Rejected,
    Removed,
    Replaced,
    Signed,
    Unconfirmed,
}
