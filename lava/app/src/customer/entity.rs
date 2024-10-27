use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{audit::AuditInfo, ledger::customer::CustomerLedgerAccountIds, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustomerId")]
pub enum CustomerEvent {
    Initialized {
        id: CustomerId,
        email: String,
        telegram_id: String,
        account_ids: CustomerLedgerAccountIds,
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
    TelegramIdUpdated {
        telegram_id: String,
        audit_info: AuditInfo,
    },
}

impl CustomerEvent {
    fn audit_info(&self) -> &AuditInfo {
        match self {
            CustomerEvent::Initialized { audit_info, .. } => audit_info,
            CustomerEvent::KycStarted { audit_info, .. } => audit_info,
            CustomerEvent::KycApproved { audit_info, .. } => audit_info,
            CustomerEvent::KycDeclined { audit_info, .. } => audit_info,
            CustomerEvent::TelegramIdUpdated { audit_info, .. } => audit_info,
        }
    }
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Customer {
    pub id: CustomerId,
    pub email: String,
    pub telegram_id: String,
    pub account_ids: CustomerLedgerAccountIds,
    pub status: AccountStatus,
    pub level: KycLevel,
    #[builder(setter(strip_option, into), default)]
    pub applicant_id: Option<String>,
    pub(super) events: EntityEvents<CustomerEvent>,
}

impl core::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User: {}, email: {}", self.id, self.email)
    }
}

impl Customer {
    pub fn may_create_loan(&self) -> bool {
        true
    }

    pub fn audit_info(&self) -> Vec<AuditInfo> {
        self.events
            .iter_persisted()
            .map(|e| e.event.audit_info().clone())
            .collect()
    }

    pub fn start_kyc(&mut self, applicant_id: String, audit_info: AuditInfo) {
        self.events.push(CustomerEvent::KycStarted {
            applicant_id: applicant_id.clone(),
            audit_info,
        });
        self.applicant_id = Some(applicant_id);
    }

    pub fn approve_kyc(&mut self, level: KycLevel, applicant_id: String, audit_info: AuditInfo) {
        self.events.push(CustomerEvent::KycApproved {
            level,
            applicant_id: applicant_id.clone(),
            audit_info,
        });

        self.applicant_id = Some(applicant_id);
        self.level = KycLevel::Basic;
        self.status = AccountStatus::Active;
    }

    pub fn deactivate(&mut self, applicant_id: String, audit_info: AuditInfo) {
        self.events.push(CustomerEvent::KycDeclined {
            applicant_id,
            audit_info,
        });
        self.level = KycLevel::NotKyced;
        self.status = AccountStatus::Inactive;
    }

    pub fn update_telegram_id(&mut self, new_telegram_id: String, audit_info: AuditInfo) {
        self.events.push(CustomerEvent::TelegramIdUpdated {
            telegram_id: new_telegram_id.clone(),
            audit_info,
        });
        self.telegram_id = new_telegram_id;
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
                    account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_ids(*account_ids)
                        .email(email.clone())
                        .telegram_id(telegram_id.clone())
                        .account_ids(*account_ids)
                        .level(KycLevel::NotKyced)
                        .status(AccountStatus::Inactive);
                }
                CustomerEvent::KycStarted { applicant_id, .. } => {
                    builder = builder.applicant_id(applicant_id.clone());
                }
                CustomerEvent::KycApproved {
                    level,
                    applicant_id,
                    ..
                } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .level(*level)
                        .status(AccountStatus::Active);
                }
                CustomerEvent::KycDeclined { applicant_id, .. } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .status(AccountStatus::Inactive);
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
    pub(super) account_ids: CustomerLedgerAccountIds,
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
                account_ids: self.account_ids,
                audit_info: self.audit_info,
            }],
        )
    }
}
