use async_graphql::*;

use crate::{deposit::DepositBase, primitives::*, withdrawal::WithdrawalBase};

#[derive(Union)]
pub enum DepositAccountHistoryEntry {
    Deposit(DepositEntry),
    Withdrawal(WithdrawalEntry),
    CancelledWithdrawal(CancelledWithdrawalEntry),
    Disbursal(DisbursalEntry),
    Payment(PaymentEntry),
    Freeze(FreezeEntry),
    Unfreeze(UnfreezeEntry),
    Unknown(UnknownEntry),
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DepositEntry {
    #[graphql(skip)]
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct WithdrawalEntry {
    #[graphql(skip)]
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CancelledWithdrawalEntry {
    #[graphql(skip)]
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
pub struct DisbursalEntry {
    pub disbursal_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
pub struct PaymentEntry {
    pub payment_allocation_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
pub struct FreezeEntry {
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
    pub amount: UsdCents,
}

#[derive(SimpleObject)]
pub struct UnfreezeEntry {
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
    pub amount: UsdCents,
}

#[derive(SimpleObject)]
pub struct UnknownEntry {
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[ComplexObject]
impl DepositEntry {
    async fn deposit(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositBase> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let deposit = app
            .deposits()
            .find_deposit_by_id(sub, self.tx_id)
            .await?
            .expect("deposit should exist");
        Ok(DepositBase::from(deposit))
    }
}

#[ComplexObject]
impl WithdrawalEntry {
    async fn withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<WithdrawalBase> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let withdrawal = app
            .deposits()
            .find_withdrawal_by_id(sub, self.tx_id)
            .await?
            .expect("withdrawal should exist");
        Ok(WithdrawalBase::from(withdrawal))
    }
}

#[ComplexObject]
impl CancelledWithdrawalEntry {
    async fn withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<WithdrawalBase> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let withdrawal = app
            .deposits()
            .find_withdrawal_by_cancelled_tx_id(sub, self.tx_id)
            .await?;
        Ok(WithdrawalBase::from(withdrawal))
    }
}

impl From<lana_app::deposit::DepositAccountHistoryEntry> for DepositAccountHistoryEntry {
    fn from(entry: lana_app::deposit::DepositAccountHistoryEntry) -> Self {
        match entry {
            lana_app::deposit::DepositAccountHistoryEntry::Deposit(entry) => {
                Self::Deposit(DepositEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Withdrawal(entry) => {
                Self::Withdrawal(WithdrawalEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::CancelledWithdrawal(entry) => {
                Self::CancelledWithdrawal(CancelledWithdrawalEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Disbursal(entry) => {
                Self::Disbursal(DisbursalEntry {
                    disbursal_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Payment(entry) => {
                Self::Payment(PaymentEntry {
                    payment_allocation_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Freeze(entry) => {
                Self::Freeze(FreezeEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                    amount: entry.amount,
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Unfreeze(entry) => {
                Self::Unfreeze(UnfreezeEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                    amount: entry.amount,
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Unknown(entry) => {
                Self::Unknown(UnknownEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Ignored => {
                unreachable!("Ignored entries should not be returned to the client")
            }
        }
    }
}
