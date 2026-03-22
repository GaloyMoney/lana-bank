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
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("custodianId".to_string())
)]
pub struct Custodian {
    custodian_id: CustodianId,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCustodian>,
}

impl From<DomainCustodian> for Custodian {
    fn from(custodian: DomainCustodian) -> Self {
        Self {
            custodian_id: custodian.id,
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

    async fn provider(&self) -> &str {
        &self.entity.provider
    }
}

#[derive(InputObject)]
pub struct KomainuConfigInput {
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
pub struct BitgoConfigInput {
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
pub struct SelfCustodyConfigInput {
    name: String,
    #[graphql(secret)]
    account_xpub: String,
    network: SelfCustodyNetwork,
}

#[derive(async_graphql::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfCustodyNetwork {
    Testnet3,
    Testnet4,
    Signet,
    Mainnet,
}

#[derive(InputObject)]
pub struct ManualConfigInput {
    name: String,
}

impl From<ManualConfigInput> for DomainCustodianConfig {
    fn from(_config: ManualConfigInput) -> Self {
        DomainCustodianConfig::Manual
    }
}

impl From<KomainuConfigInput> for DomainKomainuConfig {
    fn from(config: KomainuConfigInput) -> Self {
        Self {
            api_key: config.api_key,
            api_secret: config.api_secret,
            testing_instance: config.testing_instance,
            secret_key: config.secret_key,
            webhook_secret: config.webhook_secret,
        }
    }
}

impl From<BitgoConfigInput> for DomainBitgoConfig {
    fn from(config: BitgoConfigInput) -> Self {
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

impl From<SelfCustodyNetwork> for DomainSelfCustodyNetwork {
    fn from(network: SelfCustodyNetwork) -> Self {
        match network {
            SelfCustodyNetwork::Testnet3 => DomainSelfCustodyNetwork::Testnet3,
            SelfCustodyNetwork::Testnet4 => DomainSelfCustodyNetwork::Testnet4,
            SelfCustodyNetwork::Signet => DomainSelfCustodyNetwork::Signet,
            SelfCustodyNetwork::Mainnet => DomainSelfCustodyNetwork::Mainnet,
        }
    }
}

impl From<SelfCustodyConfigInput> for DomainSelfCustodyConfig {
    fn from(config: SelfCustodyConfigInput) -> Self {
        Self {
            account_xpub: config.account_xpub,
            network: config.network.into(),
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianCreateInput {
    Komainu(KomainuConfigInput),
    Bitgo(BitgoConfigInput),
    SelfCustody(SelfCustodyConfigInput),
    Manual(ManualConfigInput),
}

impl CustodianCreateInput {
    pub fn name(&self) -> &str {
        match self {
            CustodianCreateInput::Komainu(conf) => &conf.name,
            CustodianCreateInput::Bitgo(conf) => &conf.name,
            CustodianCreateInput::SelfCustody(conf) => &conf.name,
            CustodianCreateInput::Manual(conf) => &conf.name,
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
            CustodianCreateInput::Manual(..) => DomainCustodianConfig::Manual,
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianConfigInput {
    Komainu(KomainuConfigInput),
    Bitgo(BitgoConfigInput),
    SelfCustody(SelfCustodyConfigInput),
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
    pub custodian_id: CustodianId,
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
