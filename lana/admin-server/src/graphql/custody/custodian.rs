use async_graphql::*;

use crate::primitives::*;

pub use core_custody::{
    Custodian as DomainCustodian, CustodianConfig as DomainCustodianConfig,
    KomainuConfig as DomainKomainuConfig,
};
pub use lana_app::custody::custodian::CustodiansByNameCursor;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Custodian {
    id: ID,
    custodian_config_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCustodian>,
}

impl From<DomainCustodian> for Custodian {
    fn from(custodian_config: DomainCustodian) -> Self {
        Self {
            id: custodian_config.id.to_global_id(),
            custodian_config_id: custodian_config.id.into(),
            created_at: custodian_config.created_at().into(),
            entity: Arc::new(custodian_config),
        }
    }
}

#[ComplexObject]
impl Custodian {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn custodian_config(&self) -> CustodianConfig {
        match &self.entity.custodian {
            DomainCustodianConfig::Komainu(_) => CustodianConfig::Komainu,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CustodianConfig {
    Komainu,
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
}

impl From<KomainuConfig> for DomainKomainuConfig {
    fn from(config: KomainuConfig) -> Self {
        Self {
            api_key: config.api_key,
            api_secret: config.api_secret,
            testing_instance: config.testing_instance,
            secret_key: config.secret_key,
        }
    }
}

#[derive(OneofObject)]
pub enum CustodianCreateInput {
    Komainu(KomainuConfig),
}

impl CustodianCreateInput {
    pub fn name(&self) -> &str {
        match self {
            CustodianCreateInput::Komainu(conf) => &conf.name,
        }
    }
}

impl From<CustodianCreateInput> for DomainCustodianConfig {
    fn from(input: CustodianCreateInput) -> Self {
        match input {
            CustodianCreateInput::Komainu(config) => DomainCustodianConfig::Komainu(config.into()),
        }
    }
}

crate::mutation_payload! { CustodianCreatePayload, custodian_config: Custodian }
