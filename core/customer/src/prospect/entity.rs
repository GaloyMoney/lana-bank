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
        #[serde(default)]
        party_id: Option<PartyId>,
        #[serde(default)]
        email: Option<String>,
        #[serde(default)]
        telegram_handle: Option<String>,
        #[serde(default)]
        customer_type: Option<CustomerType>,
        public_id: PublicId,
        stage: ProspectStage,
    },
    PartyLinked {
        party_id: PartyId,
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
    // Legacy event variants kept for backward compatibility deserialization
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
    pub party_id: PartyId,
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
    events: EntityEvents<ProspectEvent>,
}

impl core::fmt::Display for Prospect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prospect: {}", self.id)
    }
}

impl Prospect {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn verification_link_created_at(&self) -> Option<DateTime<Utc>> {
        self.events.iter_persisted().rev().find_map(|e| {
            matches!(e.event, ProspectEvent::VerificationLinkCreated { .. })
                .then_some(e.recorded_at)
        })
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
        applicant_id: &str,
        level: KycLevel,
    ) -> Result<Idempotent<NewCustomer>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycApproved { .. }
        );
        self.ensure_open()?;
        let stored_id = self
            .applicant_id
            .as_deref()
            .ok_or(ProspectError::KycNotStarted)?;
        if stored_id != applicant_id {
            return Err(ProspectError::ApplicantIdMismatch {
                expected: self.applicant_id.clone(),
                actual: applicant_id.to_string(),
            });
        }
        let applicant_id = applicant_id.to_string();
        self.level = level;
        self.kyc_status = KycStatus::Approved;
        self.status = ProspectStatus::Converted;
        let stage = self.compute_stage();
        self.events
            .push(ProspectEvent::KycApproved { level, stage });
        self.stage = stage;

        let new_customer = NewCustomer::builder()
            .id(CustomerId::from(self.id))
            .party_id(self.party_id)
            .public_id(self.public_id.clone())
            .applicant_id(applicant_id)
            .kyc_verification(KycVerification::Verified)
            .level(level)
            .activity(Activity::Active)
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

        let new_customer = NewCustomer::builder()
            .id(CustomerId::from(self.id))
            .party_id(self.party_id)
            .public_id(self.public_id.clone())
            .applicant_id("manual-conversion")
            .kyc_verification(KycVerification::NoKyc)
            .level(KycLevel::Basic)
            .activity(Activity::Active)
            .build()
            .expect("Could not build customer from prospect");

        Ok(Idempotent::Executed(new_customer))
    }

    pub fn decline_kyc(&mut self, applicant_id: &str) -> Result<Idempotent<()>, ProspectError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ProspectEvent::KycDeclined { .. }
        );
        self.ensure_open()?;
        let stored_id = self
            .applicant_id
            .as_deref()
            .ok_or(ProspectError::KycNotStarted)?;
        if stored_id != applicant_id {
            return Err(ProspectError::ApplicantIdMismatch {
                expected: self.applicant_id.clone(),
                actual: applicant_id.to_string(),
            });
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
}

impl TryFromEvents<ProspectEvent> for Prospect {
    fn try_from_events(events: EntityEvents<ProspectEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ProspectBuilder::default();

        for event in events.iter_all() {
            match event {
                ProspectEvent::Initialized {
                    id,
                    party_id,
                    public_id,
                    stage,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .public_id(public_id.clone())
                        .level(KycLevel::NotKyced)
                        .stage(*stage);
                    if let Some(party_id) = party_id {
                        builder = builder.party_id(*party_id);
                    }
                }
                ProspectEvent::PartyLinked { party_id } => {
                    builder = builder.party_id(*party_id);
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
                // Legacy event variants - no-op for state reconstruction
                ProspectEvent::TelegramHandleUpdated { .. }
                | ProspectEvent::PersonalInfoUpdated { .. } => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewProspect {
    #[builder(setter(into))]
    pub(super) id: ProspectId,
    pub(super) party_id: PartyId,
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
                party_id: Some(self.party_id),
                email: None,
                telegram_handle: None,
                customer_type: None,
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
        let party_id = PartyId::new();
        let events = EntityEvents::init(
            id,
            [ProspectEvent::Initialized {
                id,
                party_id: Some(party_id),
                email: None,
                telegram_handle: None,
                customer_type: None,
                public_id: PublicId::new("test-public-id"),
                stage: ProspectStage::New,
            }],
        );
        Prospect::try_from_events(events).expect("Failed to create prospect")
    }

    #[test]
    fn approve_kyc_fails_when_kyc_not_started() {
        let mut prospect = create_test_prospect();

        let result = prospect.approve_kyc("some-id", KycLevel::Basic);

        assert!(matches!(result, Err(ProspectError::KycNotStarted)));
    }

    #[test]
    fn approve_kyc_succeeds_after_kyc_started() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("correct-applicant-id".to_string())
            .expect("start_kyc should succeed");

        let result = prospect.approve_kyc("correct-applicant-id", KycLevel::Basic);

        assert!(result.is_ok());
    }

    #[test]
    fn approve_kyc_fails_with_wrong_applicant_id() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("correct-applicant-id".to_string())
            .expect("start_kyc should succeed");

        let result = prospect.approve_kyc("wrong-applicant-id", KycLevel::Basic);

        assert!(matches!(
            result,
            Err(ProspectError::ApplicantIdMismatch { .. })
        ));
    }

    #[test]
    fn decline_kyc_fails_when_kyc_not_started() {
        let mut prospect = create_test_prospect();

        let result = prospect.decline_kyc("some-id");

        assert!(matches!(result, Err(ProspectError::KycNotStarted)));
    }

    #[test]
    fn decline_kyc_succeeds_after_kyc_started() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("correct-applicant-id".to_string())
            .expect("start_kyc should succeed");

        let result = prospect.decline_kyc("correct-applicant-id");

        assert!(result.is_ok());
    }

    #[test]
    fn decline_kyc_fails_with_wrong_applicant_id() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("correct-applicant-id".to_string())
            .expect("start_kyc should succeed");

        let result = prospect.decline_kyc("wrong-applicant-id");

        assert!(matches!(
            result,
            Err(ProspectError::ApplicantIdMismatch { .. })
        ));
    }

    #[test]
    fn close_fails_when_already_converted() {
        let mut prospect = create_test_prospect();
        let _ = prospect
            .start_kyc("applicant-id".to_string())
            .expect("start_kyc should succeed");
        let _ = prospect
            .approve_kyc("applicant-id", KycLevel::Basic)
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
