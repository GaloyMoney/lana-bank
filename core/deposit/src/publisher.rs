use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CoreDepositEvent, PublicDeposit, PublicDepositAccount, PublicWithdrawal,
    account::{DepositAccount, DepositAccountEvent},
    deposit::{Deposit, DepositEvent},
    withdrawal::{Withdrawal, WithdrawalEvent},
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

    pub async fn publish_deposit_account_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &DepositAccount,
        new_events: es_entity::LastPersisted<'_, DepositAccountEvent>,
    ) -> Result<(), sqlx::Error> {
        use DepositAccountEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Initialized { .. } => Some(CoreDepositEvent::DepositAccountCreated {
                    entity: PublicDepositAccount::from(entity),
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_withdrawal_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Withdrawal,
        new_events: es_entity::LastPersisted<'_, WithdrawalEvent>,
    ) -> Result<(), sqlx::Error> {
        use WithdrawalEvent::*;
        let publish_events = new_events
            .filter_map(|event| match &event.event {
                Confirmed { .. } => Some(CoreDepositEvent::WithdrawalConfirmed {
                    entity: PublicWithdrawal::from(entity),
                }),
                ApprovalProcessConcluded { .. } => {
                    Some(CoreDepositEvent::WithdrawalApprovalConcluded {
                        entity: PublicWithdrawal::from(entity),
                    })
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }

    pub async fn publish_deposit_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Deposit,
        new_events: es_entity::LastPersisted<'_, DepositEvent>,
    ) -> Result<(), sqlx::Error> {
        use DepositEvent::*;
        let publish_events = new_events
            .map(|event| match &event.event {
                Initialized { .. } => CoreDepositEvent::DepositInitialized {
                    entity: PublicDeposit::from(entity),
                },
                Reverted { .. } => CoreDepositEvent::DepositReverted {
                    entity: PublicDeposit::from(entity),
                },
            })
            .collect::<Vec<_>>();
        self.outbox
            .publish_all_persisted(op, publish_events)
            .await?;
        Ok(())
    }
}
