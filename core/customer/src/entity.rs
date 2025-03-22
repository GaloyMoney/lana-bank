use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustomerId")]
pub enum CustomerEvent {
    Initialized {
        id: CustomerId,
        email: String,
        telegram_id: String,
        customer_type: CustomerType,
        audit_info: AuditInfo,
    },
    AuthenticationIdUpdated {
        authentication_id: AuthenticationId,
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
    AccountStatusUpdated {
        status: AccountStatus,
        audit_info: AuditInfo,
    },
    TelegramIdUpdated {
        telegram_id: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Customer {
    pub id: CustomerId,
    #[builder(setter(strip_option), default)]
    pub authentication_id: Option<AuthenticationId>,
    pub email: String,
    pub telegram_id: String,
    #[builder(default)]
    pub status: AccountStatus,
    pub level: KycLevel,
    pub customer_type: CustomerType,
    #[builder(setter(strip_option, into), default)]
    pub applicant_id: Option<String>,
    pub(super) events: EntityEvents<CustomerEvent>,
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

    pub fn update_authentication_id(
        &mut self,
        authentication_id: AuthenticationId,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            CustomerEvent::AuthenticationIdUpdated { authentication_id: existing_id } if existing_id == &authentication_id
        );
        self.authentication_id = Some(authentication_id);
        self.events
            .push(CustomerEvent::AuthenticationIdUpdated { authentication_id });
        Idempotent::Executed(())
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

        self.update_account_status(AccountStatus::Active, audit_info)
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
        self.update_account_status(AccountStatus::Inactive, audit_info)
    }

    fn update_account_status(
        &mut self,
        status: AccountStatus,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::AccountStatusUpdated { status: existing_status, .. } if existing_status == &status
        );
        self.events
            .push(CustomerEvent::AccountStatusUpdated { status, audit_info });
        self.status = status;
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .email(email.clone())
                        .telegram_id(telegram_id.clone())
                        .customer_type(*customer_type)
                        .level(KycLevel::NotKyced);
                }
                CustomerEvent::AuthenticationIdUpdated { authentication_id } => {
                    builder = builder.authentication_id(*authentication_id);
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
                CustomerEvent::AccountStatusUpdated { status, .. } => {
                    builder = builder.status(*status);
                }
                CustomerEvent::TelegramIdUpdated { telegram_id, .. } => {
                    builder = builder.telegram_id(telegram_id.clone());
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
    pub(super) status: AccountStatus,
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
                audit_info: self.audit_info,
            }],
        )
    }
}
