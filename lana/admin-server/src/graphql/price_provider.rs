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

    async fn active(&self) -> bool {
        self.entity.active()
    }
}

#[derive(InputObject)]
pub struct BitfinexCreateInput {
    name: String,
}

#[derive(InputObject)]
pub struct ManualPriceCreateInput {
    name: String,
    usd_cents_per_btc: u64,
}

#[derive(InputObject)]
pub struct ManualPriceConfigInput {
    usd_cents_per_btc: u64,
}

#[derive(OneofObject)]
pub enum PriceProviderCreateInput {
    Bitfinex(BitfinexCreateInput),
    ManualPrice(ManualPriceCreateInput),
}

impl PriceProviderCreateInput {
    pub fn name(&self) -> &str {
        match self {
            PriceProviderCreateInput::Bitfinex(conf) => &conf.name,
            PriceProviderCreateInput::ManualPrice(conf) => &conf.name,
        }
    }
}

impl From<PriceProviderCreateInput> for DomainPriceProviderConfig {
    fn from(input: PriceProviderCreateInput) -> Self {
        match input {
            PriceProviderCreateInput::Bitfinex(_) => DomainPriceProviderConfig::Bitfinex,
            PriceProviderCreateInput::ManualPrice(conf) => DomainPriceProviderConfig::ManualPrice {
                usd_cents_per_btc: conf.usd_cents_per_btc,
            },
        }
    }
}

#[derive(OneofObject)]
pub enum PriceProviderConfigInput {
    Bitfinex(BitfinexCreateInput),
    ManualPrice(ManualPriceConfigInput),
}

impl From<PriceProviderConfigInput> for DomainPriceProviderConfig {
    fn from(input: PriceProviderConfigInput) -> Self {
        match input {
            PriceProviderConfigInput::Bitfinex(_) => DomainPriceProviderConfig::Bitfinex,
            PriceProviderConfigInput::ManualPrice(conf) => DomainPriceProviderConfig::ManualPrice {
                usd_cents_per_btc: conf.usd_cents_per_btc,
            },
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

crate::mutation_payload! { PriceProviderActivatePayload, price_provider: PriceProvider }

crate::mutation_payload! { PriceProviderDeactivatePayload, price_provider: PriceProvider }

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
