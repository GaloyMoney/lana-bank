use serde::{Deserialize, Serialize};

use super::PublicCustomer;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated { entity: PublicCustomer },
    CustomerKycUpdated { entity: PublicCustomer },
    CustomerEmailUpdated { entity: PublicCustomer },
}
