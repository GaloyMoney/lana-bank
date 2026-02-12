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
        email: String,
        telegram_id: String,
        customer_type: CustomerType,
        activity: Activity,
        public_id: PublicId,
        #[serde(default)]
        applicant_id: Option<String>,
        #[serde(default)]
        level: KycLevel,
        #[serde(default)]
        kyc_verification: KycVerification,
    },
    KycVerificationUpdated {
        kyc_verification: KycVerification,
    },
    TelegramIdUpdated {
        telegram_id: String,
    },
    EmailUpdated {
        email: String,
    },
    ActivityUpdated {
        activity: Activity,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Customer {
    pub id: CustomerId,
    pub email: String,
    pub telegram_id: String,
    #[builder(default)]
    pub kyc_verification: KycVerification,
    #[builder(default)]
    pub activity: Activity,
    pub level: KycLevel,
    pub customer_type: CustomerType,
    #[builder(setter(strip_option, into), default)]
    pub applicant_id: Option<String>,
    pub public_id: PublicId,
    events: EntityEvents<CustomerEvent>,
}

impl core::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Customer: {}, email: {}, customer_type: {}",
            self.id, self.email, self.customer_type
        )
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
        self.applicant_id.is_some()
    }

    pub(crate) fn update_account_kyc_verification(
        &mut self,
        kyc_verification: KycVerification,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::KycVerificationUpdated { kyc_verification: existing_kyc_verification, .. } if existing_kyc_verification == &kyc_verification,
            => CustomerEvent::KycVerificationUpdated { .. }
        );
        self.events
            .push(CustomerEvent::KycVerificationUpdated { kyc_verification });
        self.kyc_verification = kyc_verification;
        Idempotent::Executed(())
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

    pub fn update_telegram_id(&mut self, new_telegram_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::TelegramIdUpdated { telegram_id: existing_telegram_id , ..} if existing_telegram_id == &new_telegram_id
        );
        self.events.push(CustomerEvent::TelegramIdUpdated {
            telegram_id: new_telegram_id.clone(),
        });
        self.telegram_id = new_telegram_id;
        Idempotent::Executed(())
    }

    pub fn update_email(&mut self, new_email: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::EmailUpdated { email: existing_email, .. } if existing_email == &new_email,
            => CustomerEvent::EmailUpdated { .. }
        );
        self.events.push(CustomerEvent::EmailUpdated {
            email: new_email.clone(),
        });
        self.email = new_email;
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
                    email,
                    telegram_id,
                    customer_type,
                    public_id,
                    activity,
                    applicant_id,
                    level,
                    kyc_verification,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .email(email.clone())
                        .telegram_id(telegram_id.clone())
                        .customer_type(*customer_type)
                        .public_id(public_id.clone())
                        .activity(*activity)
                        .level(*level)
                        .kyc_verification(*kyc_verification);
                    if let Some(applicant_id) = applicant_id {
                        builder = builder.applicant_id(applicant_id.clone());
                    }
                }
                CustomerEvent::KycVerificationUpdated {
                    kyc_verification, ..
                } => {
                    builder = builder.kyc_verification(*kyc_verification);
                }
                CustomerEvent::TelegramIdUpdated { telegram_id, .. } => {
                    builder = builder.telegram_id(telegram_id.clone());
                }
                CustomerEvent::EmailUpdated { email, .. } => {
                    builder = builder.email(email.clone());
                }
                CustomerEvent::ActivityUpdated { activity, .. } => {
                    builder = builder.activity(*activity);
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCustomer {
    #[builder(setter(into))]
    pub(crate) id: CustomerId,
    #[builder(setter(into))]
    pub(crate) email: String,
    #[builder(setter(into))]
    pub(crate) telegram_id: String,
    #[builder(setter(into))]
    pub(crate) customer_type: CustomerType,
    #[builder(default)]
    pub(crate) kyc_verification: KycVerification,
    #[builder(default)]
    pub(crate) activity: Activity,
    #[builder(setter(into))]
    pub(crate) public_id: PublicId,
    #[builder(setter(strip_option, into), default)]
    pub(crate) applicant_id: Option<String>,
    #[builder(default)]
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
                email: self.email,
                telegram_id: self.telegram_id,
                customer_type: self.customer_type,
                activity: self.activity,
                public_id: self.public_id,
                applicant_id: self.applicant_id,
                level: self.level,
                kyc_verification: self.kyc_verification,
            }],
        )
    }
}
