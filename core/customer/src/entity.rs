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
        telegram_handle: String,
        customer_type: CustomerType,
        activity: Activity,
        public_id: PublicId,
        applicant_id: String,
        #[serde(default)]
        level: KycLevel,
        #[serde(default)]
        kyc_verification: KycVerification,
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
    KycRejected {
        applicant_id: String,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Customer {
    pub id: CustomerId,
    pub email: String,
    pub telegram_handle: String,
    #[builder(default)]
    pub kyc_verification: KycVerification,
    #[builder(default)]
    pub activity: Activity,
    pub level: KycLevel,
    pub customer_type: CustomerType,
    #[builder(setter(into))]
    pub applicant_id: String,
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
        true
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

    pub fn update_telegram_handle(&mut self, new_telegram_handle: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::TelegramHandleUpdated { telegram_handle: existing_telegram_handle , ..} if existing_telegram_handle == &new_telegram_handle
        );
        self.events.push(CustomerEvent::TelegramHandleUpdated {
            telegram_handle: new_telegram_handle.clone(),
        });
        self.telegram_handle = new_telegram_handle;
        Idempotent::Executed(())
    }

    pub fn reject_kyc(&mut self, applicant_id: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::KycRejected { .. }
        );
        self.events
            .push(CustomerEvent::KycRejected { applicant_id });
        self.kyc_verification = KycVerification::Rejected;
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
                    telegram_handle,
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
                        .telegram_handle(telegram_handle.clone())
                        .customer_type(*customer_type)
                        .public_id(public_id.clone())
                        .activity(*activity)
                        .level(*level)
                        .kyc_verification(*kyc_verification)
                        .applicant_id(applicant_id.clone());
                }
                CustomerEvent::TelegramHandleUpdated {
                    telegram_handle, ..
                } => {
                    builder = builder.telegram_handle(telegram_handle.clone());
                }
                CustomerEvent::EmailUpdated { email, .. } => {
                    builder = builder.email(email.clone());
                }
                CustomerEvent::ActivityUpdated { activity, .. } => {
                    builder = builder.activity(*activity);
                }
                CustomerEvent::KycRejected { .. } => {
                    builder = builder.kyc_verification(KycVerification::Rejected);
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
    pub(crate) telegram_handle: String,
    #[builder(setter(into))]
    pub(crate) customer_type: CustomerType,
    #[builder(default)]
    pub(crate) kyc_verification: KycVerification,
    #[builder(default)]
    pub(crate) activity: Activity,
    #[builder(setter(into))]
    pub(crate) public_id: PublicId,
    #[builder(setter(into))]
    pub(crate) applicant_id: String,
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
                telegram_handle: self.telegram_handle,
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
