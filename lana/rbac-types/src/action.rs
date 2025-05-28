use std::{fmt::Display, str::FromStr};

use authz::action_description::*;

use core_access::CoreAccessAction;
use core_accounting::CoreAccountingAction;
use core_credit::CoreCreditAction;
use core_customer::CoreCustomerAction;
use core_deposit::CoreDepositAction;
use dashboard::DashboardModuleAction;
use governance::GovernanceAction;

pub const PERMISSION_SET_APP_READER: &str = "app_reader";
pub const PERMISSION_SET_APP_WRITER: &str = "app_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LanaAction {
    App(AppAction),
    Governance(GovernanceAction),
    Access(CoreAccessAction),
    Customer(CoreCustomerAction),
    Accounting(CoreAccountingAction),
    Dashboard(DashboardModuleAction),
    Deposit(CoreDepositAction),
    Credit(CoreCreditAction),
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
                App => flatten(module, AppAction::entities()),
                Governance => flatten(module, GovernanceAction::entities()),
                Access => flatten(module, CoreAccessAction::entities()),
                Customer => flatten(module, CoreCustomerAction::entities()),
                Accounting => flatten(module, CoreAccountingAction::entities()),
                Dashboard => flatten(module, DashboardModuleAction::entities()),
                Deposit => flatten(module, CoreDepositAction::entities()),
                Credit => flatten(module, CoreCreditAction::entities()),
            };

            result.extend(actions);
        }

        result
    }
}

impl From<AppAction> for LanaAction {
    fn from(action: AppAction) -> Self {
        LanaAction::App(action)
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

impl Display for LanaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", LanaActionDiscriminants::from(self))?;
        use LanaAction::*;
        match self {
            App(action) => action.fmt(f),
            Governance(action) => action.fmt(f),
            Access(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            Dashboard(action) => action.fmt(f),
            Accounting(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Credit(action) => action.fmt(f),
        }
    }
}

impl FromStr for LanaAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, action) = s.split_once(':').expect("missing colon");
        use LanaActionDiscriminants::*;
        let res = match module.parse()? {
            App => LanaAction::from(action.parse::<AppAction>()?),
            Governance => LanaAction::from(action.parse::<GovernanceAction>()?),
            Access => LanaAction::from(action.parse::<CoreAccessAction>()?),
            Customer => LanaAction::from(action.parse::<CoreCustomerAction>()?),
            Dashboard => LanaAction::from(action.parse::<DashboardModuleAction>()?),
            Accounting => LanaAction::from(action.parse::<CoreAccountingAction>()?),
            Deposit => LanaAction::from(action.parse::<CoreDepositAction>()?),
            Credit => LanaAction::from(action.parse::<CoreCreditAction>()?),
        };
        Ok(res)
    }
}

macro_rules! impl_trivial_action {
    ($from_type:ty, $variant:ident) => {
        impl From<$from_type> for AppAction {
            fn from(action: $from_type) -> Self {
                AppAction::$variant(action)
            }
        }

        impl From<$from_type> for LanaAction {
            fn from(action: $from_type) -> Self {
                LanaAction::App(AppAction::$variant(action))
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AppAction {
    TermsTemplate(TermsTemplateAction),
    Report(ReportAction),
    Audit(AuditAction),
    Document(DocumentAction),
}

impl AppAction {
    pub fn entities() -> Vec<(AppActionDiscriminants, Vec<ActionDescription<NoPath>>)> {
        use AppActionDiscriminants::*;

        let mut result = vec![];

        for entity in <AppActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                TermsTemplate => TermsTemplateAction::describe(),
                Report => ReportAction::describe(),
                Audit => AuditAction::describe(),
                Document => DocumentAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

impl Display for AppAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", AppActionDiscriminants::from(self))?;
        use AppAction::*;
        match self {
            TermsTemplate(action) => action.fmt(f),
            Report(action) => action.fmt(f),
            Audit(action) => action.fmt(f),
            Document(action) => action.fmt(f),
        }
    }
}

impl FromStr for AppAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use AppActionDiscriminants::*;
        let res = match entity.parse()? {
            TermsTemplate => AppAction::from(action.parse::<TermsTemplateAction>()?),
            Report => AppAction::from(action.parse::<ReportAction>()?),
            Audit => AppAction::from(action.parse::<AuditAction>()?),
            Document => AppAction::from(action.parse::<DocumentAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum TermsTemplateAction {
    Read,
    Update,
    Create,
    List,
}

impl TermsTemplateAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        use TermsTemplateAction::*;

        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Read => ActionDescription::new(variant, &[PERMISSION_SET_APP_READER]),
                Update => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
                Create => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
                List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_WRITER, PERMISSION_SET_APP_READER],
                ),
            };
            res.push(action_description);
        }

        res
    }
}

impl_trivial_action!(TermsTemplateAction, TermsTemplate);

#[derive(Clone, PartialEq, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum AuditAction {
    List,
}

impl AuditAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_READER, PERMISSION_SET_APP_WRITER],
                ),
            };
            res.push(action_description);
        }

        res
    }
}

impl_trivial_action!(AuditAction, Audit);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DocumentAction {
    Create,
    Read,
    List,
    GenerateDownloadLink,
    Delete,
    Archive,
}

impl DocumentAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_READER, PERMISSION_SET_APP_WRITER],
                ),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_READER, PERMISSION_SET_APP_WRITER],
                ),
                Self::GenerateDownloadLink => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_READER, PERMISSION_SET_APP_WRITER],
                ),
                Self::Delete => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
                Self::Archive => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

impl_trivial_action!(DocumentAction, Document);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportAction {
    Read,
    List,
    Create,
    Upload,
    GenerateDownloadLink,
}

impl ReportAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_READER, PERMISSION_SET_APP_WRITER],
                ),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_APP_READER, PERMISSION_SET_APP_WRITER],
                ),
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
                Self::Upload => ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER]),
                Self::GenerateDownloadLink => {
                    ActionDescription::new(variant, &[PERMISSION_SET_APP_WRITER])
                }
            };
            res.push(action_description);
        }

        res
    }
}

impl_trivial_action!(ReportAction, Report);

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
        // Report
        test_to_and_from_string(
            LanaAction::App(AppAction::Report(ReportAction::List)),
            "app:report:list",
        )?;
        Ok(())
    }
}
