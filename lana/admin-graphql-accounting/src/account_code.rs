use async_graphql::scalar;
use serde::{Deserialize, Serialize};

use lana_app::accounting::{
    AccountCode as DomainAccountCode, AccountCodeSection as DomainAccountCodeSection,
};

scalar!(AccountCode);
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AccountCode(String);

impl From<&DomainAccountCode> for AccountCode {
    fn from(value: &DomainAccountCode) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<AccountCode> for DomainAccountCode {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(value: AccountCode) -> Result<Self, Self::Error> {
        Ok(value.0.parse()?)
    }
}

impl TryFrom<AccountCode> for Vec<DomainAccountCodeSection> {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(value: AccountCode) -> Result<Self, Self::Error> {
        Ok(Self::from(DomainAccountCode::try_from(value)?))
    }
}
