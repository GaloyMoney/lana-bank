use std::{fmt::Display, str::FromStr};

use authz::action_description::*;

use contract_creation::ContractModuleAction;
use core_access::CoreAccessAction;
use core_accounting::CoreAccountingAction;
use core_audit::AuditAction;
use core_credit::CoreCreditAction;
use core_custody::CoreCustodyAction;
use core_customer::CoreCustomerAction;
use core_deposit::CoreDepositAction;
use core_report::CoreReportAction;
use dashboard::DashboardModuleAction;
use governance::GovernanceAction;

pub const PERMISSION_SET_APP_VIEWER: &str = "app_viewer";
pub const PERMISSION_SET_APP_WRITER: &str = "app_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LanaAction {
    Audit(AuditAction),
    Governance(GovernanceAction),
    Access(CoreAccessAction),
    Customer(CoreCustomerAction),
    Accounting(CoreAccountingAction),
    Dashboard(DashboardModuleAction),
    Deposit(CoreDepositAction),
    Credit(CoreCreditAction),
    Custody(CoreCustodyAction),
    Report(CoreReportAction),
    Contract(ContractModuleAction),
}

impl LanaAction {
    /// Returns description of all actions defined in `LanaAction`.
    pub fn action_descriptions() -> Vec<ActionDescription<FullPath>> {
        use LanaActionDiscriminants::*;

        fn flatten<Entity: Display + Copy>(
            module: &LanaActionDiscriminants,
            entity_actions: Vec<(Entity, Vec<ActionDescription<NoPath>>)>,
        ) -> Vec<ActionDescription<FullPath>> {
            entity_actions
                .into_iter()
                .flat_map(|(entity, actions)| {
                    actions
                        .into_iter()
                        .map(move |action| action.inject_path(module, entity))
                })
                .collect()
        }

        let mut result = vec![];

        for module in <LanaActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match module {
                Audit => flatten(module, AuditAction::entities()),
                Governance => flatten(module, GovernanceAction::entities()),
                Access => flatten(module, CoreAccessAction::entities()),
                Customer => flatten(module, CoreCustomerAction::entities()),
                Accounting => flatten(module, CoreAccountingAction::entities()),
                Dashboard => flatten(module, DashboardModuleAction::entities()),
                Deposit => flatten(module, CoreDepositAction::entities()),
                Credit => flatten(module, CoreCreditAction::entities()),
                Custody => flatten(module, CoreCustodyAction::entities()),
                Report => flatten(module, CoreReportAction::entities()),
                Contract => flatten(module, ContractModuleAction::entities()),
            };

            result.extend(actions);
        }

        result
    }
}

impl From<AuditAction> for LanaAction {
    fn from(action: AuditAction) -> Self {
        LanaAction::Audit(action)
    }
}
impl From<DashboardModuleAction> for LanaAction {
    fn from(action: DashboardModuleAction) -> Self {
        LanaAction::Dashboard(action)
    }
}
impl From<GovernanceAction> for LanaAction {
    fn from(action: GovernanceAction) -> Self {
        LanaAction::Governance(action)
    }
}
impl From<CoreAccessAction> for LanaAction {
    fn from(action: CoreAccessAction) -> Self {
        LanaAction::Access(action)
    }
}
impl From<CoreCustomerAction> for LanaAction {
    fn from(action: CoreCustomerAction) -> Self {
        LanaAction::Customer(action)
    }
}
impl From<CoreAccountingAction> for LanaAction {
    fn from(action: CoreAccountingAction) -> Self {
        LanaAction::Accounting(action)
    }
}
impl From<CoreDepositAction> for LanaAction {
    fn from(action: CoreDepositAction) -> Self {
        LanaAction::Deposit(action)
    }
}
impl From<CoreCreditAction> for LanaAction {
    fn from(action: CoreCreditAction) -> Self {
        LanaAction::Credit(action)
    }
}
impl From<CoreCustodyAction> for LanaAction {
    fn from(action: CoreCustodyAction) -> Self {
        LanaAction::Custody(action)
    }
}
impl From<CoreReportAction> for LanaAction {
    fn from(action: CoreReportAction) -> Self {
        LanaAction::Report(action)
    }
}
impl From<ContractModuleAction> for LanaAction {
    fn from(action: ContractModuleAction) -> Self {
        LanaAction::Contract(action)
    }
}

impl Display for LanaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", LanaActionDiscriminants::from(self))?;
        use LanaAction::*;
        match self {
            Audit(action) => action.fmt(f),
            Governance(action) => action.fmt(f),
            Access(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            Dashboard(action) => action.fmt(f),
            Accounting(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Credit(action) => action.fmt(f),
            Custody(action) => action.fmt(f),
            Report(action) => action.fmt(f),
            Contract(action) => action.fmt(f),
        }
    }
}

impl FromStr for LanaAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, action) = s.split_once(':').expect("missing colon");
        use LanaActionDiscriminants::*;
        let res = match module.parse()? {
            Audit => LanaAction::from(action.parse::<AuditAction>()?),
            Governance => LanaAction::from(action.parse::<GovernanceAction>()?),
            Access => LanaAction::from(action.parse::<CoreAccessAction>()?),
            Customer => LanaAction::from(action.parse::<CoreCustomerAction>()?),
            Dashboard => LanaAction::from(action.parse::<DashboardModuleAction>()?),
            Accounting => LanaAction::from(action.parse::<CoreAccountingAction>()?),
            Deposit => LanaAction::from(action.parse::<CoreDepositAction>()?),
            Credit => LanaAction::from(action.parse::<CoreCreditAction>()?),
            Custody => LanaAction::from(action.parse::<CoreCustodyAction>()?),
            Report => LanaAction::from(action.parse::<CoreReportAction>()?),
            Contract => LanaAction::from(action.parse::<ContractModuleAction>()?),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_to_and_from_string(action: LanaAction, result: &str) -> anyhow::Result<()> {
        let action_str = action.to_string();
        assert_eq!(&action_str, result);

        let parsed_action: LanaAction = action_str.parse()?;
        assert_eq!(parsed_action, action);

        Ok(())
    }

    #[test]
    fn action_serialization() -> anyhow::Result<()> {
        test_to_and_from_string(
            LanaAction::Audit(AuditAction::from(core_audit::AuditEntityAction::List)),
            "audit:audit:list",
        )?;

        test_to_and_from_string(
            LanaAction::Report(CoreReportAction::Report(
                core_report::ReportEntityAction::Generate,
            )),
            "report:report:generate",
        )?;
        Ok(())
    }
}
