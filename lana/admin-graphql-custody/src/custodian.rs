use async_graphql::*;
use url::Url;

use crate::primitives::*;

pub use admin_graphql_shared::custody::Custodian;

pub use lana_app::custody::custodian::{
    BitgoConfig as DomainBitgoConfig, Custodian as DomainCustodian,
    CustodianConfig as DomainCustodianConfig, CustodiansByNameCursor,
    KomainuConfig as DomainKomainuConfig,
};

#[derive(InputObject)]
pub struct KomainuConfig {
    name: String,
    api_key: String,
    #[graphql(secret)]
    api_secret: String,
    testing_instance: bool,
    #[graphql(secret)]
    secret_key: String,
    #[graphql(secret)]
    webhook_secret: String,
}

#[derive(InputObject)]
pub struct BitgoConfig {
    name: String,
    #[graphql(secret)]
    long_lived_token: String,
    #[graphql(secret)]
    passphrase: String,
    testing_instance: bool,
    enterprise_id: String,
    webhook_url: String,
    #[graphql(secret)]
    webhook_secret: String,
}

impl From<KomainuConfig> for DomainKomainuConfig {
    fn from(config: KomainuConfig) -> Self {
        Self {
            api_key: config.api_key,
            api_secret: config.api_secret,
            testing_instance: config.testing_instance,
            secret_key: config.secret_key,
            webhook_secret: config.webhook_secret,
        }
    }
}

impl From<BitgoConfig> for DomainBitgoConfig {
    fn from(config: BitgoConfig) -> Self {
        Self {
            long_lived_token: config.long_lived_token,
            passphrase: config.passphrase,
            testing_instance: config.testing_instance,
            enterprise_id: config.enterprise_id,
            webhook_url: Url::parse(&config.webhook_url).expect("webhook_url must be a valid URL"),
            webhook_secret: config.webhook_secret,
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianCreateInput {
    Komainu(KomainuConfig),
    Bitgo(BitgoConfig),
}

impl CustodianCreateInput {
    pub fn name(&self) -> &str {
        match self {
            CustodianCreateInput::Komainu(conf) => &conf.name,
            CustodianCreateInput::Bitgo(conf) => &conf.name,
        }
    }
}

impl From<CustodianCreateInput> for DomainCustodianConfig {
    fn from(input: CustodianCreateInput) -> Self {
        match input {
            CustodianCreateInput::Komainu(config) => DomainCustodianConfig::Komainu(config.into()),
            CustodianCreateInput::Bitgo(config) => DomainCustodianConfig::Bitgo(config.into()),
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianConfigInput {
    Komainu(KomainuConfig),
    Bitgo(BitgoConfig),
}

impl From<CustodianConfigInput> for DomainCustodianConfig {
    fn from(input: CustodianConfigInput) -> Self {
        match input {
            CustodianConfigInput::Komainu(config) => DomainCustodianConfig::Komainu(config.into()),
            CustodianConfigInput::Bitgo(config) => DomainCustodianConfig::Bitgo(config.into()),
        }
    }
}

#[derive(InputObject)]
pub struct CustodianConfigUpdateInput {
    pub custodian_id: UUID,
    pub config: CustodianConfigInput,
}

mutation_payload! { CustodianCreatePayload, custodian: Custodian }

mutation_payload! { CustodianConfigUpdatePayload, custodian: Custodian }
