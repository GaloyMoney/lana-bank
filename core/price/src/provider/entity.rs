use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::primitives::PriceProviderId;

use super::{config::*, error::*};

#[derive(EsEvent, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PriceProviderId")]
pub enum PriceProviderEvent {
    Initialized {
        id: PriceProviderId,
        name: String,
        provider: String,
    },
    ConfigUpdated {
        provider_config: serde_json::Value,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct PriceProvider {
    pub id: PriceProviderId,
    pub(super) provider_config: serde_json::Value,
    pub name: String,
    pub provider: String,
    events: EntityEvents<PriceProviderEvent>,
}

impl PriceProvider {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for PriceProvider")
    }

    pub fn update_config(&mut self, new_config: PriceProviderConfig) -> Idempotent<()> {
        let new_value = serde_json::to_value(&new_config).expect("config serializes");

        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: PriceProviderEvent::ConfigUpdated { provider_config }
                if *provider_config == new_value,
            resets_on: PriceProviderEvent::ConfigUpdated { .. }
        );

        self.provider_config = new_value.clone();

        self.events.push(PriceProviderEvent::ConfigUpdated {
            provider_config: new_value,
        });

        Idempotent::Executed(())
    }

    pub fn config(&self) -> Result<PriceProviderConfig, PriceProviderError> {
        serde_json::from_value(self.provider_config.clone()).map_err(PriceProviderError::Serde)
    }
}

impl TryFromEvents<PriceProviderEvent> for PriceProvider {
    fn try_from_events(
        events: EntityEvents<PriceProviderEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = PriceProviderBuilder::default();

        for event in events.iter_all() {
            match event {
                PriceProviderEvent::Initialized {
                    id, name, provider, ..
                } => {
                    builder = builder
                        .id(*id)
                        .name(name.clone())
                        .provider(provider.clone())
                }
                PriceProviderEvent::ConfigUpdated {
                    provider_config, ..
                } => builder = builder.provider_config(provider_config.clone()),
            }
        }

        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewPriceProvider {
    #[builder(setter(into))]
    pub(super) id: PriceProviderId,
    pub(super) name: String,
    pub(super) provider: String,
    pub(super) provider_config: serde_json::Value,
}

impl NewPriceProvider {
    pub fn builder() -> NewPriceProviderBuilder {
        Default::default()
    }
}

impl IntoEvents<PriceProviderEvent> for NewPriceProvider {
    fn into_events(self) -> EntityEvents<PriceProviderEvent> {
        EntityEvents::init(
            self.id,
            [
                PriceProviderEvent::Initialized {
                    id: self.id,
                    name: self.name,
                    provider: self.provider,
                },
                PriceProviderEvent::ConfigUpdated {
                    provider_config: self.provider_config,
                },
            ],
        )
    }
}
