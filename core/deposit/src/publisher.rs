use outbox::{Outbox, OutboxEventMarker};

use super::event::CoreDepositEvent;
use crate::{
    account::{DepositAccount, DepositAccountEvent, error::DepositAccountError},
    deposit::{Deposit, DepositEvent, error::DepositError},
    withdrawal::{Withdrawal, WithdrawalEvent, error::WithdrawalError},
};

pub struct DepositPublisher<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for DepositPublisher<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> DepositPublisher<E>
where
    E: OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_deposit_account(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &DepositAccount,
        new_events: es_entity::LastPersisted<'_, DepositAccountEvent>,
    ) -> Result<(), DepositAccountError> {
        use DepositAccountEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreDepositEvent::DepositAccountCreated {
                    id: entity.id,
                    account_holder_id: entity.account_holder_id,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_withdrawal(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Withdrawal,
        new_events: es_entity::LastPersisted<'_, WithdrawalEvent>,
    ) -> Result<(), WithdrawalError> {
        use WithdrawalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Confirmed { .. } => Some(CoreDepositEvent::WithdrawalConfirmed {
                    id: entity.id,
                    deposit_account_id: entity.deposit_account_id,
                    amount: entity.amount,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_deposit(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Deposit,
        new_events: es_entity::LastPersisted<'_, DepositEvent>,
    ) -> Result<(), DepositError> {
        use DepositEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreDepositEvent::DepositInitialized {
                    id: entity.id,
                    deposit_account_id: entity.deposit_account_id,
                    amount: entity.amount,
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(db.tx(), publish_events)
            .await?;
        Ok(())
    }
}
