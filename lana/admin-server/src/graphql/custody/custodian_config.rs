use async_graphql::*;

use crate::primitives::*;

pub use core_custody::{
    Custodian as DomainCustodian, CustodianConfig as DomainCustodianConfig,
    KomainuConfig as DomainKomainuConfig,
};
pub use lana_app::custody::custodian_config::CustodianConfigsByNameCursor;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CustodianConfig {
    id: ID,
    custodian_config_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCustodianConfig>,
}

impl From<DomainCustodianConfig> for CustodianConfig {
    fn from(custodian_config: DomainCustodianConfig) -> Self {
        Self {
            id: custodian_config.id.to_global_id(),
            custodian_config_id: custodian_config.id.into(),
            created_at: custodian_config.created_at().into(),
            entity: Arc::new(custodian_config),
        }
    }
}

#[ComplexObject]
impl CustodianConfig {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn custodian(&self) -> Custodian {
        match &self.entity.custodian {
            DomainCustodian::Komainu(_) => Custodian::Komainu,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Custodian {
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
pub enum CustodianConfigCreateInput {
    Komainu(KomainuConfig),
}

impl CustodianConfigCreateInput {
    pub fn name(&self) -> &str {
        match self {
            CustodianConfigCreateInput::Komainu(conf) => &conf.name,
        }
    }
}

impl From<CustodianConfigCreateInput> for DomainCustodian {
    fn from(input: CustodianConfigCreateInput) -> Self {
        match input {
            CustodianConfigCreateInput::Komainu(config) => DomainCustodian::Komainu(config.into()),
        }
    }
}

crate::mutation_payload! { CustodianConfigCreatePayload, custodian_config: CustodianConfig }
