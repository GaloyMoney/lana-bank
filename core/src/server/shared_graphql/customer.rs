use async_graphql::*;

use crate::{
    app::LavaApp,
    ledger, primitives,
    server::shared_graphql::{loan::Loan, primitives::UUID},
};

use super::balance::CustomerBalance;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum KycLevel {
    Zero,
    One,
    Two,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum AccountStatus {
    Active,
    Inactive,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Customer {
    customer_id: UUID,
    email: String,
    status: AccountStatus,
    level: KycLevel,
    applicant_id: Option<String>,
    #[graphql(skip)]
    account_ids: ledger::customer::CustomerLedgerAccountIds,
}

#[ComplexObject]
impl Customer {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CustomerBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app.ledger().get_customer_balance(self.account_ids).await?;
        Ok(CustomerBalance::from(balance))
    }

    async fn loans(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Loan>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let loans: Vec<Loan> = app
            .loans()
            .list_for_customer(None, primitives::CustomerId::from(&self.customer_id))
            .await?
            .into_iter()
            .map(Loan::from)
            .collect();

        Ok(loans)
    }
}

impl From<primitives::KycLevel> for KycLevel {
    fn from(level: primitives::KycLevel) -> Self {
        match level {
            primitives::KycLevel::NotKyced => KycLevel::Zero,
            primitives::KycLevel::Basic => KycLevel::One,
            primitives::KycLevel::Advanced => KycLevel::Two,
        }
    }
}

impl From<primitives::AccountStatus> for AccountStatus {
    fn from(level: primitives::AccountStatus) -> Self {
        match level {
            primitives::AccountStatus::Active => AccountStatus::Active,
            primitives::AccountStatus::Inactive => AccountStatus::Inactive,
        }
    }
}

impl From<crate::customer::Customer> for Customer {
    fn from(user: crate::customer::Customer) -> Self {
        Customer {
            customer_id: UUID::from(user.id),
            applicant_id: user.applicant_id,
            email: user.email,
            account_ids: user.account_ids,
            status: AccountStatus::from(user.status),
            level: KycLevel::from(user.level),
        }
    }
}
