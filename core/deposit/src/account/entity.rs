use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::error::DepositAccountError;
use crate::{ledger::*, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositAccountId")]
pub enum DepositAccountEvent {
    Initialized {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
        account_ids: DepositAccountLedgerAccountIds,
        status: DepositAccountStatus,
        public_id: PublicId,
    },
    AccountStatusUpdated {
        status: DepositAccountStatus,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DepositAccount {
    pub id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
    pub account_ids: DepositAccountLedgerAccountIds,
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
    ) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::AccountStatusUpdated { status: existing_status, .. } if existing_status == &status,
            => DepositAccountEvent::AccountStatusUpdated { .. }
        );
        if status == DepositAccountStatus::Closed {
            return Err(DepositAccountError::CannotCloseAccount);
        }
        if self.status == DepositAccountStatus::Closed {
            return Err(DepositAccountError::CannotUpdateClosedAccount);
        }
        self.events
            .push(DepositAccountEvent::AccountStatusUpdated { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }

    pub fn freeze(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        if self.status != DepositAccountStatus::Active {
            return Err(DepositAccountError::CannotFreezeNonActiveAccount(self.id));
        }
        self.update_status(DepositAccountStatus::Frozen)
    }

    pub fn unfreeze(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        if self.status != DepositAccountStatus::Frozen {
            return Err(DepositAccountError::CannotUnfreezeNonFrozenAccount(self.id));
        }
        self.update_status(DepositAccountStatus::Active)
    }

    pub fn close(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::AccountStatusUpdated {
                status: DepositAccountStatus::Closed
            }
        );

        if self.status == DepositAccountStatus::Closed {
            return Err(DepositAccountError::CannotCloseAccount);
        }

        if self.status == DepositAccountStatus::Frozen {
            return Err(DepositAccountError::CannotCloseFrozenAccount);
        }

        let status = DepositAccountStatus::Closed;
        self.events
            .push(DepositAccountEvent::AccountStatusUpdated { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
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
                    account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_holder_id(*account_holder_id)
                        .account_ids(*account_ids)
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
    pub(super) account_ids: DepositAccountLedgerAccountIds,
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
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
                account_ids: self.account_ids,
                status: DepositAccountStatus::Active,
                public_id: self.public_id,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{EntityEvents, TryFromEvents as _};
    use public_id::PublicId;

    use crate::{DepositAccountHolderId, DepositAccountId, DepositAccountStatus};

    use super::{
        DepositAccount, DepositAccountError, DepositAccountEvent, DepositAccountLedgerAccountIds,
    };

    fn initial_events() -> Vec<DepositAccountEvent> {
        let id = DepositAccountId::new();
        vec![DepositAccountEvent::Initialized {
            id,
            account_holder_id: DepositAccountHolderId::new(),
            account_ids: DepositAccountLedgerAccountIds::new(id),
            status: DepositAccountStatus::Inactive,
            public_id: PublicId::new("1"),
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
                .update_status(DepositAccountStatus::Active)
                .unwrap()
                .did_execute()
        );

        assert!(
            account
                .update_status(DepositAccountStatus::Active)
                .unwrap()
                .was_ignored()
        );

        assert!(
            account
                .update_status(DepositAccountStatus::Frozen)
                .unwrap()
                .did_execute()
        );

        assert!(matches!(
            account.close(),
            Err(DepositAccountError::CannotCloseFrozenAccount)
        ));

        assert!(
            account
                .update_status(DepositAccountStatus::Active)
                .unwrap()
                .did_execute()
        );

        assert!(account.close().unwrap().did_execute());

        assert!(matches!(
            account.update_status(DepositAccountStatus::Active),
            Err(DepositAccountError::CannotUpdateClosedAccount)
        ));
    }

    #[test]
    fn cannot_freeze_inactive_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        assert_eq!(account.status, DepositAccountStatus::Inactive);
        assert!(account.freeze().is_err());
    }

    #[test]
    fn can_freeze_active_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        let _ = account.update_status(DepositAccountStatus::Active);
        assert!(account.freeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Frozen);
    }

    #[test]
    fn cannot_unfreeze_inactive_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        assert_eq!(account.status, DepositAccountStatus::Inactive);
        assert!(account.unfreeze().is_err());
    }

    #[test]
    fn can_unfreeze_frozen_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        let _ = account.update_status(DepositAccountStatus::Active);
        let _ = account.freeze().unwrap();
        assert!(account.unfreeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Active);
    }
}
