use async_graphql::*;
use url::Url;

use es_entity::Sort;

use crate::{graphql::primitives::SortDirection, primitives::*};

pub use lana_app::custody::{
    CustodiansSortBy as DomainCustodiansSortBy,
    custodian::{
        BitgoConfig as DomainBitgoConfig, Custodian as DomainCustodian,
        CustodianConfig as DomainCustodianConfig, KomainuConfig as DomainKomainuConfig,
        SelfCustodyConfig as DomainSelfCustodyConfig,
        SelfCustodyNetwork as DomainSelfCustodyNetwork,
    },
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Custodian {
    id: ID,
    custodian_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCustodian>,
}

impl From<DomainCustodian> for Custodian {
    fn from(custodian: DomainCustodian) -> Self {
        Self {
            id: custodian.id.to_global_id(),
            custodian_id: custodian.id.into(),
            created_at: custodian.created_at().into(),
            entity: Arc::new(custodian),
        }
    }
}

#[ComplexObject]
impl Custodian {
    async fn name(&self) -> &str {
        &self.entity.name
    }
}

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

#[derive(InputObject)]
pub struct SelfCustodyConfig {
    name: String,
    #[graphql(secret)]
    xpub: String,
    network: SelfCustodyNetwork,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfCustodyNetwork {
    Mainnet,
    Testnet,
    Signet,
}

impl From<SelfCustodyNetwork> for DomainSelfCustodyNetwork {
    fn from(network: SelfCustodyNetwork) -> Self {
        match network {
            SelfCustodyNetwork::Mainnet => DomainSelfCustodyNetwork::Mainnet,
            SelfCustodyNetwork::Testnet => DomainSelfCustodyNetwork::Testnet,
            SelfCustodyNetwork::Signet => DomainSelfCustodyNetwork::Signet,
        }
    }
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

impl From<SelfCustodyConfig> for DomainSelfCustodyConfig {
    fn from(config: SelfCustodyConfig) -> Self {
        Self {
            xpub: config.xpub,
            network: config.network.into(),
            next_derivation_index: 0,
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianCreateInput {
    Komainu(KomainuConfig),
    Bitgo(BitgoConfig),
    SelfCustody(SelfCustodyConfig),
}

impl CustodianCreateInput {
    pub fn name(&self) -> &str {
        match self {
            CustodianCreateInput::Komainu(conf) => &conf.name,
            CustodianCreateInput::Bitgo(conf) => &conf.name,
            CustodianCreateInput::SelfCustody(conf) => &conf.name,
        }
    }
}

impl From<CustodianCreateInput> for DomainCustodianConfig {
    fn from(input: CustodianCreateInput) -> Self {
        match input {
            CustodianCreateInput::Komainu(config) => DomainCustodianConfig::Komainu(config.into()),
            CustodianCreateInput::Bitgo(config) => DomainCustodianConfig::Bitgo(config.into()),
            CustodianCreateInput::SelfCustody(config) => {
                DomainCustodianConfig::SelfCustody(config.into())
            }
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianConfigInput {
    Komainu(KomainuConfig),
    Bitgo(BitgoConfig),
    SelfCustody(SelfCustodyConfig),
}

impl From<CustodianConfigInput> for DomainCustodianConfig {
    fn from(input: CustodianConfigInput) -> Self {
        match input {
            CustodianConfigInput::Komainu(config) => DomainCustodianConfig::Komainu(config.into()),
            CustodianConfigInput::Bitgo(config) => DomainCustodianConfig::Bitgo(config.into()),
            CustodianConfigInput::SelfCustody(config) => {
                DomainCustodianConfig::SelfCustody(config.into())
            }
        }
    }
}

#[derive(InputObject)]
pub struct CustodianConfigUpdateInput {
    pub custodian_id: UUID,
    pub config: CustodianConfigInput,
}

crate::mutation_payload! { CustodianCreatePayload, custodian: Custodian }

crate::mutation_payload! { CustodianConfigUpdatePayload, custodian: Custodian }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustodiansSortBy {
    Name,
    #[default]
    CreatedAt,
}

impl From<CustodiansSortBy> for DomainCustodiansSortBy {
    fn from(by: CustodiansSortBy) -> Self {
        match by {
            CustodiansSortBy::Name => DomainCustodiansSortBy::Name,
            CustodiansSortBy::CreatedAt => DomainCustodiansSortBy::CreatedAt,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct CustodiansSort {
    #[graphql(default)]
    pub by: CustodiansSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<CustodiansSort> for Sort<DomainCustodiansSortBy> {
    fn from(sort: CustodiansSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<CustodiansSort> for DomainCustodiansSortBy {
    fn from(sort: CustodiansSort) -> Self {
        sort.by.into()
    }
}
