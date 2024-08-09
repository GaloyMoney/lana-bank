use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{entity::*, ledger::customer::CustomerLedgerAccountIds, primitives::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomerEvent {
    Initialized {
        id: CustomerId,
        email: String,
        account_ids: CustomerLedgerAccountIds,
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
}

impl EntityEvent for CustomerEvent {
    type EntityId = CustomerId;
    fn event_table_name() -> &'static str {
        "customer_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Customer {
    pub id: CustomerId,
    pub email: String,
    pub account_ids: CustomerLedgerAccountIds,
    pub status: AccountStatus,
    pub level: KycLevel,
    #[builder(setter(strip_option, into), default)]
    pub applicant_id: Option<String>,
    pub(super) events: EntityEvents<CustomerEvent>,
}

impl Customer {
    pub fn may_create_loan(&self) -> bool {
        true
    }
}

impl core::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User: {}, email: {}", self.id, self.email)
    }
}

impl Entity for Customer {
    type Event = CustomerEvent;
}

impl Customer {
    pub fn start_kyc(&mut self, applicant_id: String) {
        self.events.push(CustomerEvent::KycStarted {
            applicant_id: applicant_id.clone(),
        });
        self.applicant_id = Some(applicant_id);
    }

    pub fn approve_kyc(&mut self, level: KycLevel, applicant_id: String) {
        self.events.push(CustomerEvent::KycApproved {
            level,
            applicant_id: applicant_id.clone(),
        });

        self.applicant_id = Some(applicant_id);
        self.level = KycLevel::Basic;
        self.status = AccountStatus::Active;
    }

    pub fn deactivate(&mut self, applicant_id: String) {
        self.events
            .push(CustomerEvent::KycDeclined { applicant_id });
        self.level = KycLevel::NotKyced;
        self.status = AccountStatus::Inactive;
    }
}

impl TryFrom<EntityEvents<CustomerEvent>> for Customer {
    type Error = EntityError;

    fn try_from(events: EntityEvents<CustomerEvent>) -> Result<Self, Self::Error> {
        let mut builder = CustomerBuilder::default();
        for event in events.iter() {
            match event {
                CustomerEvent::Initialized {
                    id,
                    email,
                    account_ids,
                } => {
                    builder = builder
                        .id(*id)
                        .account_ids(*account_ids)
                        .email(email.clone())
                        .account_ids(*account_ids)
                        .level(KycLevel::NotKyced)
                        .status(AccountStatus::Inactive);
                }
                CustomerEvent::KycStarted { applicant_id } => {
                    builder = builder.applicant_id(applicant_id.clone());
                }
                CustomerEvent::KycApproved {
                    level,
                    applicant_id,
                } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .level(*level)
                        .status(AccountStatus::Active)
                }
                CustomerEvent::KycDeclined { applicant_id } => {
                    builder = builder
                        .applicant_id(applicant_id.clone())
                        .status(AccountStatus::Inactive)
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
    pub(super) account_ids: CustomerLedgerAccountIds,
}

impl NewCustomer {
    pub fn builder() -> NewCustomerBuilder {
        NewCustomerBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<CustomerEvent> {
        EntityEvents::init(
            self.id,
            [CustomerEvent::Initialized {
                id: self.id,
                email: self.email,
                account_ids: self.account_ids,
            }],
        )
    }
}
