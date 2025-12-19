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
    AccountHolderStatusUpdated {
        status: DepositAccountStatus,
    },
    Frozen {
        status: DepositAccountStatus,
    },
    Unfrozen {
        status: DepositAccountStatus,
    },
    Closed {
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

    pub fn is_closed(&self) -> bool {
        self.status == DepositAccountStatus::Closed
    }

    pub fn is_frozen(&self) -> bool {
        self.status == DepositAccountStatus::Frozen
    }

    pub fn update_status_via_holder(
        &mut self,
        status: DepositAccountHolderStatus,
    ) -> Result<Idempotent<()>, DepositAccountError> {
        let status = status.into();
        if self.status == status {
            return Ok(Idempotent::AlreadyApplied);
        }
        if self.is_closed() {
            return Err(DepositAccountError::CannotUpdateClosedAccount(self.id));
        }
        if self.is_frozen() {
            return Err(DepositAccountError::CannotUpdateFrozenAccount(self.id));
        }
        self.events
            .push(DepositAccountEvent::AccountHolderStatusUpdated { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }

    pub fn freeze(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::Frozen { .. },
            => DepositAccountEvent::Unfrozen { .. }
        );
        if self.is_closed() {
            return Err(DepositAccountError::CannotUpdateClosedAccount(self.id));
        }
        if self.status == DepositAccountStatus::Inactive {
            return Err(DepositAccountError::CannotFreezeInactiveAccount(self.id));
        }
        let status = DepositAccountStatus::Frozen;
        self.events.push(DepositAccountEvent::Frozen { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }

    pub fn unfreeze(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::Unfrozen { .. },
            => DepositAccountEvent::Frozen { .. }
        );
        if !self.is_frozen() {
            return Ok(Idempotent::AlreadyApplied);
        }
        if self.is_closed() {
            return Err(DepositAccountError::CannotUpdateClosedAccount(self.id));
        }
        let status = DepositAccountStatus::Active;
        self.events.push(DepositAccountEvent::Unfrozen { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }

    pub fn close(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            DepositAccountEvent::Closed { .. }
        );
        if self.is_frozen() {
            return Err(DepositAccountError::CannotUpdateFrozenAccount(self.id));
        }

        let status = DepositAccountStatus::Closed;
        self.events.push(DepositAccountEvent::Closed { status });
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
                DepositAccountEvent::AccountHolderStatusUpdated { status, .. } => {
                    builder = builder.status(*status);
                }
                DepositAccountEvent::Frozen { status, .. } => {
                    builder = builder.status(*status);
                }
                DepositAccountEvent::Unfrozen { status, .. } => {
                    builder = builder.status(*status);
                }
                DepositAccountEvent::Closed { status, .. } => {
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

    use crate::{
        DepositAccountHolderId, DepositAccountHolderStatus, DepositAccountId, DepositAccountStatus,
    };

    use super::{
        DepositAccount, DepositAccountError, DepositAccountEvent, DepositAccountLedgerAccountIds,
    };

    fn initial_events() -> Vec<DepositAccountEvent> {
        let id = DepositAccountId::new();
        vec![DepositAccountEvent::Initialized {
            id,
            account_holder_id: DepositAccountHolderId::new(),
            account_ids: DepositAccountLedgerAccountIds::new(id),
            status: DepositAccountStatus::Active,
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
                .update_status_via_holder(DepositAccountHolderStatus::Inactive)
                .unwrap()
                .did_execute()
        );

        assert!(
            account
                .update_status_via_holder(DepositAccountHolderStatus::Inactive)
                .unwrap()
                .was_already_applied()
        );

        assert!(
            account
                .update_status_via_holder(DepositAccountHolderStatus::Active)
                .unwrap()
                .did_execute()
        );
    }

    #[test]
    fn cannot_freeze_non_active_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        let _ = account.update_status_via_holder(DepositAccountHolderStatus::Inactive);
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

        assert_eq!(account.status, DepositAccountStatus::Active);
        assert!(account.freeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Frozen);
    }

    #[test]
    fn can_unfreeze_frozen_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        let _ = account.freeze().unwrap();
        assert_eq!(account.status, DepositAccountStatus::Frozen);
        assert!(account.unfreeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Active);
    }

    #[test]
    fn can_close_active_or_inactive_account() {
        let mut account_1 = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();
        assert_eq!(account_1.status, DepositAccountStatus::Active);

        assert!(account_1.close().unwrap().did_execute());
        assert_eq!(account_1.status, DepositAccountStatus::Closed);

        let mut account_2 = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();
        let _ = account_2
            .update_status_via_holder(DepositAccountHolderStatus::Inactive)
            .unwrap();
        assert_eq!(account_2.status, DepositAccountStatus::Inactive);

        assert!(account_2.close().unwrap().did_execute());
        assert_eq!(account_2.status, DepositAccountStatus::Closed);
    }

    #[test]
    fn can_not_close_frozen_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();
        assert!(account.freeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Frozen);

        assert!(matches!(
            account.close(),
            Err(DepositAccountError::CannotUpdateFrozenAccount(_))
        ));
    }

    #[test]
    fn can_not_update_closed_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();
        assert!(account.close().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Closed);

        assert!(matches!(
            account.freeze(),
            Err(DepositAccountError::CannotUpdateClosedAccount(_))
        ));

        assert!(matches!(
            account.update_status_via_holder(DepositAccountHolderStatus::Active),
            Err(DepositAccountError::CannotUpdateClosedAccount(_))
        ));
        assert!(matches!(
            account.update_status_via_holder(DepositAccountHolderStatus::Inactive),
            Err(DepositAccountError::CannotUpdateClosedAccount(_))
        ));
    }

    #[test]
    fn can_freeze_and_unfreeze_multiple_times() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();
        assert!(account.freeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Frozen);
        assert!(account.freeze().unwrap().was_already_applied());

        assert!(account.unfreeze().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Active);
        assert!(
            account
                .update_status_via_holder(DepositAccountHolderStatus::Active)
                .unwrap()
                .was_already_applied()
        );

        assert!(account.freeze().unwrap().did_execute());
    }
}
