use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use chart_of_accounts::{new::AccountCode, ChartId};

use super::error::DepositConfigError;

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
#[builder(build_fn(error = "DepositConfigError"))]
pub struct DepositConfigValues {
    #[builder(setter(into))]
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_omnibus_parent_code: AccountCode,
}

impl DepositConfigValues {
    pub fn builder() -> DepositConfigValuesBuilder {
        DepositConfigValuesBuilder::default()
    }
}
