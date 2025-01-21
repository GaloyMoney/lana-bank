use crate::accounting_init::*;

pub(crate) async fn init(statements: &Statements) -> Result<StatementsInit, AccountingInitError> {
    Ok(StatementsInit)
}
