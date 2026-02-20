#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod collateral_ledger;
mod collateral_templates;
mod constants;
mod credit_ledger;
mod credit_templates;
mod velocity;

pub use collateral_ledger::CollateralLedger;
pub use credit_ledger::CreditLedger;
