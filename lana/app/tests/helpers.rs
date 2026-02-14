#![allow(dead_code)]

use lana_app::{
    access::{Access, config::AccessConfig},
    authorization::{Authorization, seed},
    outbox::Outbox,
    primitives::Subject,
};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_access(
    pool: &sqlx::PgPool,
    authz: &Authorization,
) -> anyhow::Result<(Access, Subject)> {
    let superuser_email = "superuser@test.io".to_string();
    let outbox = Outbox::init(pool, obix::MailboxConfig::builder().build()?).await?;

    let config = AccessConfig {
        superuser_email: Some(superuser_email.clone()),
    };

    let clock = es_entity::clock::ClockHandle::realtime();
    let access = Access::init(
        pool,
        config,
        rbac_types::LanaAction::action_descriptions(),
        seed::PREDEFINED_ROLES,
        &[
            lana_app::customer::CUSTOMER_SYNC,
            lana_app::customer::SUMSUB,
            lana_app::credit::INTEREST_ACCRUAL,
            lana_app::credit::COLLATERALIZATION_SYNC,
            lana_app::credit::CREDIT_FACILITY_ACTIVATION,
            lana_app::credit::PENDING_FACILITY_CREATION,
            lana_app::credit::CREDIT_FACILITY_PAYMENT_ALLOCATION,
            lana_app::credit::DISBURSAL_APPROVAL,
            lana_app::credit::OBLIGATION_SYNC,
            lana_app::deposit::DEPOSIT_APPROVAL,
            lana_app::custody::CUSTODY_KEY_ROTATION,
            lana_app::accounting::ACCOUNTING_TRIAL_BALANCE,
            lana_app::governance::GOVERNANCE,
        ],
        authz,
        &outbox,
        clock,
    )
    .await?;

    let superuser = access
        .users()
        .find_by_email(None, &superuser_email)
        .await?
        .expect("Superuser not found");

    Ok((access, Subject::from(superuser.id)))
}
