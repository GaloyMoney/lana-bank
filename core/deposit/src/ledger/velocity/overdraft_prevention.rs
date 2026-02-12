use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{velocity::*, *};

use crate::ledger::error::*;

pub(crate) struct OverdraftPrevention;

const OVERDRAFT_PREVENTION_ID: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000001");

impl OverdraftPrevention {
    #[record_error_severity]
    #[instrument(name = "ledger.overdraft_prevention.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<VelocityLimitId, DepositLedgerError> {
        let limit = NewVelocityLimit::builder()
            .id(OVERDRAFT_PREVENTION_ID)
            .name("Overdraft Prevention")
            .description("Prevent overdraft on withdrawals")
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
                Ok(OVERDRAFT_PREVENTION_ID.into())
            }
            Err(e) => Err(e.into()),
            Ok(limit) => Ok(limit.id()),
        }
    }
}
