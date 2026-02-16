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
        stage: ProspectStage,
    },
    KycStarted {
        applicant_id: String,
        stage: ProspectStage,
    },
    KycApproved {
        level: KycLevel,
        stage: ProspectStage,
    },
    KycPending {
        stage: ProspectStage,
    },
    KycDeclined {
        stage: ProspectStage,
    },
    ManuallyConverted {
        stage: ProspectStage,
    },
    VerificationLinkCreated {
        url: String,
    },
    Closed {
        stage: ProspectStage,
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
    pub stage: ProspectStage,
    pub public_id: PublicId,
    #[builder(setter(strip_option), default)]
    pub personal_info: Option<PersonalInfo>,
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

    fn ensure_open(&self) -> Result<(), ProspectError> {
        match self.status {
            ProspectStatus::Converted => Err(ProspectError::AlreadyConverted),
            ProspectStatus::Closed => Err(ProspectError::AlreadyClosed),
            ProspectStatus::Open => Ok(()),
        }
    }

    fn compute_stage(&self) -> ProspectStage {
        match self.status {
            ProspectStatus::Closed => ProspectStage::Closed,
            ProspectStatus::Converted => ProspectStage::Converted,
            ProspectStatus::Open => match self.kyc_status {
                KycStatus::Declined => ProspectStage::KycDeclined,
                KycStatus::Pending => ProspectStage::KycPending,
                KycStatus::Started => ProspectStage::KycStarted,
                KycStatus::NotStarted => ProspectStage::New,
                KycStatus::Approved => {
                    tracing::error!(
                        prospect_id = %self.id,
                        "prospect has KycStatus::Approved but ProspectStatus::Open - expected Converted"
                    );
                    ProspectStage::New
                }
            },
        }
    }

    pub fn start_kyc(&mut self, applicant_id: String) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycStarted { .. }
        );
        self.ensure_open()?;
        self.applicant_id = Some(applicant_id.clone());
        self.kyc_status = KycStatus::Started;
        let stage = self.compute_stage();
        self.events.push(ProspectEvent::KycStarted {
            applicant_id,
            stage,
        });
        self.stage = stage;
        Ok(Idempotent::Executed(()))
    }

    pub fn set_kyc_pending(&mut self) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycPending { .. }
        );
        self.ensure_open()?;
        self.kyc_status = KycStatus::Pending;
        let stage = self.compute_stage();
        self.events.push(ProspectEvent::KycPending { stage });
        self.stage = stage;
        Ok(Idempotent::Executed(()))
    }

    pub fn approve_kyc(
        &mut self,
        level: KycLevel,
        personal_info: PersonalInfo,
    ) -> Result<Idempotent<NewCustomer>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycApproved { .. }
        );
        self.ensure_open()?;
        let applicant_id = self
            .applicant_id
            .clone()
            .ok_or(ProspectError::KycNotStarted)?;
        self.level = level;
        self.kyc_status = KycStatus::Approved;
        self.status = ProspectStatus::Converted;
        let stage = self.compute_stage();
        self.events
            .push(ProspectEvent::KycApproved { level, stage });
        self.stage = stage;

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
            .personal_info(personal_info)
            .build()
            .expect("Could not build customer from prospect");

        Ok(Idempotent::Executed(new_customer))
    }

    pub fn convert_manually(&mut self) -> Result<Idempotent<NewCustomer>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::ManuallyConverted { .. }
        );
        self.ensure_open()?;
        self.status = ProspectStatus::Converted;
        self.kyc_status = KycStatus::Approved;
        let stage = self.compute_stage();
        self.events.push(ProspectEvent::ManuallyConverted { stage });
        self.stage = stage;

        let personal_info = self
            .personal_info
            .clone()
            .unwrap_or_else(PersonalInfo::dummy);

        let new_customer = NewCustomer::builder()
            .id(CustomerId::from(self.id))
            .email(self.email.clone())
            .telegram_handle(self.telegram_handle.clone())
            .customer_type(self.customer_type)
            .public_id(self.public_id.clone())
            .applicant_id("manual-conversion")
            .kyc_verification(KycVerification::NoKyc)
            .level(KycLevel::Basic)
            .activity(Activity::Active)
            .personal_info(personal_info)
            .build()
            .expect("Could not build customer from prospect");

        Ok(Idempotent::Executed(new_customer))
    }

    pub fn decline_kyc(&mut self) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycDeclined { .. }
        );
        self.ensure_open()?;
        if self.applicant_id.is_none() {
            return Err(ProspectError::KycNotStarted);
        }
        self.kyc_status = KycStatus::Declined;
        let stage = self.compute_stage();
        self.events.push(ProspectEvent::KycDeclined { stage });
        self.stage = stage;
        Ok(Idempotent::Executed(()))
    }

    pub fn close(&mut self) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(self.events.iter_all().rev(), ProspectEvent::Closed { .. });
        self.ensure_open()?;
        self.status = ProspectStatus::Closed;
        let stage = self.compute_stage();
        self.events.push(ProspectEvent::Closed { stage });
        self.stage = stage;
        Ok(Idempotent::Executed(()))
    }

    pub fn record_verification_link_created(
        &mut self,
        url: String,
    ) -> Result<Idempotent<()>, ProspectError> {
        self.ensure_open()?;
        self.events
            .push(ProspectEvent::VerificationLinkCreated { url: url.clone() });
        self.verification_link = Some(url);
        Ok(Idempotent::Executed(()))
    }

    pub fn update_personal_info(
        &mut self,
        personal_info: PersonalInfo,
    ) -> Idempotent<PersonalInfo> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::PersonalInfoUpdated { personal_info: existing, .. } if existing == &personal_info
        );
        self.events.push(ProspectEvent::PersonalInfoUpdated {
            personal_info: personal_info.clone(),
        });
        self.personal_info = Some(personal_info.clone());
        Idempotent::Executed(personal_info)
    }

    pub fn update_telegram_handle(
        &mut self,
        new_telegram_handle: String,
    ) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::TelegramHandleUpdated { telegram_handle: existing_telegram_handle , ..} if existing_telegram_handle == &new_telegram_handle
        );
        self.ensure_open()?;
        self.events.push(ProspectEvent::TelegramHandleUpdated {
            telegram_handle: new_telegram_handle.clone(),
        });
        self.telegram_handle = new_telegram_handle;
        Ok(Idempotent::Executed(()))
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
                    stage,
                } => {
                    builder = builder
                        .id(*id)
                        .email(email.clone())
                        .telegram_handle(telegram_handle.clone())
                        .customer_type(*customer_type)
                        .public_id(public_id.clone())
                        .level(KycLevel::NotKyced)
                        .stage(*stage);
                }
                ProspectEvent::KycStarted {
                    applicant_id,
                    stage,
                } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .kyc_status(KycStatus::Started)
                        .stage(*stage);
                }
                ProspectEvent::KycPending { stage } => {
                    builder = builder.kyc_status(KycStatus::Pending).stage(*stage);
                }
                ProspectEvent::KycApproved { level, stage } => {
                    builder = builder
                        .level(*level)
                        .kyc_status(KycStatus::Approved)
                        .status(ProspectStatus::Converted)
                        .stage(*stage);
                }
                ProspectEvent::ManuallyConverted { stage } => {
                    builder = builder
                        .kyc_status(KycStatus::Approved)
                        .status(ProspectStatus::Converted)
                        .stage(*stage);
                }
                ProspectEvent::VerificationLinkCreated { url } => {
                    builder = builder.verification_link(url.clone());
                }
                ProspectEvent::KycDeclined { stage } => {
                    builder = builder.kyc_status(KycStatus::Declined).stage(*stage);
                }
                ProspectEvent::Closed { stage } => {
                    builder = builder.status(ProspectStatus::Closed).stage(*stage);
                }
                ProspectEvent::TelegramHandleUpdated { telegram_handle } => {
                    builder = builder.telegram_handle(telegram_handle.clone());
                }
                ProspectEvent::PersonalInfoUpdated { personal_info } => {
                    builder = builder.personal_info(personal_info.clone());
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
                stage: ProspectStage::New,
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
                stage: ProspectStage::New,
            }],
        );
        Prospect::try_from_events(events).expect("Failed to create prospect")
    }

    #[test]
    fn approve_kyc_fails_when_kyc_not_started() {
        let mut prospect = create_test_prospect();

        let result = prospect.approve_kyc(KycLevel::Basic, PersonalInfo::dummy());

        assert!(matches!(result, Err(ProspectError::KycNotStarted)));
    }

    #[test]
    fn approve_kyc_succeeds_after_kyc_started() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("correct-applicant-id".to_string())
            .expect("start_kyc should succeed");

        let result = prospect.approve_kyc(KycLevel::Basic, PersonalInfo::dummy());

        assert!(result.is_ok());
    }

    #[test]
    fn decline_kyc_fails_when_kyc_not_started() {
        let mut prospect = create_test_prospect();

        let result = prospect.decline_kyc();

        assert!(matches!(result, Err(ProspectError::KycNotStarted)));
    }

    #[test]
    fn decline_kyc_succeeds_after_kyc_started() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("correct-applicant-id".to_string())
            .expect("start_kyc should succeed");

        let result = prospect.decline_kyc();

        assert!(result.is_ok());
    }

    #[test]
    fn close_fails_when_already_converted() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("applicant-id".to_string())
            .expect("start_kyc should succeed");
        let _ = prospect
            .approve_kyc(KycLevel::Basic, PersonalInfo::dummy())
            .expect("approve_kyc should succeed");

        let result = prospect.close();

        assert!(matches!(result, Err(ProspectError::AlreadyConverted)));
    }

    #[test]
    fn start_kyc_fails_when_closed() {
        let mut prospect = create_test_prospect();
        let _ = prospect.close().expect("close should succeed");

        let result = prospect.start_kyc("applicant-id".to_string());

        assert!(matches!(result, Err(ProspectError::AlreadyClosed)));
    }
}
