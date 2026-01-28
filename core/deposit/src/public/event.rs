use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{PublicDeposit, PublicDepositAccount, PublicWithdrawal};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreDepositEvent {
    DepositAccountCreated { entity: PublicDepositAccount },
    DepositInitialized { entity: PublicDeposit },
    WithdrawalConfirmed { entity: PublicWithdrawal },
    DepositReverted { entity: PublicDeposit },
}
