use super::*;

pub(super) async fn execute(
    credit_facility: &mut CreditFacility,
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ledger: &Ledger,
    audit: &Audit,
    interest_accrual_repo: InterestAccrualRepo,
    jobs: &Jobs,
    price: PriceOfOneBTC,
) -> Result<(), CreditFacilityError> {
    let Ok(credit_facility_activation) = credit_facility.activation_data(price) else {
        return Ok(());
    };

    ledger
        .activate_credit_facility(credit_facility_activation.clone())
        .await?;
    let audit_info = audit
        .record_system_entry_in_tx(
            db_tx,
            Object::CreditFacility,
            CreditFacilityAction::Activate,
        )
        .await?;
    credit_facility.activate(credit_facility_activation, chrono::Utc::now(), audit_info);

    let audit_info = audit
        .record_system_entry_in_tx(
            db_tx,
            Object::CreditFacility,
            CreditFacilityAction::RecordInterest,
        )
        .await?;
    let new_accrual = credit_facility
        .start_interest_accrual(audit_info.clone())?
        .expect("Accrual start date is before facility expiry date");
    let accrual = interest_accrual_repo
        .create_in_tx(db_tx, new_accrual)
        .await?;
    match jobs
        .create_and_spawn_at_in_tx(
            db_tx,
            credit_facility.id,
            interest::CreditFacilityJobConfig {
                credit_facility_id: credit_facility.id,
            },
            accrual
                .next_incurrence_period()
                .expect("New accrual has first incurrence period")
                .end,
        )
        .await
    {
        Ok(_) | Err(JobError::DuplicateId) => (),
        Err(err) => Err(err)?,
    };

    Ok(())
}
