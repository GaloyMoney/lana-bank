use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use audit::AuditInfo;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositAccountId")]
pub enum DepositAccountEvent {
    Initialized {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
        ledger_account_id: CalaAccountId,
        reference: String,
        name: String,
        description: String,
        status: AccountStatus,
        audit_info: AuditInfo,
    },
    AccountStatusUpdated {
        status: AccountStatus,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DepositAccount {
    pub id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
    pub reference: String,
    pub name: String,
    pub description: String,
    pub status: AccountStatus,

    events: EntityEvents<DepositAccountEvent>,
}

impl DepositAccount {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("Deposit Account has never been persisted")
    }

    pub fn update_account_status(
        &mut self,
        status: AccountStatus,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::AccountStatusUpdated { status: existing_status, .. } if existing_status == &status
        );
        self.events
            .push(DepositAccountEvent::AccountStatusUpdated { status, audit_info });
        self.status = status;
        Idempotent::Executed(())
    }
}

impl TryFromEvents<DepositAccountEvent> for DepositAccount {
    fn try_from_events(events: EntityEvents<DepositAccountEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositAccountBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositAccountEvent::Initialized {
                    id,
                    account_holder_id,
                    reference,
                    name,
                    description,
                    status,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_holder_id(*account_holder_id)
                        .reference(reference.to_string())
                        .name(name.to_string())
                        .description(description.to_string())
                        .status(*status)
                }
                DepositAccountEvent::AccountStatusUpdated { status, .. } => {
                    builder = builder.status(*status);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDepositAccount {
    #[builder(setter(into))]
    pub(super) id: DepositAccountId,
    #[builder(setter(into))]
    pub(super) account_holder_id: DepositAccountHolderId,
    pub(super) reference: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) active: bool,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewDepositAccount {
    pub fn builder() -> NewDepositAccountBuilder {
        NewDepositAccountBuilder::default()
    }
}

impl IntoEvents<DepositAccountEvent> for NewDepositAccount {
    fn into_events(self) -> EntityEvents<DepositAccountEvent> {
        EntityEvents::init(
            self.id,
            [DepositAccountEvent::Initialized {
                id: self.id,
                account_holder_id: self.account_holder_id,
                ledger_account_id: self.id.into(),
                reference: self.reference,
                name: self.name,
                description: self.description,
                status: if self.active {
                    AccountStatus::Active
                } else {
                    AccountStatus::Inactive
                },
                audit_info: self.audit_info,
            }],
        )
    }
}
