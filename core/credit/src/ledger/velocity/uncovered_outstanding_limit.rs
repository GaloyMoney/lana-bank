use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{velocity::*, *};

pub struct UncoveredOutstandingLimit;

const UNCOVERED_OUTSTANDING_LIMIT_ID: uuid::Uuid =
    uuid::uuid!("00000000-0000-0000-0000-000000000003");

impl UncoveredOutstandingLimit {
    #[record_error_severity]
    #[instrument(name = "ledger.uncovered_outstanding_limit.init", skip_all)]
    pub async fn init(
        ledger: &CalaLedger,
    ) -> Result<VelocityLimitId, crate::ledger::CreditLedgerError> {
        let limit = NewVelocityLimit::builder()
            .id(UNCOVERED_OUTSTANDING_LIMIT_ID)
            .name("Uncovered Outstanding Limit")
            .description("Prevent uncovered outstanding account from going negative")
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
                Ok(UNCOVERED_OUTSTANDING_LIMIT_ID.into())
            }
            Err(e) => Err(e.into()),
            Ok(limit) => Ok(limit.id()),
        }
    }
}
