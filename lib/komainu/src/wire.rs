use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct Wallet {
    pub name: String,
    pub address: String,
    pub asset: String,
    pub status: String,
}

#[derive(Clone, Serialize)]
pub struct GetToken {
    pub api_user: String,
    pub api_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct GetTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct Many<T> {
    pub count: u64,
    pub data: Vec<T>,
    pub has_next: bool,
    pub page: u64,
}
