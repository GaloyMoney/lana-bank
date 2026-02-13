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
    },
    KycApproved {
        applicant_id: String,
        level: KycLevel,
    },
    KycDeclined {
        applicant_id: String,
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
    ) -> Result<Idempotent<NewCustomer>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycApproved { applicant_id: existing, .. } if existing == &applicant_id,
            => ProspectEvent::KycDeclined { .. }
        );
        if self.applicant_id.as_ref() != Some(&applicant_id) {
            return Err(ProspectError::ApplicantIdMismatch {
                expected: self.applicant_id.clone(),
                actual: applicant_id,
            });
        }
        self.events.push(ProspectEvent::KycApproved {
            level,
            applicant_id: applicant_id.clone(),
        });
        self.applicant_id = Some(applicant_id.clone());
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

    pub fn decline_kyc(&mut self, applicant_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycDeclined { .. } | ProspectEvent::KycApproved { .. }
        );
        self.events
            .push(ProspectEvent::KycDeclined { applicant_id });
        self.kyc_status = KycStatus::Declined;
        Idempotent::Executed(())
    }

    pub fn close(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::Closed { .. } | ProspectEvent::KycApproved { .. }
        );
        self.events.push(ProspectEvent::Closed {});
        self.status = ProspectStatus::Closed;
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
                        .kyc_status(KycStatus::Approved)
                        .status(ProspectStatus::Converted);
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
    fn approve_kyc_fails_when_applicant_id_not_set() {
        let mut prospect = create_test_prospect();

        let result = prospect.approve_kyc(KycLevel::Basic, "some-applicant-id".to_string());

        match result {
            Err(ProspectError::ApplicantIdMismatch { expected, actual }) => {
                assert_eq!(expected, None);
                assert_eq!(actual, "some-applicant-id");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    #[test]
    fn approve_kyc_fails_when_applicant_id_mismatch() {
        let mut prospect = create_test_prospect();
        let _ = prospect.start_kyc("correct-applicant-id".to_string());

        let result = prospect.approve_kyc(KycLevel::Basic, "wrong-applicant-id".to_string());

        match result {
            Err(ProspectError::ApplicantIdMismatch { expected, actual }) => {
                assert_eq!(expected, Some("correct-applicant-id".to_string()));
                assert_eq!(actual, "wrong-applicant-id");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    #[test]
    fn approve_kyc_succeeds_when_applicant_id_matches() {
        let mut prospect = create_test_prospect();
        let _ = prospect.start_kyc("correct-applicant-id".to_string());

        let result = prospect.approve_kyc(KycLevel::Basic, "correct-applicant-id".to_string());

        assert!(result.is_ok());
    }
}
