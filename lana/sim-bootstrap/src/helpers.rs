use futures::StreamExt;

use lana_events::*;

use lana_app::{
    app::LanaApp,
    customer::{CustomerId, CustomerType},
    primitives::{DepositAccountId, Subject, UsdCents},
    terms::{FacilityDuration, InterestInterval, ObligationDuration, TermValues},
};
use rust_decimal_macros::dec;

pub async fn create_customer(
    sub: &Subject,
    app: &LanaApp,
    suffix: &str,
) -> anyhow::Result<(CustomerId, DepositAccountId)> {
    let customer_email = format!("customer{}@example.com", suffix);
    let telegram = format!("customer{}", suffix);
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
            let mut stream = app.outbox().listen_persisted(None).await?;
            let customer = app
                .customers()
                .create(sub, customer_email.clone(), telegram, customer_type)
                .await?;
            while let Some(msg) = stream.next().await {
                if let Some(LanaEvent::Deposit(CoreDepositEvent::DepositAccountCreated {
                    account_holder_id,
                    id,
                })) = &msg.payload
                {
                    if CustomerId::from(*account_holder_id) == customer.id {
                        return Ok((customer.id, *id));
                    }
                }
            }
            unreachable!()
        }
    }
}

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
        .interest_due_from_accrual_after_days(ObligationDuration::Days(0))
        .obligation_overdue_duration(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .build()
        .unwrap()
}

pub fn std_terms_12m() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(FacilityDuration::Months(12))
        .interest_due_from_accrual_after_days(ObligationDuration::Days(0))
        .obligation_overdue_duration(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .build()
        .unwrap()
}
