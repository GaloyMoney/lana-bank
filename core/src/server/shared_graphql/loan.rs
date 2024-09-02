use async_graphql::*;

use crate::{
    app::LavaApp,
    ledger,
    loan::{ApprovalStatus, LoanCollaterizationState},
    primitives::{CollateralAction, CustomerId, LoanStatus},
    server::shared_graphql::{customer::Customer, primitives::*, terms::TermValues},
};

use super::convert::ToGlobalId;

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
    status: LoanStatus,
    collateral: Satoshis,
    principal: UsdCents,
    transactions: Vec<LoanHistory>,
    collateralization_state: LoanCollaterizationState,
    approval_status: ApprovalStatus,
}

#[derive(async_graphql::Union)]
pub enum LoanHistory {
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
}

#[derive(SimpleObject)]
struct Collateral {
    btc_balance: Satoshis,
}

#[derive(SimpleObject)]
struct LoanOutstanding {
    usd_balance: UsdCents,
}

#[derive(SimpleObject)]
struct InterestIncome {
    usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub(super) struct LoanBalance {
    collateral: Collateral,
    outstanding: LoanOutstanding,
    interest_incurred: InterestIncome,
}

impl From<ledger::loan::LoanBalance> for LoanBalance {
    fn from(balance: ledger::loan::LoanBalance) -> Self {
        Self {
            collateral: Collateral {
                btc_balance: balance.collateral,
            },
            outstanding: LoanOutstanding {
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
        let approved_at: Option<Timestamp> = loan.approved_at().map(|a| a.into());
        let expires_at: Option<Timestamp> = loan.expires_at().map(|e| e.into());

        let collateral = loan.collateral();
        let principal = loan.initial_principal();
        let transactions = loan.history().into_iter().map(LoanHistory::from).collect();
        let collateralization_state = loan.collateralization();
        let approval_status = loan.approval_status();

        Loan {
            id: loan.id.to_global_id(),
            loan_id: UUID::from(loan.id),
            customer_id: UUID::from(loan.customer_id),
            status: loan.status(),
            loan_terms: TermValues::from(loan.terms),
            account_ids: loan.account_ids,
            created_at,
            approved_at,
            expires_at,
            collateral,
            principal,
            transactions,
            collateralization_state,
            approval_status,
        }
    }
}

impl From<crate::loan::LoanHistory> for LoanHistory {
    fn from(transaction: crate::loan::LoanHistory) -> Self {
        match transaction {
            crate::loan::LoanHistory::Payment(payment) => LoanHistory::Payment(payment.into()),
            crate::loan::LoanHistory::Interest(interest) => LoanHistory::Interest(interest.into()),
            crate::loan::LoanHistory::Collateral(collateral) => {
                LoanHistory::Collateral(collateral.into())
            }
            crate::loan::LoanHistory::Origination(origination) => {
                LoanHistory::Origination(origination.into())
            }
            crate::loan::LoanHistory::Collateralization(collateralization) => {
                LoanHistory::Collateralization(collateralization.into())
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
