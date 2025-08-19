use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
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
        activity: AccountActivity,
        public_id: PublicId,
        audit_info: AuditInfo,
    },
    KycStarted {
        applicant_id: String,
        audit_info: AuditInfo,
    },
    KycApproved {
        applicant_id: String,
        level: KycLevel,
        audit_info: AuditInfo,
    },
    KycDeclined {
        applicant_id: String,
        audit_info: AuditInfo,
    },
    StatusUpdated {
        status: CustomerStatus,
        audit_info: AuditInfo,
    },
    TelegramIdUpdated {
        telegram_id: String,
        audit_info: AuditInfo,
    },
    EmailUpdated {
        email: String,
        audit_info: AuditInfo,
    },
    AccountActivityUpdated {
        activity: AccountActivity,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Customer {
    pub id: CustomerId,
    pub email: String,
    pub telegram_id: String,
    #[builder(default)]
    pub status: CustomerStatus,
    #[builder(default)]
    pub activity: AccountActivity,
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

    pub fn start_kyc(&mut self, applicant_id: String, audit_info: AuditInfo) {
        self.events.push(CustomerEvent::KycStarted {
            applicant_id: applicant_id.clone(),
            audit_info,
        });
        self.applicant_id = Some(applicant_id);
    }

    pub fn approve_kyc(
        &mut self,
        level: KycLevel,
        applicant_id: String,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::KycApproved { .. },
            => CustomerEvent::KycDeclined { .. }
        );
        self.events.push(CustomerEvent::KycApproved {
            level,
            applicant_id: applicant_id.clone(),
            audit_info: audit_info.clone(),
        });

        self.applicant_id = Some(applicant_id);
        self.level = KycLevel::Basic;

        self.update_account_status(CustomerStatus::Active, audit_info)
    }

    pub fn decline_kyc(&mut self, applicant_id: String, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::KycDeclined { .. },
            => CustomerEvent::KycApproved { .. }
        );
        self.events.push(CustomerEvent::KycDeclined {
            applicant_id,
            audit_info: audit_info.clone(),
        });
        self.level = KycLevel::NotKyced;
        self.update_account_status(CustomerStatus::Inactive, audit_info)
    }

    fn update_account_status(
        &mut self,
        status: CustomerStatus,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::StatusUpdated { status: existing_status, .. } if existing_status == &status
        );
        self.events
            .push(CustomerEvent::StatusUpdated { status, audit_info });
        self.status = status;
        Idempotent::Executed(())
    }

    pub(crate) fn update_account_activity(
        &mut self,
        activity: AccountActivity,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::AccountActivityUpdated { activity: existing_activity, .. } if existing_activity == &activity
        );
        self.events.push(CustomerEvent::AccountActivityUpdated {
            activity,
            audit_info,
        });
        self.activity = activity;
        Idempotent::Executed(())
    }

    pub fn update_telegram_id(
        &mut self,
        new_telegram_id: String,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::TelegramIdUpdated { telegram_id: existing_telegram_id , ..} if existing_telegram_id == &new_telegram_id
        );
        self.events.push(CustomerEvent::TelegramIdUpdated {
            telegram_id: new_telegram_id.clone(),
            audit_info,
        });
        self.telegram_id = new_telegram_id;
        Idempotent::Executed(())
    }

    pub fn update_email(&mut self, new_email: String, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::EmailUpdated { email: existing_email, .. } if existing_email == &new_email,
            => CustomerEvent::EmailUpdated { .. }
        );
        self.events.push(CustomerEvent::EmailUpdated {
            email: new_email.clone(),
            audit_info,
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .email(email.clone())
                        .telegram_id(telegram_id.clone())
                        .customer_type(*customer_type)
                        .public_id(public_id.clone())
                        .activity(*activity)
                        .level(KycLevel::NotKyced);
                }
                CustomerEvent::KycStarted { applicant_id, .. } => {
                    builder = builder.applicant_id(applicant_id.clone());
                }
                CustomerEvent::KycApproved {
                    level,
                    applicant_id,
                    ..
                } => builder = builder.applicant_id(applicant_id.clone()).level(*level),
                CustomerEvent::KycDeclined { applicant_id, .. } => {
                    builder = builder.applicant_id(applicant_id.clone())
                }
                CustomerEvent::StatusUpdated { status, .. } => {
                    builder = builder.status(*status);
                }
                CustomerEvent::TelegramIdUpdated { telegram_id, .. } => {
                    builder = builder.telegram_id(telegram_id.clone());
                }
                CustomerEvent::EmailUpdated { email, .. } => {
                    builder = builder.email(email.clone());
                }
                CustomerEvent::AccountActivityUpdated { activity, .. } => {
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
    pub(super) id: CustomerId,
    #[builder(setter(into))]
    pub(super) email: String,
    #[builder(setter(into))]
    pub(super) telegram_id: String,
    #[builder(setter(into))]
    pub(super) customer_type: CustomerType,
    #[builder(setter(skip), default)]
    pub(super) status: CustomerStatus,
    #[builder(setter(skip), default)]
    pub(super) activity: AccountActivity,
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
    pub(super) audit_info: AuditInfo,
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
                audit_info: self.audit_info,
            }],
        )
    }
}
