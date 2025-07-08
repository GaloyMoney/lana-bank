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
