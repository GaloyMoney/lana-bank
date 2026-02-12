use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct Notification {
    pub event_type: EventType,
    pub entity: EntityType,
    pub entity_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    BalanceUpdated,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Wallet,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub name: String,
    pub asset: String,
    pub address: String,
    pub balance: WalletBalance,
    pub workspace: String,
    pub external_reference: String,
    pub account: String,
    pub organization: String,
    pub wallet_category: WalletCategory,
    pub status: WalletStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub id: String,
    pub request_type: RequestType,
    pub status: RequestStatus,
    pub entity: RequestEntity,
    pub entity_id: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub workspace: String,
    pub organization: String,
    pub account: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub wallet_id: String,
    pub direction: TransactionDirection,
    pub asset: String,
    pub amount: Decimal,
    pub fees: Decimal,
    pub created_at: DateTime<Utc>,
    pub transaction_type: String,
    pub status: TransactionStatus,
    pub tx_hash: String,
    pub sender_address: String,
    pub receiver_address: String,
    pub note: String,
    pub created_by: String,
    pub workspace: String,
    pub external_reference: String,
    pub organization: String,
    pub account: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WalletBalance {
    pub total: Decimal,
    pub available: Decimal,
    pub staked: Decimal,
    pub locked: Decimal,
    pub balance_updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum WalletCategory {
    Custody,
    Staking,
    Collateral,
}

#[derive(Clone, Debug, Deserialize)]
pub enum WalletStatus {
    Active,
    Inactive,
    Approved,
    HsmCoinUpdated,
    Pending,
    PendingCreateInHsm,
    PendingViewOnly,
    Rejected,
    ViewOnly,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestType {
    CreateTransaction,
    CollateralOperation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestStatus {
    Created,
    Pending,
    Approved,
    Rejected,
    Cancelled,
    Expired,
    Blocked,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestEntity {
    Transaction,
    Collateral,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionDirection {
    In,
    Out,
    Flat,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Broadcasted,
    Confirmed,
    Failed,
}

#[derive(Clone, Serialize)]
pub(crate) struct GetToken {
    pub api_user: String,
    pub api_secret: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Many<T> {
    pub data: Vec<T>,
    pub has_next: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Fallible<T> {
    Error {
        error_code: String,
        errors: Vec<String>,
        status: u16,
    },
    Ok(T),
}
