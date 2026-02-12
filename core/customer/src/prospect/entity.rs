use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{entity::NewCustomer, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ProspectId")]
pub enum ProspectEvent {
    Initialized {
        id: ProspectId,
        email: String,
        telegram_handle: String,
        customer_type: CustomerType,
        public_id: PublicId,
    },
    KycStarted {
        applicant_id: String,
    },
    KycApproved {
        applicant_id: String,
        level: KycLevel,
    },
    KycDeclined {
        applicant_id: String,
    },
    TelegramHandleUpdated {
        telegram_handle: String,
    },
    EmailUpdated {
        email: String,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Prospect {
    pub id: ProspectId,
    pub email: String,
    pub telegram_handle: String,
    pub customer_type: CustomerType,
    #[builder(default)]
    pub kyc_status: KycStatus,
    #[builder(setter(strip_option, into), default)]
    pub applicant_id: Option<String>,
    #[builder(default)]
    pub level: KycLevel,
    pub public_id: PublicId,
    events: EntityEvents<ProspectEvent>,
}

impl core::fmt::Display for Prospect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Prospect: {}, email: {}, customer_type: {}",
            self.id, self.email, self.customer_type
        )
    }
}

impl Prospect {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn start_kyc(&mut self, applicant_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycStarted { .. }
        );
        self.events.push(ProspectEvent::KycStarted {
            applicant_id: applicant_id.clone(),
        });
        self.applicant_id = Some(applicant_id);
        self.kyc_status = KycStatus::Pending;
        Idempotent::Executed(())
    }

    pub fn approve_kyc(
        &mut self,
        level: KycLevel,
        applicant_id: String,
    ) -> Idempotent<NewCustomer> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycApproved { .. },
            => ProspectEvent::KycDeclined { .. }
        );
        self.events.push(ProspectEvent::KycApproved {
            level,
            applicant_id: applicant_id.clone(),
        });
        self.applicant_id = Some(applicant_id.clone());
        self.level = level;
        self.kyc_status = KycStatus::Approved;

        let new_customer = NewCustomer::builder()
            .id(CustomerId::from(self.id))
            .email(self.email.clone())
            .telegram_handle(self.telegram_handle.clone())
            .customer_type(self.customer_type)
            .public_id(self.public_id.clone())
            .applicant_id(applicant_id)
            .kyc_verification(KycVerification::Verified)
            .level(level)
            .activity(Activity::Active)
            .build()
            .expect("Could not build customer from prospect");

        Idempotent::Executed(new_customer)
    }

    pub fn decline_kyc(&mut self, applicant_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycDeclined { .. },
            => ProspectEvent::KycApproved { .. }
        );
        self.events
            .push(ProspectEvent::KycDeclined { applicant_id });
        self.kyc_status = KycStatus::Declined;
        Idempotent::Executed(())
    }

    pub fn update_telegram_handle(&mut self, new_telegram_handle: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::TelegramHandleUpdated { telegram_handle: existing_telegram_handle , ..} if existing_telegram_handle == &new_telegram_handle
        );
        self.events.push(ProspectEvent::TelegramHandleUpdated {
            telegram_handle: new_telegram_handle.clone(),
        });
        self.telegram_handle = new_telegram_handle;
        Idempotent::Executed(())
    }

    pub fn update_email(&mut self, new_email: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::EmailUpdated { email: existing_email, .. } if existing_email == &new_email,
            => ProspectEvent::EmailUpdated { .. }
        );
        self.events.push(ProspectEvent::EmailUpdated {
            email: new_email.clone(),
        });
        self.email = new_email;
        Idempotent::Executed(())
    }
}

impl TryFromEvents<ProspectEvent> for Prospect {
    fn try_from_events(events: EntityEvents<ProspectEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ProspectBuilder::default();

        for event in events.iter_all() {
            match event {
                ProspectEvent::Initialized {
                    id,
                    email,
                    telegram_handle,
                    customer_type,
                    public_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .email(email.clone())
                        .telegram_handle(telegram_handle.clone())
                        .customer_type(*customer_type)
                        .public_id(public_id.clone())
                        .level(KycLevel::NotKyced);
                }
                ProspectEvent::KycStarted { applicant_id, .. } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .kyc_status(KycStatus::Pending);
                }
                ProspectEvent::KycApproved {
                    level,
                    applicant_id,
                    ..
                } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .level(*level)
                        .kyc_status(KycStatus::Approved);
                }
                ProspectEvent::KycDeclined { .. } => {
                    builder = builder.kyc_status(KycStatus::Declined);
                }
                ProspectEvent::TelegramHandleUpdated {
                    telegram_handle, ..
                } => {
                    builder = builder.telegram_handle(telegram_handle.clone());
                }
                ProspectEvent::EmailUpdated { email, .. } => {
                    builder = builder.email(email.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewProspect {
    #[builder(setter(into))]
    pub(super) id: ProspectId,
    #[builder(setter(into))]
    pub(super) email: String,
    #[builder(setter(into))]
    pub(super) telegram_handle: String,
    #[builder(setter(into))]
    pub(super) customer_type: CustomerType,
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
}

impl NewProspect {
    pub fn builder() -> NewProspectBuilder {
        NewProspectBuilder::default()
    }
}

impl IntoEvents<ProspectEvent> for NewProspect {
    fn into_events(self) -> EntityEvents<ProspectEvent> {
        EntityEvents::init(
            self.id,
            [ProspectEvent::Initialized {
                id: self.id,
                email: self.email,
                telegram_handle: self.telegram_handle,
                customer_type: self.customer_type,
                public_id: self.public_id,
            }],
        )
    }
}
