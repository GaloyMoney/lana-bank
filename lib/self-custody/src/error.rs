use bitcoin::NetworkKind;

use crate::config::SelfCustodyNetwork;

#[derive(thiserror::Error, Debug)]
pub enum SelfCustodyError {
    #[error("SelfCustodyError - InvalidXpub: {0}")]
    InvalidXpub(String),
    #[error(
        "SelfCustodyError - XpubNetworkMismatch: xpub network {actual:?} does not match selected network {expected}"
    )]
    XpubNetworkMismatch {
        expected: SelfCustodyNetwork,
        actual: NetworkKind,
    },
    #[error("SelfCustodyError - Http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("SelfCustodyError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("SelfCustodyError - UrlParse: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("SelfCustodyError - Bip32: {0}")]
    Bip32(#[from] bitcoin::bip32::Error),
    #[error("SelfCustodyError - InvalidDerivedPublicKey")]
    InvalidDerivedPublicKey,
    #[error("SelfCustodyError - InvalidEsploraBalance")]
    InvalidEsploraBalance,
}
