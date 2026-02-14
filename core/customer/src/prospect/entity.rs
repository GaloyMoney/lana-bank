use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::error::ProspectError;
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
        #[serde(default)]
        inbox_id: String,
    },
    KycApproved {
        level: KycLevel,
        #[serde(default)]
        inbox_id: String,
    },
    KycPending {
        #[serde(default)]
        inbox_id: String,
    },
    KycDeclined {
        #[serde(default)]
        inbox_id: String,
    },
    ManuallyConverted {},
    VerificationLinkCreated {
        url: String,
    },
    Closed {},
    TelegramHandleUpdated {
        telegram_handle: String,
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
    pub status: ProspectStatus,
    #[builder(default)]
    pub kyc_status: KycStatus,
    #[builder(setter(strip_option, into), default)]
    pub applicant_id: Option<String>,
    #[builder(setter(strip_option, into), default)]
    pub verification_link: Option<String>,
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

    pub fn start_kyc(&mut self, applicant_id: String, inbox_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycStarted { .. }
        );
        self.events.push(ProspectEvent::KycStarted {
            applicant_id: applicant_id.clone(),
            inbox_id,
        });
        self.applicant_id = Some(applicant_id);
        self.kyc_status = KycStatus::Started;
        Idempotent::Executed(())
    }

    pub fn set_kyc_pending(&mut self, inbox_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycPending { .. }
        );
        self.events.push(ProspectEvent::KycPending { inbox_id });
        self.kyc_status = KycStatus::Pending;
        Idempotent::Executed(())
    }

    pub fn approve_kyc(
        &mut self,
        level: KycLevel,
        inbox_id: String,
    ) -> Result<Idempotent<NewCustomer>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycApproved { .. }
        );
        let applicant_id = self
            .applicant_id
            .clone()
            .ok_or(ProspectError::KycNotStarted)?;
        self.events
            .push(ProspectEvent::KycApproved { level, inbox_id });
        self.level = level;
        self.kyc_status = KycStatus::Approved;
        self.status = ProspectStatus::Converted;

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

        Ok(Idempotent::Executed(new_customer))
    }

    pub fn convert_manually(&mut self) -> Idempotent<NewCustomer> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::ManuallyConverted { .. } | ProspectEvent::KycApproved { .. }
        );
        self.events.push(ProspectEvent::ManuallyConverted {});
        self.status = ProspectStatus::Converted;
        self.kyc_status = KycStatus::Approved;

        let new_customer = NewCustomer::builder()
            .id(CustomerId::from(self.id))
            .email(self.email.clone())
            .telegram_handle(self.telegram_handle.clone())
            .customer_type(self.customer_type)
            .public_id(self.public_id.clone())
            .applicant_id("manual-conversion")
            .level(KycLevel::Basic)
            .activity(Activity::Active)
            .build()
            .expect("Could not build customer from prospect");

        Idempotent::Executed(new_customer)
    }

    pub fn decline_kyc(&mut self, inbox_id: String) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycDeclined { .. }
        );
        if self.applicant_id.is_none() {
            return Err(ProspectError::KycNotStarted);
        }
        self.events.push(ProspectEvent::KycDeclined { inbox_id });
        self.kyc_status = KycStatus::Declined;
        Ok(Idempotent::Executed(()))
    }

    pub fn close(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::Closed { .. }
                | ProspectEvent::KycApproved { .. }
                | ProspectEvent::ManuallyConverted { .. }
        );
        self.events.push(ProspectEvent::Closed {});
        self.status = ProspectStatus::Closed;
        Idempotent::Executed(())
    }

    pub fn record_verification_link_created(&mut self, url: String) -> Idempotent<()> {
        self.events
            .push(ProspectEvent::VerificationLinkCreated { url: url.clone() });
        self.verification_link = Some(url);
        Idempotent::Executed(())
    }

    pub fn stage(&self) -> ProspectStage {
        match self.status {
            ProspectStatus::Closed => ProspectStage::Closed,
            ProspectStatus::Converted => ProspectStage::Converted,
            ProspectStatus::Open => match self.kyc_status {
                KycStatus::Declined => ProspectStage::KycDeclined,
                KycStatus::Pending => ProspectStage::KycPending,
                KycStatus::Started => ProspectStage::KycStarted,
                KycStatus::NotStarted | KycStatus::Approved => ProspectStage::New,
            },
        }
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
                        .kyc_status(KycStatus::Started);
                }
                ProspectEvent::KycPending { .. } => {
                    builder = builder.kyc_status(KycStatus::Pending);
                }
                ProspectEvent::KycApproved { level, .. } => {
                    builder = builder
                        .level(*level)
                        .kyc_status(KycStatus::Approved)
                        .status(ProspectStatus::Converted);
                }
                ProspectEvent::ManuallyConverted { .. } => {
                    builder = builder
                        .kyc_status(KycStatus::Approved)
                        .status(ProspectStatus::Converted);
                }
                ProspectEvent::VerificationLinkCreated { url, .. } => {
                    builder = builder.verification_link(url.clone());
                }
                ProspectEvent::KycDeclined { .. } => {
                    builder = builder.kyc_status(KycStatus::Declined);
                }
                ProspectEvent::Closed { .. } => {
                    builder = builder.status(ProspectStatus::Closed);
                }
                ProspectEvent::TelegramHandleUpdated {
                    telegram_handle, ..
                } => {
                    builder = builder.telegram_handle(telegram_handle.clone());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_prospect() -> Prospect {
        let id = ProspectId::new();
        let events = EntityEvents::init(
            id,
            [ProspectEvent::Initialized {
                id,
                email: "test@example.com".to_string(),
                telegram_handle: "test_handle".to_string(),
                customer_type: CustomerType::Individual,
                public_id: PublicId::new("test-public-id"),
            }],
        );
        Prospect::try_from_events(events).expect("Failed to create prospect")
    }

    #[test]
    fn approve_kyc_fails_when_kyc_not_started() {
        let mut prospect = create_test_prospect();

        let result = prospect.approve_kyc(KycLevel::Basic, "callback-1".to_string());

        assert!(matches!(result, Err(ProspectError::KycNotStarted)));
    }

    #[test]
    fn approve_kyc_succeeds_after_kyc_started() {
        let mut prospect = create_test_prospect();
        let _ = prospect.start_kyc("correct-applicant-id".to_string(), "callback-1".to_string());

        let result = prospect.approve_kyc(KycLevel::Basic, "callback-2".to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn decline_kyc_fails_when_kyc_not_started() {
        let mut prospect = create_test_prospect();

        let result = prospect.decline_kyc("callback-1".to_string());

        assert!(matches!(result, Err(ProspectError::KycNotStarted)));
    }

    #[test]
    fn decline_kyc_succeeds_after_kyc_started() {
        let mut prospect = create_test_prospect();
        let _ = prospect.start_kyc("correct-applicant-id".to_string(), "callback-1".to_string());

        let result = prospect.decline_kyc("callback-2".to_string());

        assert!(result.is_ok());
    }
}
