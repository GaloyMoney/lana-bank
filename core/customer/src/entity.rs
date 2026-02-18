use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustomerId")]
pub enum CustomerEvent {
    Initialized {
        id: CustomerId,
        #[serde(default)]
        party_id: Option<PartyId>,
        #[serde(default)]
        email: Option<String>,
        #[serde(default)]
        telegram_handle: Option<String>,
        #[serde(default)]
        customer_type: Option<CustomerType>,
        activity: Activity,
        public_id: PublicId,
        applicant_id: String,
        level: KycLevel,
        kyc_verification: KycVerification,
        #[serde(default)]
        personal_info: Option<PersonalInfo>,
    },
    PartyLinked {
        party_id: PartyId,
    },
    TelegramHandleUpdated {
        telegram_handle: String,
    },
    EmailUpdated {
        email: String,
    },
    ActivityUpdated {
        activity: Activity,
    },
    KycRejected {},
    PersonalInfoUpdated {
        personal_info: PersonalInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Customer {
    pub id: CustomerId,
    pub party_id: PartyId,
    pub kyc_verification: KycVerification,
    pub activity: Activity,
    pub level: KycLevel,
    #[builder(setter(into))]
    pub applicant_id: String,
    pub public_id: PublicId,
    events: EntityEvents<CustomerEvent>,
}

impl core::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Customer: {}", self.id)
    }
}

impl Customer {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn may_create_loan(&self) -> bool {
        true
    }

    pub fn should_sync_financial_transactions(&self) -> bool {
        self.kyc_verification.is_verified()
    }

    pub(crate) fn update_activity(&mut self, activity: Activity) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::ActivityUpdated { activity: existing_activity, .. } if existing_activity == &activity,
            => CustomerEvent::ActivityUpdated { .. }
        );
        self.events
            .push(CustomerEvent::ActivityUpdated { activity });
        self.activity = activity;
        Idempotent::Executed(())
    }

    pub fn reject_kyc(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::KycRejected { .. }
        );
        self.events.push(CustomerEvent::KycRejected {});
        self.kyc_verification = KycVerification::Rejected;
        Idempotent::Executed(())
    }
}

impl TryFromEvents<CustomerEvent> for Customer {
    fn try_from_events(events: EntityEvents<CustomerEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CustomerBuilder::default();

        for event in events.iter_all() {
            match event {
                CustomerEvent::Initialized {
                    id,
                    party_id,
                    activity,
                    public_id,
                    applicant_id,
                    level,
                    kyc_verification,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .activity(*activity)
                        .public_id(public_id.clone())
                        .level(*level)
                        .kyc_verification(*kyc_verification)
                        .applicant_id(applicant_id.clone());
                    if let Some(party_id) = party_id {
                        builder = builder.party_id(*party_id);
                    }
                }
                CustomerEvent::PartyLinked { party_id } => {
                    builder = builder.party_id(*party_id);
                }
                CustomerEvent::ActivityUpdated { activity, .. } => {
                    builder = builder.activity(*activity);
                }
                CustomerEvent::KycRejected { .. } => {
                    builder = builder.kyc_verification(KycVerification::Rejected);
                }
                // Legacy event variants - no-op for state reconstruction
                CustomerEvent::TelegramHandleUpdated { .. }
                | CustomerEvent::EmailUpdated { .. }
                | CustomerEvent::PersonalInfoUpdated { .. } => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCustomer {
    #[builder(setter(into))]
    pub(crate) id: CustomerId,
    pub(crate) party_id: PartyId,
    pub(crate) kyc_verification: KycVerification,
    pub(crate) activity: Activity,
    #[builder(setter(into))]
    pub(crate) public_id: PublicId,
    #[builder(setter(into))]
    pub(crate) applicant_id: String,
    pub(crate) level: KycLevel,
}

impl NewCustomer {
    pub fn builder() -> NewCustomerBuilder {
        NewCustomerBuilder::default()
    }
}

impl IntoEvents<CustomerEvent> for NewCustomer {
    fn into_events(self) -> EntityEvents<CustomerEvent> {
        EntityEvents::init(
            self.id,
            [CustomerEvent::Initialized {
                id: self.id,
                party_id: Some(self.party_id),
                email: None,
                telegram_handle: None,
                customer_type: None,
                activity: self.activity,
                public_id: self.public_id,
                applicant_id: self.applicant_id,
                level: self.level,
                kyc_verification: self.kyc_verification,
                personal_info: None,
            }],
        )
    }
}
