use std::{fmt::Display, str::FromStr};

use lana_ids::{DocumentId, ReportId, TermsTemplateId};

use authz::AllOrOne;
use core_access::CoreAccessObject;
use core_accounting::CoreAccountingObject;
use core_credit::CoreCreditObject;
use core_custody::CoreCustodyObject;
use core_customer::{CustomerId, CustomerObject};
use core_deposit::CoreDepositObject;
use dashboard::DashboardModuleObject;
use governance::GovernanceObject;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LanaObject {
    App(AppObject),
    Governance(GovernanceObject),
    Access(CoreAccessObject),
    Customer(CustomerObject),
    Accounting(CoreAccountingObject),
    Deposit(CoreDepositObject),
    Credit(CoreCreditObject),
    Custody(CoreCustodyObject),
    Dashboard(DashboardModuleObject),
}

impl From<AppObject> for LanaObject {
    fn from(object: AppObject) -> Self {
        LanaObject::App(object)
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

impl Display for LanaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", LanaObjectDiscriminants::from(self))?;
        use LanaObject::*;
        match self {
            App(action) => action.fmt(f),
            Governance(action) => action.fmt(f),
            Access(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            Accounting(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Credit(action) => action.fmt(f),
            Custody(action) => action.fmt(f),
            Dashboard(action) => action.fmt(f),
        }
    }
}

impl FromStr for LanaObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, object) = s.split_once('/').expect("missing colon");
        use LanaObjectDiscriminants::*;
        let res = match module.parse().expect("invalid module") {
            App => LanaObject::from(object.parse::<AppObject>()?),
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
        };
        Ok(res)
    }
}

es_entity::entity_id!(ApplicantId, AuditId);

pub type ApplicantAllOrOne = AllOrOne<ApplicantId>;
pub type TermsTemplateAllOrOne = AllOrOne<TermsTemplateId>;
pub type DocumentAllOrOne = AllOrOne<DocumentId>;
pub type ReportAllOrOne = AllOrOne<ReportId>;
pub type AuditAllOrOne = AllOrOne<AuditId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AppObject {
    Applicant(ApplicantAllOrOne),
    TermsTemplate(TermsTemplateAllOrOne),
    Document(DocumentAllOrOne),
    Report(ReportAllOrOne),
    Audit(AuditAllOrOne),
}

impl AppObject {
    pub const fn all_terms_templates() -> Self {
        Self::TermsTemplate(AllOrOne::All)
    }
    pub const fn terms_template(id: TermsTemplateId) -> Self {
        Self::TermsTemplate(AllOrOne::ById(id))
    }
    pub const fn all_documents() -> Self {
        Self::Document(AllOrOne::All)
    }
    pub const fn document(id: DocumentId) -> Self {
        Self::Document(AllOrOne::ById(id))
    }
    pub const fn all_reports() -> Self {
        Self::Report(AllOrOne::All)
    }
    pub const fn report(id: ReportId) -> Self {
        Self::Report(AllOrOne::ById(id))
    }
    pub const fn all_audits() -> Self {
        Self::Audit(AllOrOne::All)
    }
}

impl Display for AppObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = AppObjectDiscriminants::from(self);
        match self {
            Self::Applicant(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Self::TermsTemplate(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Self::Document(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Self::Report(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Self::Audit(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for AppObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use AppObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Applicant => {
                let obj_ref = id.parse().map_err(|_| "could not parse AppObject")?;
                Self::Applicant(obj_ref)
            }
            TermsTemplate => {
                let obj_ref = id.parse().map_err(|_| "could not parse AppObject")?;
                Self::TermsTemplate(obj_ref)
            }
            Document => {
                let obj_ref = id.parse().map_err(|_| "could not parse AppObject")?;
                Self::Document(obj_ref)
            }
            Report => {
                let obj_ref = id.parse().map_err(|_| "could not parse AppObject")?;
                Self::Report(obj_ref)
            }
            Audit => {
                let obj_ref = id.parse().map_err(|_| "could not parse AppObject")?;
                Self::Audit(obj_ref)
            }
        };

        Ok(res)
    }
}

pub type CustomerAllOrOne = AllOrOne<CustomerId>;

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
        // App
        // test_to_and_from_string(
        //     LanaObject::App(AppObject::Customer(AllOrOne::All)),
        //     "app/customer/*",
        // )?;

        // Governance
        test_to_and_from_string(
            LanaObject::Governance(GovernanceObject::Committee(AllOrOne::All)),
            "governance/committee/*",
        )?;

        Ok(())
    }
}
