use async_graphql::*;

use es_entity::Sort;

use crate::{graphql::primitives::SortDirection, primitives::*};

pub use lana_app::price::{
    PriceProvider as DomainPriceProvider, PriceProviderConfig as DomainPriceProviderConfig,
    PriceProvidersSortBy as DomainPriceProvidersSortBy,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PriceProvider {
    id: ID,
    price_provider_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPriceProvider>,
}

impl From<DomainPriceProvider> for PriceProvider {
    fn from(provider: DomainPriceProvider) -> Self {
        Self {
            id: provider.id.to_global_id(),
            price_provider_id: provider.id.into(),
            created_at: provider.created_at().into(),
            entity: Arc::new(provider),
        }
    }
}

#[ComplexObject]
impl PriceProvider {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn provider(&self) -> &str {
        &self.entity.provider
    }
}

#[derive(InputObject)]
pub struct BitfinexCreateInput {
    name: String,
}

#[derive(OneofObject)]
pub enum PriceProviderCreateInput {
    Bitfinex(BitfinexCreateInput),
}

impl PriceProviderCreateInput {
    pub fn name(&self) -> &str {
        match self {
            PriceProviderCreateInput::Bitfinex(conf) => &conf.name,
        }
    }
}

impl From<PriceProviderCreateInput> for DomainPriceProviderConfig {
    fn from(input: PriceProviderCreateInput) -> Self {
        match input {
            PriceProviderCreateInput::Bitfinex(_) => DomainPriceProviderConfig::Bitfinex,
        }
    }
}

#[derive(OneofObject)]
pub enum PriceProviderConfigInput {
    Bitfinex(BitfinexCreateInput),
}

impl From<PriceProviderConfigInput> for DomainPriceProviderConfig {
    fn from(input: PriceProviderConfigInput) -> Self {
        match input {
            PriceProviderConfigInput::Bitfinex(_) => DomainPriceProviderConfig::Bitfinex,
        }
    }
}

#[derive(InputObject)]
pub struct PriceProviderConfigUpdateInput {
    pub price_provider_id: UUID,
    pub config: PriceProviderConfigInput,
}

crate::mutation_payload! { PriceProviderCreatePayload, price_provider: PriceProvider }

crate::mutation_payload! { PriceProviderConfigUpdatePayload, price_provider: PriceProvider }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PriceProvidersSortBy {
    Name,
    #[default]
    CreatedAt,
}

impl From<PriceProvidersSortBy> for DomainPriceProvidersSortBy {
    fn from(by: PriceProvidersSortBy) -> Self {
        match by {
            PriceProvidersSortBy::Name => DomainPriceProvidersSortBy::Name,
            PriceProvidersSortBy::CreatedAt => DomainPriceProvidersSortBy::CreatedAt,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct PriceProvidersSort {
    #[graphql(default)]
    pub by: PriceProvidersSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<PriceProvidersSort> for Sort<DomainPriceProvidersSortBy> {
    fn from(sort: PriceProvidersSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<PriceProvidersSort> for DomainPriceProvidersSortBy {
    fn from(sort: PriceProvidersSort) -> Self {
        sort.by.into()
    }
}
