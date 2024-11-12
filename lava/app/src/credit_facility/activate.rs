use crate::audit::Audit;

use super::*;

pub(super) async fn execute(
    credit_facility: &mut CreditFacility,
    db: &mut es_entity::DbOp<'_>,
    ledger: &Ledger,
    audit: &Audit,
    repo: &CreditFacilityRepo,
    jobs: &Jobs,
    price: PriceOfOneBTC,
) -> Result<(), CreditFacilityError> {
    let Ok(credit_facility_activation) = credit_facility.activation_data(price) else {
        return Ok(());
    };

    let audit_info = audit
        .record_system_entry_in_tx(
            db.tx(),
            Object::CreditFacility,
            CreditFacilityAction::Activate,
        )
        .await?;
    credit_facility.activate(
        credit_facility_activation.clone(),
        chrono::Utc::now(),
        audit_info,
    );

    let audit_info = audit
        .record_system_entry_in_tx(
            db.tx(),
            Object::CreditFacility,
            CreditFacilityAction::RecordInterest,
        )
        .await?;
    let new_accrual = credit_facility
        .start_interest_accrual(audit_info.clone())?
        .expect("Accrual start date is before facility expiry date");
    let accrual = interest_accrual_repo.create_in_op(db, new_accrual).await?;
    match jobs
        .create_and_spawn_at_in_op(
            db,
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

    ledger
        .activate_credit_facility(credit_facility_activation)
        .await?;

    Ok(())
}
