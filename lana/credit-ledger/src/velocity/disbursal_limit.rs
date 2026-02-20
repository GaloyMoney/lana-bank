use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{velocity::*, *};

pub struct DisbursalLimit;

const DISBURSAL_LIMIT_ID: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000002");

impl DisbursalLimit {
    #[record_error_severity]
    #[instrument(name = "ledger.disbursal_limit.init", skip_all)]
    pub async fn init(
        ledger: &CalaLedger,
    ) -> Result<VelocityLimitId, core_credit::CreditLedgerError> {
        let limit = NewVelocityLimit::builder()
            .id(DISBURSAL_LIMIT_ID)
            .name("Disbursal Limit")
            .description("Limit for disbursals")
            .window(vec![])
            .limit(
                NewLimit::builder()
                    .balance(vec![
                        NewBalanceLimit::builder()
                            .layer("SETTLED")
                            .amount("decimal('0.0')")
                            .enforcement_direction("DEBIT")
                            .build()
                            .expect("balance limit"),
                    ])
                    .build()
                    .expect("limit"),
            )
            .build()
            .expect("velocity limit");

        match ledger.velocities().create_limit(limit).await {
            Err(cala_ledger::velocity::error::VelocityError::LimitIdAlreadyExists) => {
                Ok(DISBURSAL_LIMIT_ID.into())
            }
            Err(e) => Err(core_credit::CreditLedgerError::from_ledger(e)),
            Ok(limit) => Ok(limit.id()),
        }
    }
}
