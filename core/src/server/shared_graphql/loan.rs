use async_graphql::*;

use crate::{
    app::LavaApp,
    ledger,
    loan::TransactionType,
    primitives::{CustomerId, LoanStatus},
    server::shared_graphql::{customer::Customer, primitives::*, terms::TermValues},
};

use super::convert::ToGlobalId;

#[derive(SimpleObject)]
struct LoanTransaction {
    amount: TransactionAmount,
    transaction_type: TransactionType,
    recorded_at: Timestamp,
}

#[derive(Union)]
enum TransactionAmount {
    Sats(SatoshiAmount),
    Cents(UsdCentAmount),
}

#[derive(SimpleObject)]
struct SatoshiAmount {
    value: Satoshis,
}

#[derive(SimpleObject)]
struct UsdCentAmount {
    value: UsdCents,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Loan {
    id: ID,
    loan_id: UUID,
    created_at: Timestamp,
    loan_terms: TermValues,
    #[graphql(skip)]
    customer_id: UUID,
    #[graphql(skip)]
    account_ids: crate::ledger::loan::LoanAccountIds,
    status: LoanStatus,
    collateral: Satoshis,
    transactions: Vec<LoanTransaction>,
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
        let collateral = loan.collateral();
        let transactions = loan
            .transactions()
            .into_iter()
            .map(LoanTransaction::from)
            .collect();

        Loan {
            id: loan.id.to_global_id(),
            loan_id: UUID::from(loan.id),
            customer_id: UUID::from(loan.customer_id),
            status: loan.status(),
            loan_terms: TermValues::from(loan.terms),
            account_ids: loan.account_ids,
            created_at,
            collateral,
            transactions,
        }
    }
}

impl From<crate::loan::LoanTransaction> for LoanTransaction {
    fn from(tx: crate::loan::LoanTransaction) -> Self {
        let amount = match tx.amount {
            crate::loan::TransactionAmount::Sats(amt) => {
                TransactionAmount::Sats(SatoshiAmount { value: amt })
            }
            crate::loan::TransactionAmount::Cents(amt) => {
                TransactionAmount::Cents(UsdCentAmount { value: amt })
            }
        };

        Self {
            amount,
            transaction_type: tx.transaction_type,
            recorded_at: tx.recorded_at.into(),
        }
    }
}
