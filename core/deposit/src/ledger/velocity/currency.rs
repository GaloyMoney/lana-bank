use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{velocity::*, *};

use crate::ledger::error::*;

pub struct CurrencyGuard;

const CURRENCY_GUARD_ID: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000002");

impl CurrencyGuard {
    #[record_error_severity]
    #[instrument(name = "ledger.currency_guard.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<VelocityLimitId, DepositLedgerError> {
        let limit = NewVelocityLimit::builder()
            .id(CURRENCY_GUARD_ID)
            .name("Currency Guard")
            .description("Reject postings in currencies other than the account currency")
            .window(vec![])
            .condition("params.account_currency != context.vars.entry.currency")
            .params(vec![
                NewParamDefinition::builder()
                    .name("account_currency")
                    .r#type(ParamDataType::String)
                    .build()
                    .expect("account currency param"),
            ])
            .limit(
                NewLimit::builder()
                    .balance(vec![
                        NewBalanceLimit::builder()
                            .layer("context.vars.entry.layer")
                            .amount("decimal('0')")
                            .enforcement_direction("context.vars.entry.direction")
                            .build()
                            .expect("balance limit"),
                    ])
                    .build()
                    .expect("limit"),
            )
            .build()
            .expect("velocity limit");

        match ledger.velocities().create_limit(limit).await {
            Err(cala_ledger::velocity::error::VelocityError::LimitIdAlreadyExists(_)) => {
                Ok(CURRENCY_GUARD_ID.into())
            }
            Err(e) => Err(e.into()),
            Ok(limit) => Ok(limit.id()),
        }
    }
}
