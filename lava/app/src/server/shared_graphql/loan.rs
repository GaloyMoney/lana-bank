use async_graphql::*;

use crate::{
    app::LavaApp,
    ledger,
    loan::LoanCollaterizationState,
    primitives::{CollateralAction, CustomerId, LoanId, LoanStatus, UserId},
    server::{
        admin::{graphql::user::User, AdminAuthContext},
        shared_graphql::{customer::Customer, primitives::*, terms::TermValues},
    },
    terms::CVLData,
};

use super::{
    convert::ToGlobalId,
    objects::{Collateral, Outstanding},
    terms::CVLPct,
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Loan {
    id: ID,
    loan_id: UUID,
    created_at: Timestamp,
    approved_at: Option<Timestamp>,
    expires_at: Option<Timestamp>,
    loan_terms: TermValues,
    #[graphql(skip)]
    customer_id: UUID,
    #[graphql(skip)]
    account_ids: crate::ledger::loan::LoanAccountIds,
    #[graphql(skip)]
    cvl_data: CVLData,
    status: LoanStatus,
    collateral: Satoshis,
    principal: UsdCents,
    transactions: Vec<LoanHistoryEntry>,
    approvals: Vec<LoanApproval>,
    repayment_plan: Vec<LoanRepaymentInPlan>,
    collateralization_state: LoanCollaterizationState,
}

#[derive(SimpleObject)]
pub struct LoanRepaymentInPlan {
    pub repayment_type: LoanRepaymentType,
    pub status: LoanRepaymentStatus,
    pub initial: UsdCents,
    pub outstanding: UsdCents,
    pub accrual_at: Timestamp,
    pub due_at: Timestamp,
}

impl From<crate::loan::LoanRepaymentInPlan> for LoanRepaymentInPlan {
    fn from(repayment: crate::loan::LoanRepaymentInPlan) -> Self {
        match repayment {
            crate::loan::LoanRepaymentInPlan::Interest(interest) => LoanRepaymentInPlan {
                repayment_type: LoanRepaymentType::Interest,
                status: interest.status.into(),
                initial: interest.initial,
                outstanding: interest.outstanding,
                accrual_at: interest.accrual_at.into(),
                due_at: interest.due_at.into(),
            },
            crate::loan::LoanRepaymentInPlan::Principal(interest) => LoanRepaymentInPlan {
                repayment_type: LoanRepaymentType::Principal,
                status: interest.status.into(),
                initial: interest.initial,
                outstanding: interest.outstanding,
                accrual_at: interest.accrual_at.into(),
                due_at: interest.due_at.into(),
            },
        }
    }
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum LoanRepaymentType {
    Principal,
    Interest,
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum LoanRepaymentStatus {
    Upcoming,
    Due,
    Overdue,
    Paid,
}

impl From<crate::loan::RepaymentStatus> for LoanRepaymentStatus {
    fn from(status: crate::loan::RepaymentStatus) -> Self {
        match status {
            crate::loan::RepaymentStatus::Paid => LoanRepaymentStatus::Paid,
            crate::loan::RepaymentStatus::Due => LoanRepaymentStatus::Due,
            crate::loan::RepaymentStatus::Overdue => LoanRepaymentStatus::Overdue,
            crate::loan::RepaymentStatus::Upcoming => LoanRepaymentStatus::Upcoming,
        }
    }
}

#[derive(async_graphql::Union)]
pub enum LoanHistoryEntry {
    Payment(IncrementalPayment),
    Interest(InterestAccrued),
    Collateral(CollateralUpdated),
    Origination(LoanOrigination),
    Collateralization(CollateralizationUpdated),
}

#[derive(SimpleObject)]
pub struct IncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct InterestAccrued {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: Timestamp,
    pub action: CollateralAction,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct LoanOrigination {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CollateralizationUpdated {
    pub state: LoanCollaterizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_principal: UsdCents,
    pub price: UsdCents,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct LoanApproval {
    user_id: UUID,
    approved_at: Timestamp,
}

#[ComplexObject]
impl Loan {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<LoanBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app.ledger().get_loan_balance(self.account_ids).await?;
        Ok(LoanBalance::from(balance))
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app
            .customers()
            .find_by_id(None, CustomerId::from(&self.customer_id))
            .await?;

        match user {
            Some(user) => Ok(Customer::from(user)),
            None => panic!("user not found for a loan. should not be possible"),
        }
    }

    async fn current_cvl(&self, ctx: &Context<'_>) -> async_graphql::Result<CVLPct> {
        let app = ctx.data_unchecked::<LavaApp>();
        let price = app.price().usd_cents_per_btc().await?;
        Ok(self.cvl_data.cvl(price))
    }

    async fn user_can_approve(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let loan_id = LoanId::from(&self.loan_id);
        Ok(app
            .loans()
            .user_can_approve(sub, loan_id, false)
            .await
            .is_ok())
    }

    async fn user_can_update_collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let loan_id = LoanId::from(&self.loan_id);
        Ok(app
            .loans()
            .user_can_update_collateral(sub, loan_id, false)
            .await
            .is_ok())
    }

    async fn user_can_update_collateralization_state(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let loan_id = LoanId::from(&self.loan_id);
        Ok(app
            .loans()
            .user_can_update_collateralization_state(sub, loan_id, false)
            .await
            .is_ok())
    }

    async fn user_can_record_payment_or_complete_loan(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let loan_id = LoanId::from(&self.loan_id);
        Ok(app
            .loans()
            .user_can_record_payment_or_complete_loan(sub, loan_id, false)
            .await
            .is_ok())
    }
}

#[ComplexObject]
impl LoanApproval {
    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app
            .users()
            .find_by_id_internal(UserId::from(&self.user_id))
            .await?
            .expect("should always find user for a given UserId");
        Ok(User::from(user))
    }
}

#[derive(SimpleObject)]
struct InterestIncome {
    usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub(super) struct LoanBalance {
    collateral: Collateral,
    outstanding: Outstanding,
    interest_incurred: InterestIncome,
}

impl From<ledger::loan::LoanBalance> for LoanBalance {
    fn from(balance: ledger::loan::LoanBalance) -> Self {
        Self {
            collateral: Collateral {
                btc_balance: balance.collateral,
            },
            outstanding: Outstanding {
                usd_balance: balance.principal_receivable + balance.interest_receivable,
            },
            interest_incurred: InterestIncome {
                usd_balance: balance.interest_incurred,
            },
        }
    }
}

impl ToGlobalId for crate::primitives::LoanId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("loan:{}", self))
    }
}

impl From<crate::loan::Loan> for Loan {
    fn from(loan: crate::loan::Loan) -> Self {
        let created_at = loan.created_at().into();
        let approved_at: Option<Timestamp> = loan.approved_at.map(|a| a.into());
        let expires_at: Option<Timestamp> = loan.expires_at.map(|e| e.into());

        let collateral = loan.collateral();
        let principal = loan.initial_principal();
        let transactions = loan
            .history()
            .into_iter()
            .map(LoanHistoryEntry::from)
            .collect();
        let repayment_plan = loan
            .repayment_plan()
            .into_iter()
            .map(LoanRepaymentInPlan::from)
            .collect();
        let collateralization_state = loan.collateralization();
        let approvals = loan
            .approvals()
            .into_iter()
            .map(LoanApproval::from)
            .collect();

        Loan {
            id: loan.id.to_global_id(),
            loan_id: UUID::from(loan.id),
            customer_id: UUID::from(loan.customer_id),
            status: loan.status(),
            loan_terms: TermValues::from(loan.terms),
            account_ids: loan.account_ids,
            cvl_data: loan.cvl_data(),
            created_at,
            approved_at,
            expires_at,
            collateral,
            principal,
            transactions,
            approvals,
            repayment_plan,
            collateralization_state,
        }
    }
}

impl From<crate::loan::LoanHistoryEntry> for LoanHistoryEntry {
    fn from(transaction: crate::loan::LoanHistoryEntry) -> Self {
        match transaction {
            crate::loan::LoanHistoryEntry::Payment(payment) => {
                LoanHistoryEntry::Payment(payment.into())
            }
            crate::loan::LoanHistoryEntry::Interest(interest) => {
                LoanHistoryEntry::Interest(interest.into())
            }
            crate::loan::LoanHistoryEntry::Collateral(collateral) => {
                LoanHistoryEntry::Collateral(collateral.into())
            }
            crate::loan::LoanHistoryEntry::Origination(origination) => {
                LoanHistoryEntry::Origination(origination.into())
            }
            crate::loan::LoanHistoryEntry::Collateralization(collateralization) => {
                LoanHistoryEntry::Collateralization(collateralization.into())
            }
        }
    }
}

impl From<crate::loan::IncrementalPayment> for IncrementalPayment {
    fn from(payment: crate::loan::IncrementalPayment) -> Self {
        IncrementalPayment {
            cents: payment.cents,
            recorded_at: payment.recorded_at.into(),
            tx_id: payment.tx_id.into(),
        }
    }
}

impl From<crate::loan::InterestAccrued> for InterestAccrued {
    fn from(interest: crate::loan::InterestAccrued) -> Self {
        InterestAccrued {
            cents: interest.cents,
            recorded_at: interest.recorded_at.into(),
            tx_id: interest.tx_id.into(),
        }
    }
}

impl From<crate::loan::CollateralUpdated> for CollateralUpdated {
    fn from(collateral: crate::loan::CollateralUpdated) -> Self {
        CollateralUpdated {
            satoshis: collateral.satoshis,
            recorded_at: collateral.recorded_at.into(),
            action: collateral.action,
            tx_id: collateral.tx_id.into(),
        }
    }
}

impl From<crate::loan::LoanOrigination> for LoanOrigination {
    fn from(origination: crate::loan::LoanOrigination) -> Self {
        LoanOrigination {
            cents: origination.cents,
            recorded_at: origination.recorded_at.into(),
            tx_id: origination.tx_id.into(),
        }
    }
}

impl From<crate::loan::CollateralizationUpdated> for CollateralizationUpdated {
    fn from(collateralization: crate::loan::CollateralizationUpdated) -> Self {
        CollateralizationUpdated {
            state: collateralization.state,
            collateral: collateralization.collateral,
            outstanding_interest: collateralization.outstanding_interest,
            outstanding_principal: collateralization.outstanding_principal,
            price: collateralization.price.into_inner(),
            recorded_at: collateralization.recorded_at.into(),
        }
    }
}

impl From<crate::loan::LoanApproval> for LoanApproval {
    fn from(approver: crate::loan::LoanApproval) -> Self {
        LoanApproval {
            user_id: UUID::from(approver.user_id),
            approved_at: approver.approved_at.into(),
        }
    }
}
