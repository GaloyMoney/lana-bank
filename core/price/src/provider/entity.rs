use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::primitives::PriceProviderId;

use super::config::*;

#[derive(EsEvent, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PriceProviderId")]
pub enum PriceProviderEvent {
    Initialized {
        id: PriceProviderId,
        name: String,
    },
    ConfigUpdated {
        provider_config: PriceProviderConfig,
    },
    Activated {},
    Deactivated {},
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct PriceProvider {
    pub id: PriceProviderId,
    pub(super) provider_config: PriceProviderConfig,
    pub name: String,
    pub provider: String,
    pub(super) active: bool,
    events: EntityEvents<PriceProviderEvent>,
}

impl PriceProvider {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for PriceProvider")
    }

    pub fn update_config(&mut self, new_config: PriceProviderConfig) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: PriceProviderEvent::ConfigUpdated { provider_config }
                if *provider_config == new_config,
            resets_on: PriceProviderEvent::ConfigUpdated { .. }
        );

        self.provider = PriceProviderConfigDiscriminants::from(&new_config).to_string();
        self.provider_config = new_config.clone();

        self.events.push(PriceProviderEvent::ConfigUpdated {
            provider_config: new_config,
        });

        Idempotent::Executed(())
    }

    pub fn config(&self) -> &PriceProviderConfig {
        &self.provider_config
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: PriceProviderEvent::Activated {},
            resets_on: PriceProviderEvent::Deactivated { .. }
        );

        self.active = true;
        self.events.push(PriceProviderEvent::Activated {});
        Idempotent::Executed(())
    }

    pub fn deactivate(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: PriceProviderEvent::Deactivated {},
            resets_on: PriceProviderEvent::Activated { .. }
        );

        self.active = false;
        self.events.push(PriceProviderEvent::Deactivated {});
        Idempotent::Executed(())
    }
}

impl TryFromEvents<PriceProviderEvent> for PriceProvider {
    fn try_from_events(
        events: EntityEvents<PriceProviderEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = PriceProviderBuilder::default();

        for event in events.iter_all() {
            match event {
                PriceProviderEvent::Initialized { id, name, .. } => {
                    builder = builder.id(*id).name(name.clone()).active(true)
                }
                PriceProviderEvent::ConfigUpdated {
                    provider_config, ..
                } => {
                    builder = builder
                        .provider(
                            PriceProviderConfigDiscriminants::from(provider_config).to_string(),
                        )
                        .provider_config(provider_config.clone())
                }
                PriceProviderEvent::Activated {} => builder = builder.active(true),
                PriceProviderEvent::Deactivated {} => builder = builder.active(false),
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
    #[builder(setter(custom))]
    pub(super) provider: String,
    #[builder(setter(custom))]
    pub(super) config: PriceProviderConfig,
}

impl NewPriceProviderBuilder {
    pub fn config(&mut self, config: PriceProviderConfig) -> &mut Self {
        self.provider = Some(PriceProviderConfigDiscriminants::from(&config).to_string());
        self.config = Some(config);
        self
    }
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
                },
                PriceProviderEvent::ConfigUpdated {
                    provider_config: self.config,
                },
            ],
        )
    }
}
