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
        #[serde(default = "default_account_activity")]
        activity: Activity,
        public_id: PublicId,
    },
    ActivityUpdated {
        activity: Activity,
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
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct DepositAccount {
    pub id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
    pub account_ids: DepositAccountLedgerAccountIds,
    pub status: DepositAccountStatus,
    pub activity: Activity,
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

    pub(crate) fn update_activity(&mut self, activity: Activity) -> Idempotent<()> {
        if self.activity == Activity::Escheatable {
            return Idempotent::AlreadyApplied;
        }
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: DepositAccountEvent::ActivityUpdated { activity: existing_activity, .. } if existing_activity == &activity,
            resets_on: DepositAccountEvent::ActivityUpdated { .. }
        );
        self.events
            .push(DepositAccountEvent::ActivityUpdated { activity });
        self.activity = activity;
        Idempotent::Executed(())
    }

    pub fn freeze(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: DepositAccountEvent::Frozen { .. },
            resets_on: DepositAccountEvent::Unfrozen { .. }
        );
        if self.is_closed() {
            return Err(DepositAccountError::CannotUpdateClosedAccount(self.id));
        }
        let status = DepositAccountStatus::Frozen;
        self.events.push(DepositAccountEvent::Frozen { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }

    pub fn unfreeze(&mut self) -> Result<Idempotent<()>, DepositAccountError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            already_applied: DepositAccountEvent::Unfrozen { .. },
            resets_on: DepositAccountEvent::Frozen { .. }
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
            already_applied: DepositAccountEvent::Closed { .. }
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
    fn try_from_events(
        events: EntityEvents<DepositAccountEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = DepositAccountBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositAccountEvent::Initialized {
                    id,
                    account_holder_id,
                    status,
                    activity,
                    public_id,
                    account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_holder_id(*account_holder_id)
                        .account_ids(*account_ids)
                        .status(*status)
                        .activity(*activity)
                        .public_id(public_id.clone())
                }
                DepositAccountEvent::ActivityUpdated { activity, .. } => {
                    builder = builder.activity(*activity);
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
    #[builder(setter(skip), default = "Activity::Active")]
    pub(super) activity: Activity,
    #[builder(setter(skip), default)]
    pub(super) status: DepositAccountStatus,
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
                activity: self.activity,
                public_id: self.public_id,
            }],
        )
    }
}

fn default_account_activity() -> Activity {
    Activity::Active
}

#[cfg(test)]
mod tests {
    use es_entity::{EntityEvents, TryFromEvents as _};
    use public_id::PublicId;

    use crate::{Activity, DepositAccountHolderId, DepositAccountId, DepositAccountStatus};

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
            activity: Activity::Active,
            public_id: PublicId::new("1"),
        }]
    }

    #[test]
    fn update_activity_idempotency() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        assert!(account.update_activity(Activity::Inactive).did_execute());
        assert_eq!(account.activity, Activity::Inactive);

        assert!(
            account
                .update_activity(Activity::Inactive)
                .was_already_applied()
        );

        assert!(account.update_activity(Activity::Active).did_execute());
        assert_eq!(account.activity, Activity::Active);
    }

    #[test]
    fn escheatable_activity_is_terminal() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();

        assert!(account.update_activity(Activity::Escheatable).did_execute());
        assert_eq!(account.activity, Activity::Escheatable);

        assert!(
            account
                .update_activity(Activity::Inactive)
                .was_already_applied()
        );
        assert_eq!(account.activity, Activity::Escheatable);

        assert!(
            account
                .update_activity(Activity::Active)
                .was_already_applied()
        );
        assert_eq!(account.activity, Activity::Escheatable);
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
    fn can_close_active_account() {
        let mut account = DepositAccount::try_from_events(EntityEvents::init(
            DepositAccountId::new(),
            initial_events(),
        ))
        .unwrap();
        assert_eq!(account.status, DepositAccountStatus::Active);

        assert!(account.close().unwrap().did_execute());
        assert_eq!(account.status, DepositAccountStatus::Closed);
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
    fn can_not_freeze_closed_account() {
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

        assert!(account.freeze().unwrap().did_execute());
    }
}
