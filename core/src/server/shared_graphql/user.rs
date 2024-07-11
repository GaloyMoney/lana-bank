use async_graphql::*;

use crate::server::admin::AdminAuthContext;
use crate::{
    app::LavaApp,
    ledger, primitives,
    server::shared_graphql::{loan::Loan, primitives::UUID},
};

use super::balance::UserBalance;

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
pub struct User {
    user_id: UUID,
    email: String,
    btc_deposit_address: String,
    ust_deposit_address: String,
    status: AccountStatus,
    level: KycLevel,
    applicant_id: Option<String>,
    #[graphql(skip)]
    account_ids: ledger::user::UserLedgerAccountIds,
}

#[ComplexObject]
impl User {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<UserBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app.ledger().get_user_balance(self.account_ids).await?;
        Ok(UserBalance::from(balance))
    }

    async fn loans(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Loan>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let loans: Vec<Loan> = app
            .loans()
            .list_for_user(Some(sub), primitives::UserId::from(&self.user_id))
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

impl From<crate::user::User> for User {
    fn from(user: crate::user::User) -> Self {
        User {
            user_id: UUID::from(user.id),
            applicant_id: user.applicant_id,
            btc_deposit_address: user.account_addresses.btc_address,
            ust_deposit_address: user.account_addresses.tron_usdt_address,
            email: user.email,
            account_ids: user.account_ids,
            status: AccountStatus::from(user.status),
            level: KycLevel::from(user.level),
        }
    }
}
