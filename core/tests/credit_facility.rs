mod helpers;

use chrono::Utc;
use rust_decimal_macros::dec;
use serial_test::file_serial;

use lava_core::{
    app::*,
    audit::Audit,
    authorization::Authorization,
    credit_facility::*,
    customer::{CustomerConfig, Customers},
    data_export::Export,
    job::{JobExecutorConfig, Jobs},
    ledger::{credit_facility::CreditFacilityBalance, Ledger, LedgerConfig},
    primitives::*,
    terms::*,
    user::User,
};
use uuid::Uuid;

async fn superuser() -> anyhow::Result<(User, Subject, AuditInfo)> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (_, superuser, superuser_subject) = helpers::init_users(&pool, &authz, &audit).await?;

    let audit_info = AuditInfo::from((AuditEntryId::from(1), superuser_subject));

    Ok((superuser, superuser_subject, audit_info))
}

fn random_email() -> String {
    format!("{}@integrationtest.com", Uuid::new_v4())
}

fn random_tg() -> String {
    format!("{}", Uuid::new_v4())
}

async fn create_new_customer(sub: &Subject) -> anyhow::Result<CustomerId> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let jobs = Jobs::new(&pool, JobExecutorConfig::default());
    let export = Export::new("".to_string(), &jobs);
    let ledger = Ledger::init(LedgerConfig::default(), &authz).await?;
    let customers = Customers::new(
        &pool,
        &CustomerConfig::default(),
        &ledger,
        &authz,
        &audit,
        &export,
    );

    let new_customer = customers
        .create_customer_through_admin(sub, random_email(), random_tg())
        .await?;

    Ok(new_customer.id)
}

async fn init_approved_facility_with_disbursement(
    facility: UsdCents,
    disbursement_amount: UsdCents,
    collateral: Satoshis,
) -> anyhow::Result<CreditFacility> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let ledger = Ledger::init(LedgerConfig::default(), &authz).await?;

    let app_config = AppConfig {
        ..Default::default()
    };
    let app = LavaApp::run(pool, app_config).await?;
    let credit_facilities = app.credit_facilities();

    let (user, sub, audit_info) = superuser().await?;

    let customer_id = create_new_customer(&sub).await?;

    let credit_facility_term_values = TermValues::builder()
        .annual_rate(AnnualRatePct::from(dec!(12)))
        .duration(Duration::Months(1))
        .accrual_interval(InterestInterval::EndOfMonth)
        .incurrence_interval(InterestInterval::EndOfDay)
        .liquidation_cvl(CVLPct::from(dec!(105)))
        .margin_call_cvl(CVLPct::from(dec!(125)))
        .initial_cvl(CVLPct::from(dec!(140)))
        .build()?;

    let CreditFacility { id, .. } = credit_facilities
        .create(&sub, customer_id, facility, credit_facility_term_values)
        .await?;
    credit_facilities
        .update_collateral(&sub, id, collateral)
        .await?;
    let mut credit_facility = credit_facilities.add_approval(&sub, id).await?;

    let new_disbursement =
        credit_facility.initiate_disbursement(audit_info, disbursement_amount)?;
    let mut disbursement = Disbursement::try_from(new_disbursement.initial_events())?;

    let disbursement_data = disbursement
        .add_approval(user.id, user.current_roles(), audit_info)?
        .expect("Approved");
    let executed_at = ledger
        .record_disbursement(disbursement_data.clone())
        .await?;
    disbursement.confirm_approval(&disbursement_data, executed_at, audit_info);
    credit_facility.confirm_disbursement(
        &disbursement,
        disbursement_data.tx_id,
        Utc::now(),
        audit_info,
    );

    Ok(credit_facility)
}

#[tokio::test]
#[file_serial]
async fn interest_accrual_lifecycle() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let ledger = Ledger::init(LedgerConfig::default(), &authz).await?;

    let (_, _, audit_info) = superuser().await?;

    let facility_amount = UsdCents::from(100_000_00);
    let disbursement_amount = UsdCents::from(50_000_00);
    let collateral = Satoshis::from(500_000_000_00);
    let mut credit_facility =
        init_approved_facility_with_disbursement(facility_amount, disbursement_amount, collateral)
            .await?;

    let new_accrual = credit_facility
        .start_interest_accrual(audit_info)?
        .expect("Accrual start date is before facility expiry date");
    let idx = credit_facility
        .interest_accrual_in_progress()
        .expect("Exists");
    assert_eq!(idx, InterestAccrualIdx::FIRST);

    let mut accrual = InterestAccrual::try_from(new_accrual.initial_events())?;

    let mut incurred_amount = UsdCents::ZERO;
    let mut interest_accrual = None;
    while interest_accrual.is_none() {
        let interest_incurrence =
            accrual.initiate_incurrence(credit_facility.outstanding(), credit_facility.account_ids);
        incurred_amount += interest_incurrence.interest;

        ledger
            .record_credit_facility_interest_incurrence(interest_incurrence.clone())
            .await?;
        interest_accrual = accrual.confirm_incurrence(
            interest_incurrence,
            credit_facility.account_ids,
            audit_info,
        );
    }
    assert!(accrual.idx >= InterestAccrualIdx::FIRST);

    // TODO: Check pending interest in ledger

    let interest_accrual = interest_accrual.expect("Exists");
    let executed_at = ledger
        .record_credit_facility_interest_accrual(interest_accrual.clone())
        .await?;
    accrual.confirm_accrual(interest_accrual.clone(), executed_at, audit_info);
    credit_facility.confirm_interest_accrual(
        interest_accrual,
        accrual.idx,
        executed_at,
        audit_info,
    );

    let CreditFacilityBalance {
        interest_receivable,
        ..
    } = ledger
        .get_credit_facility_balance(credit_facility.account_ids)
        .await?;
    assert_eq!(interest_receivable, incurred_amount);

    Ok(())
}
