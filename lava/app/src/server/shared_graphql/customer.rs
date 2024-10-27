use async_graphql::*;
use dataloader::DataLoader;

use crate::{
    app::LavaApp,
    ledger,
    primitives::{self, CustomerId},
    server::{
        admin::{
            graphql::{audit::AuditEntry, credit_facility::CreditFacility, loader::LavaDataLoader},
            AdminAuthContext,
        },
        shared_graphql::{deposit::*, loan::Loan, primitives::UUID, withdraw::*},
    },
};

use super::{balance::CustomerBalance, document::Document};

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

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Customer {
    customer_id: UUID,
    email: String,
    telegram_id: String,
    status: AccountStatus,
    level: KycLevel,
    applicant_id: Option<String>,
    #[graphql(skip)]
    account_ids: ledger::customer::CustomerLedgerAccountIds,
    #[graphql(skip)]
    audit_info: Vec<crate::audit::AuditInfo>,
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

    async fn deposits(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Deposit>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let deposits = app
            .deposits()
            .list_for_customer(sub, primitives::CustomerId::from(&self.customer_id))
            .await?
            .into_iter()
            .map(Deposit::from)
            .collect();
        Ok(deposits)
    }

    async fn withdrawals(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Withdrawal>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let withdraws = app
            .withdraws()
            .list_for_customer(sub, primitives::CustomerId::from(&self.customer_id))
            .await?
            .into_iter()
            .map(Withdrawal::from)
            .collect();
        Ok(withdraws)
    }

    async fn audit(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<AuditEntry>> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let entries = loader
            .load_many(self.audit_info.iter().map(|info| info.audit_entry_id))
            .await?;

        Ok(entries.into_values().collect())
    }

    async fn user_can_create_loan(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer_id = CustomerId::from(&self.customer_id);
        Ok(app
            .loans()
            .user_can_create_loan_for_customer(sub, customer_id, false)
            .await
            .is_ok())
    }

    async fn user_can_create_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app
            .credit_facilities()
            .user_can_create(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_record_deposit(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app.deposits().user_can_record(sub, false).await.is_ok())
    }

    async fn user_can_initiate_withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app.withdraws().user_can_initiate(sub, false).await.is_ok())
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacility>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let credit_facilities: Vec<CreditFacility> = app
            .credit_facilities()
            .list_for_customer(None, primitives::CustomerId::from(&self.customer_id))
            .await?
            .into_iter()
            .map(CreditFacility::from)
            .collect();

        Ok(credit_facilities)
    }

    async fn documents(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Document>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let documents = app
            .documents()
            .list_by_customer_id(sub, primitives::CustomerId::from(&self.customer_id))
            .await?;
        Ok(documents.into_iter().map(Document::from).collect())
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
    fn from(customer: crate::customer::Customer) -> Self {
        Customer {
            audit_info: customer.audit_info(),
            customer_id: UUID::from(customer.id),
            applicant_id: customer.applicant_id,
            email: customer.email,
            telegram_id: customer.telegram_id,
            account_ids: customer.account_ids,
            status: AccountStatus::from(customer.status),
            level: KycLevel::from(customer.level),
        }
    }
}
