use tracing::instrument;

use crate::audit::Audit;

use super::*;

#[instrument(name = "credit_facility.activate.execute", skip_all)]
pub(super) async fn execute(
    credit_facility: &mut CreditFacility,
    db: &mut es_entity::DbOp<'_>,
    ledger: &Ledger,
    audit: &Audit,
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

    let now = crate::time::now();

    if let es_entity::Idempotent::Executed(next_incurrance_period) =
        credit_facility.activate(credit_facility_activation.clone(), now, audit_info)
    {
        match jobs
            .create_and_spawn_at_in_op(
                db,
                credit_facility.id,
                interest::CreditFacilityJobConfig {
                    credit_facility_id: credit_facility.id,
                },
                next_incurrance_period.end,
            )
            .await
        {
            Ok(_) | Err(JobError::DuplicateId) => (),
            Err(err) => Err(err)?,
        };

        ledger
            .activate_credit_facility(credit_facility_activation)
            .await?;
    }

    Ok(())
}
