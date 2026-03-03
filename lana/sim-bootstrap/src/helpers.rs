use std::time::Duration;

use lana_app::{
    app::LanaApp,
    customer::{CustomerId, CustomerType},
    deposit::Withdrawal,
    primitives::{DepositAccountId, Subject, UsdCents, WithdrawalId},
    terms::{DisbursalPolicy, FacilityDuration, InterestInterval, ObligationDuration, TermValues},
};
use rust_decimal_macros::dec;

#[tracing::instrument(name = "sim_bootstrap.helpers.create_customer", skip(app), err)]
pub async fn create_customer(
    sub: &Subject,
    app: &LanaApp,
    suffix: &str,
) -> anyhow::Result<(CustomerId, DepositAccountId)> {
    let customer_email = format!("customer{suffix}@example.com");
    let telegram = format!("customer{suffix}");
    let customer_type = CustomerType::Individual;

    match app
        .customers()
        .find_by_email(sub, customer_email.clone())
        .await?
    {
        Some(existing_customer) => {
            let deposit_account_id = app
                .deposits()
                .list_accounts_by_created_at_for_account_holder(
                    sub,
                    existing_customer.id,
                    Default::default(),
                    es_entity::ListDirection::Descending,
                )
                .await?
                .entities
                .into_iter()
                .next()
                .expect("Deposit account not found")
                .id;
            Ok((existing_customer.id, deposit_account_id))
        }
        None => {
            let customer = app
                .customers()
                .create_customer_bypassing_kyc(sub, customer_email.clone(), telegram, customer_type)
                .await?;
            let deposit_account = app.deposits().create_account(sub, customer.id).await?;
            Ok((customer.id, deposit_account.id))
        }
    }
}

#[tracing::instrument(name = "sim_bootstrap.helpers.make_deposit", skip(app), err)]
pub async fn make_deposit(
    sub: &Subject,
    app: &LanaApp,
    customer_id: &CustomerId,
    usd_cents: UsdCents,
) -> anyhow::Result<()> {
    let deposit_account_id = app
        .deposits()
        .list_accounts_by_created_at_for_account_holder(
            sub,
            *customer_id,
            Default::default(),
            es_entity::ListDirection::Descending,
        )
        .await?
        .entities
        .into_iter()
        .next()
        .expect("Deposit account not found")
        .id;

    let _ = app
        .deposits()
        .record_deposit(sub, deposit_account_id, usd_cents, None)
        .await?;

    Ok(())
}

pub fn std_terms() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(FacilityDuration::Months(3))
        .interest_due_duration_from_accrual(ObligationDuration::Days(0))
        .obligation_overdue_duration_from_due(ObligationDuration::Days(50))
        .obligation_liquidation_duration_from_due(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .disbursal_policy(DisbursalPolicy::SingleDisbursal)
        .build()
        .expect("std_terms builder should be valid")
}

pub fn std_terms_with_liquidation() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(FacilityDuration::Months(3))
        .interest_due_duration_from_accrual(ObligationDuration::Days(0))
        .obligation_overdue_duration_from_due(ObligationDuration::Days(50))
        .obligation_liquidation_duration_from_due(ObligationDuration::Days(60))
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .disbursal_policy(DisbursalPolicy::SingleDisbursal)
        .build()
        .expect("std_terms_with_liquidation builder should be valid")
}

pub fn std_terms_12m() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(FacilityDuration::Months(12))
        .interest_due_duration_from_accrual(ObligationDuration::Days(0))
        .obligation_overdue_duration_from_due(ObligationDuration::Days(50))
        .obligation_liquidation_duration_from_due(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .disbursal_policy(DisbursalPolicy::MultipleDisbursal)
        .build()
        .expect("std_terms_12m builder should be valid")
}

#[tracing::instrument(
    name = "sim_bootstrap.helpers.approve_and_confirm_withdrawal",
    skip(app),
    err
)]
pub async fn approve_and_confirm_withdrawal(
    sub: &Subject,
    app: &LanaApp,
    withdrawal_id: WithdrawalId,
) -> anyhow::Result<Withdrawal> {
    for attempt in 1..=100 {
        let current = app
            .deposits()
            .find_withdrawal_by_id(sub, withdrawal_id)
            .await?
            .expect("withdrawal not found");

        if current.is_approved_or_denied() == Some(true) {
            break;
        }
        if current.is_approved_or_denied() == Some(false) {
            anyhow::bail!("withdrawal approval was denied");
        }
        if attempt == 100 {
            anyhow::bail!("withdrawal approval not processed in time");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let confirmed = app
        .deposits()
        .confirm_withdrawal(sub, withdrawal_id)
        .await?;
    Ok(confirmed)
}
