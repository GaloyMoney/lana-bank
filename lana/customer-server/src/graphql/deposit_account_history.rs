use async_graphql::*;

use crate::primitives::*;

use super::{
    credit_facility::disbursal::CreditFacilityDisbursal,
    credit_facility::obligation_installment::CreditFacilityObligationInstallment, deposit::Deposit,
    withdrawal::Withdrawal,
};

#[derive(Union)]
pub enum DepositAccountHistoryEntry {
    Deposit(DepositEntry),
    Withdrawal(WithdrawalEntry),
    CancelledWithdrawal(CancelledWithdrawalEntry),
    Disbursal(DisbursalEntry),
    Payment(PaymentEntry),
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
#[graphql(complex)]
pub struct DisbursalEntry {
    #[graphql(skip)]
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PaymentEntry {
    #[graphql(skip)]
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
pub struct UnknownEntry {
    pub tx_id: UUID,
    pub recorded_at: Timestamp,
}

#[ComplexObject]
impl DepositEntry {
    async fn deposit(&self, ctx: &Context<'_>) -> async_graphql::Result<Deposit> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let deposit = app
            .deposits()
            .for_subject(sub)?
            .find_deposit_by_id(self.tx_id)
            .await?;

        Ok(Deposit::from(deposit))
    }
}

#[ComplexObject]
impl WithdrawalEntry {
    async fn withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<Withdrawal> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let withdrawal = app
            .deposits()
            .for_subject(sub)?
            .find_withdrawal_by_id(self.tx_id)
            .await?;

        Ok(Withdrawal::from(withdrawal))
    }
}

#[ComplexObject]
impl CancelledWithdrawalEntry {
    async fn withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<Withdrawal> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let withdrawal = app
            .deposits()
            .for_subject(sub)?
            .find_withdrawal_by_cancelled_tx_id(self.tx_id)
            .await?;

        Ok(Withdrawal::from(withdrawal))
    }
}

#[ComplexObject]
impl DisbursalEntry {
    async fn disbursal(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityDisbursal> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursal = app
            .credit()
            .for_subject(sub)?
            .find_disbursal_by_concluded_tx_id(self.tx_id)
            .await?;

        Ok(CreditFacilityDisbursal::from(disbursal))
    }
}

#[ComplexObject]
impl PaymentEntry {
    async fn payment(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityObligationInstallment> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let payment = app
            .credit()
            .for_subject(sub)?
            .find_obligation_installment_by_id(self.tx_id)
            .await?;

        Ok(CreditFacilityObligationInstallment::from(payment))
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
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
                })
            }
            lana_app::deposit::DepositAccountHistoryEntry::Payment(entry) => {
                Self::Payment(PaymentEntry {
                    tx_id: UUID::from(entry.tx_id),
                    recorded_at: entry.recorded_at.into(),
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
