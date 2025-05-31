use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct Wallet {
    pub name: String,
    pub address: String,
    pub asset: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct GetToken<'a> {
    pub api_user: &'a str,
    pub api_secret: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct GetTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct Many<T> {
    count: u64,
    data: Vec<T>,
    has_next: bool,
    page: u64,
}
