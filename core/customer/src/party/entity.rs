use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PartyId")]
pub enum PartyEvent {
    Initialized {
        id: PartyId,
        email: String,
        telegram_handle: String,
        customer_type: CustomerType,
    },
    EmailUpdated {
        email: String,
    },
    TelegramHandleUpdated {
        telegram_handle: String,
    },
    PersonalInfoUpdated {
        personal_info: PersonalInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Party {
    pub id: PartyId,
    pub email: String,
    pub telegram_handle: String,
    pub customer_type: CustomerType,
    #[builder(setter(strip_option), default)]
    pub personal_info: Option<PersonalInfo>,
    events: EntityEvents<PartyEvent>,
}

impl core::fmt::Display for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Party: {}", self.id)
    }
}

impl Party {
    pub fn update_email(&mut self, new_email: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            PartyEvent::EmailUpdated { email: existing_email, .. } if existing_email == &new_email,
            => PartyEvent::EmailUpdated { .. }
        );
        self.events.push(PartyEvent::EmailUpdated {
            email: new_email.clone(),
        });
        self.email = new_email;
        Idempotent::Executed(())
    }

    pub fn update_telegram_handle(&mut self, new_telegram_handle: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            PartyEvent::TelegramHandleUpdated { telegram_handle: existing, .. } if existing == &new_telegram_handle
        );
        self.events.push(PartyEvent::TelegramHandleUpdated {
            telegram_handle: new_telegram_handle.clone(),
        });
        self.telegram_handle = new_telegram_handle;
        Idempotent::Executed(())
    }

    pub fn update_personal_info(
        &mut self,
        personal_info: PersonalInfo,
    ) -> Idempotent<PersonalInfo> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            PartyEvent::PersonalInfoUpdated { personal_info: existing, .. } if existing == &personal_info
        );
        self.events.push(PartyEvent::PersonalInfoUpdated {
            personal_info: personal_info.clone(),
        });
        self.personal_info = Some(personal_info.clone());
        Idempotent::Executed(personal_info)
    }
}

impl TryFromEvents<PartyEvent> for Party {
    fn try_from_events(events: EntityEvents<PartyEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PartyBuilder::default();

        for event in events.iter_all() {
            match event {
                PartyEvent::Initialized {
                    id,
                    email,
                    telegram_handle,
                    customer_type,
                } => {
                    builder = builder
                        .id(*id)
                        .email(email.clone())
                        .telegram_handle(telegram_handle.clone())
                        .customer_type(*customer_type);
                }
                PartyEvent::EmailUpdated { email } => {
                    builder = builder.email(email.clone());
                }
                PartyEvent::TelegramHandleUpdated { telegram_handle } => {
                    builder = builder.telegram_handle(telegram_handle.clone());
                }
                PartyEvent::PersonalInfoUpdated { personal_info } => {
                    builder = builder.personal_info(personal_info.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewParty {
    #[builder(setter(into))]
    pub(super) id: PartyId,
    #[builder(setter(into))]
    pub(super) email: String,
    #[builder(setter(into))]
    pub(super) telegram_handle: String,
    #[builder(setter(into))]
    pub(super) customer_type: CustomerType,
}

impl NewParty {
    pub fn builder() -> NewPartyBuilder {
        NewPartyBuilder::default()
    }
}

impl IntoEvents<PartyEvent> for NewParty {
    fn into_events(self) -> EntityEvents<PartyEvent> {
        EntityEvents::init(
            self.id,
            [PartyEvent::Initialized {
                id: self.id,
                email: self.email,
                telegram_handle: self.telegram_handle,
                customer_type: self.customer_type,
            }],
        )
    }
}
