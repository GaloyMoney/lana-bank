use std::{fmt::Display, str::FromStr};

use contract_creation::ContractModuleObject;
use core_access::CoreAccessObject;
use core_accounting::CoreAccountingObject;
use core_audit::AuditObject;
use core_credit::CoreCreditObject;
use core_custody::CoreCustodyObject;
use core_customer::CustomerObject;
use core_deposit::CoreDepositObject;
use core_report::ReportObject;
use dashboard::DashboardModuleObject;
use governance::GovernanceObject;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LanaObject {
    Audit(AuditObject),
    Governance(GovernanceObject),
    Access(CoreAccessObject),
    Customer(CustomerObject),
    Accounting(CoreAccountingObject),
    Deposit(CoreDepositObject),
    Credit(CoreCreditObject),
    Custody(CoreCustodyObject),
    Dashboard(DashboardModuleObject),
    Report(ReportObject),
    Contract(ContractModuleObject),
}

impl From<AuditObject> for LanaObject {
    fn from(object: AuditObject) -> Self {
        LanaObject::Audit(object)
    }
}
impl From<DashboardModuleObject> for LanaObject {
    fn from(object: DashboardModuleObject) -> Self {
        LanaObject::Dashboard(object)
    }
}
impl From<GovernanceObject> for LanaObject {
    fn from(action: GovernanceObject) -> Self {
        LanaObject::Governance(action)
    }
}
impl From<CoreAccessObject> for LanaObject {
    fn from(action: CoreAccessObject) -> Self {
        LanaObject::Access(action)
    }
}
impl From<CustomerObject> for LanaObject {
    fn from(action: CustomerObject) -> Self {
        LanaObject::Customer(action)
    }
}
impl From<CoreAccountingObject> for LanaObject {
    fn from(object: CoreAccountingObject) -> Self {
        LanaObject::Accounting(object)
    }
}
impl From<CoreDepositObject> for LanaObject {
    fn from(object: CoreDepositObject) -> Self {
        LanaObject::Deposit(object)
    }
}
impl From<CoreCustodyObject> for LanaObject {
    fn from(object: CoreCustodyObject) -> Self {
        LanaObject::Custody(object)
    }
}
impl From<CoreCreditObject> for LanaObject {
    fn from(object: CoreCreditObject) -> Self {
        LanaObject::Credit(object)
    }
}

impl From<ReportObject> for LanaObject {
    fn from(object: ReportObject) -> Self {
        LanaObject::Report(object)
    }
}

impl From<ContractModuleObject> for LanaObject {
    fn from(object: ContractModuleObject) -> Self {
        LanaObject::Contract(object)
    }
}

impl Display for LanaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", LanaObjectDiscriminants::from(self))?;
        use LanaObject::*;
        match self {
            Audit(object) => object.fmt(f),
            Governance(object) => object.fmt(f),
            Access(object) => object.fmt(f),
            Customer(object) => object.fmt(f),
            Accounting(object) => object.fmt(f),
            Deposit(object) => object.fmt(f),
            Credit(object) => object.fmt(f),
            Custody(object) => object.fmt(f),
            Dashboard(object) => object.fmt(f),
            Report(object) => object.fmt(f),
            Contract(object) => object.fmt(f),
        }
    }
}

impl FromStr for LanaObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, object) = s.split_once('/').expect("missing colon");
        use LanaObjectDiscriminants::*;
        let res = match module.parse().expect("invalid module") {
            Audit => LanaObject::from(object.parse::<AuditObject>()?),
            Governance => LanaObject::from(object.parse::<GovernanceObject>()?),
            Access => LanaObject::from(object.parse::<CoreAccessObject>()?),
            Customer => LanaObject::from(object.parse::<CustomerObject>()?),
            Accounting => LanaObject::from(object.parse::<CoreAccountingObject>()?),
            Deposit => LanaObject::from(object.parse::<CoreDepositObject>()?),
            Credit => LanaObject::from(object.parse::<CoreCreditObject>()?),
            Custody => LanaObject::from(object.parse::<CoreCustodyObject>()?),
            Dashboard => LanaObject::from(
                object
                    .parse::<DashboardModuleObject>()
                    .map_err(|_| "could not parse DashboardModuleObject")?,
            ),
            Report => LanaObject::from(object.parse::<ReportObject>()?),
            Contract => LanaObject::from(
                object
                    .parse::<ContractModuleObject>()
                    .map_err(|_| "could not parse ContractModuleObject")?,
            ),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_to_and_from_string(action: LanaObject, result: &str) -> anyhow::Result<()> {
        let action_str = action.to_string();
        assert_eq!(&action_str, result);

        let parsed_action: LanaObject = action_str.parse().expect("could not parse action");
        assert_eq!(parsed_action, action);

        Ok(())
    }

    #[test]
    fn action_serialization() -> anyhow::Result<()> {
        // Governance
        test_to_and_from_string(
            LanaObject::Governance(GovernanceObject::Committee(AllOrOne::All)),
            "governance/committee/*",
        )?;

        test_to_and_from_string(
            LanaObject::Audit(AuditObject::all_audits()),
            "audit/audit/*",
        )?;

        Ok(())
    }
}
