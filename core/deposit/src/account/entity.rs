use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use audit::AuditInfo;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositAccountId")]
pub enum DepositAccountEvent {
    Initialized {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
        ledger_account_id: CalaAccountId,
        frozen_deposit_account_id: CalaAccountId,
        status: DepositAccountStatus,
        public_id: PublicId,
        audit_info: AuditInfo,
    },
    AccountStatusUpdated {
        status: DepositAccountStatus,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DepositAccount {
    pub id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
    pub ledger_account_id: CalaAccountId,
    pub frozen_deposit_account_id: CalaAccountId,
    pub status: DepositAccountStatus,
    pub public_id: PublicId,

    events: EntityEvents<DepositAccountEvent>,
}

impl DepositAccount {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("Deposit Account has never been persisted")
    }

    pub fn update_status(
        &mut self,
        status: DepositAccountStatus,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::AccountStatusUpdated { status: existing_status, .. } if existing_status == &status,
            => DepositAccountEvent::AccountStatusUpdated { .. }
        );
        self.events
            .push(DepositAccountEvent::AccountStatusUpdated { status, audit_info });
        self.status = status;
        Idempotent::Executed(())
    }

    pub fn freeze(&mut self, audit_info: AuditInfo) -> Idempotent<()> {
        self.update_status(DepositAccountStatus::Frozen, audit_info)
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
                    status,
                    public_id,
                    ledger_account_id,
                    frozen_deposit_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_holder_id(*account_holder_id)
                        .ledger_account_id(*ledger_account_id)
                        .frozen_deposit_account_id(*frozen_deposit_account_id)
                        .status(*status)
                        .public_id(public_id.clone())
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
    #[builder(setter(into))]
    pub(super) ledger_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(super) frozen_deposit_account_id: CalaAccountId,
    pub(super) active: bool,
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
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
                ledger_account_id: self.ledger_account_id,
                frozen_deposit_account_id: self.frozen_deposit_account_id,
                status: if self.active {
                    DepositAccountStatus::Active
                } else {
                    DepositAccountStatus::Inactive
                },
                public_id: self.public_id,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use audit::{AuditEntryId, AuditInfo};
    use cala_ledger::AccountId as CalaAccountId;
    use es_entity::{EntityEvents, TryFromEvents as _};
    use public_id::PublicId;

    use crate::{DepositAccountHolderId, DepositAccountId, DepositAccountStatus};

    use super::{DepositAccount, DepositAccountEvent};

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn initial_events() -> Vec<DepositAccountEvent> {
        vec![DepositAccountEvent::Initialized {
            id: DepositAccountId::new(),
            account_holder_id: DepositAccountHolderId::new(),
            ledger_account_id: CalaAccountId::new(),
            frozen_deposit_account_id: CalaAccountId::new(),
            status: DepositAccountStatus::Inactive,
            public_id: PublicId::new("1"),
            audit_info: dummy_audit_info(),
        }]
    }

    #[test]
    fn update_status_idempotency() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        assert!(
            account
                .update_status(DepositAccountStatus::Active, dummy_audit_info())
                .did_execute()
        );

        assert!(
            account
                .update_status(DepositAccountStatus::Active, dummy_audit_info())
                .was_ignored()
        );

        assert!(
            account
                .update_status(DepositAccountStatus::Frozen, dummy_audit_info())
                .did_execute()
        );

        assert!(
            account
                .update_status(DepositAccountStatus::Active, dummy_audit_info())
                .did_execute()
        );
    }
}
