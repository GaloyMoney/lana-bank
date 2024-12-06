use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display, str::FromStr};

use authz::AllOrOne;
es_entity::entity_id! { ApprovalProcessId, CommitteeId, PolicyId, CommitteeMemberId }

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalProcessStatus {
    Approved,
    Denied,
    InProgress,
}

impl ApprovalProcessStatus {
    pub fn is_concluded(&self) -> bool {
        matches!(
            self,
            ApprovalProcessStatus::Approved | ApprovalProcessStatus::Denied
        )
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct ApprovalProcessType(Cow<'static, str>);
impl ApprovalProcessType {
    pub const fn new(job_type: &'static str) -> Self {
        ApprovalProcessType(Cow::Borrowed(job_type))
    }

    #[cfg(test)]
    pub(crate) fn from_owned(job_type: String) -> Self {
        ApprovalProcessType(Cow::Owned(job_type))
    }
}

impl Display for ApprovalProcessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum GovernanceAction {
    Committee(CommitteeAction),
    Policy(PolicyAction),
    ApprovalProcess(ApprovalProcessAction),
}

impl GovernanceAction {
    pub const COMMITTEE_CREATE: Self = GovernanceAction::Committee(CommitteeAction::Create);
    pub const COMMITTEE_READ: Self = GovernanceAction::Committee(CommitteeAction::Read);
    pub const COMMITTEE_LIST: Self = GovernanceAction::Committee(CommitteeAction::List);
    pub const COMMITTEE_ADD_MEMBER: Self = GovernanceAction::Committee(CommitteeAction::AddMember);
    pub const COMMITTEE_REMOVE_MEMBER: Self =
        GovernanceAction::Committee(CommitteeAction::RemoveMember);

    pub const POLICY_CREATE: Self = GovernanceAction::Policy(PolicyAction::Create);
    pub const POLICY_READ: Self = GovernanceAction::Policy(PolicyAction::Read);
    pub const POLICY_LIST: Self = GovernanceAction::Policy(PolicyAction::List);
    pub const POLICY_UPDATE_RULES: Self = GovernanceAction::Policy(PolicyAction::UpdatePolicyRules);

    pub const APPROVAL_PROCESS_CREATE: Self =
        GovernanceAction::ApprovalProcess(ApprovalProcessAction::Create);
    pub const APPROVAL_PROCESS_READ: Self =
        GovernanceAction::ApprovalProcess(ApprovalProcessAction::Read);
    pub const APPROVAL_PROCESS_LIST: Self =
        GovernanceAction::ApprovalProcess(ApprovalProcessAction::List);
    pub const APPROVAL_PROCESS_APPROVE: Self =
        GovernanceAction::ApprovalProcess(ApprovalProcessAction::Approve);
    pub const APPROVAL_PROCESS_DENY: Self =
        GovernanceAction::ApprovalProcess(ApprovalProcessAction::Deny);
    pub const APPROVAL_PROCESS_CONCLUDE: Self =
        GovernanceAction::ApprovalProcess(ApprovalProcessAction::Conclude);
}

impl Display for GovernanceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", GovernanceActionDiscriminants::from(self))?;
        use GovernanceAction::*;
        match self {
            Committee(action) => action.fmt(f),
            Policy(action) => action.fmt(f),
            ApprovalProcess(action) => action.fmt(f),
        }
    }
}

impl FromStr for GovernanceAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use GovernanceActionDiscriminants::*;
        let res = match entity.parse()? {
            Committee => GovernanceAction::from(action.parse::<CommitteeAction>()?),
            Policy => GovernanceAction::from(action.parse::<PolicyAction>()?),
            ApprovalProcess => GovernanceAction::from(action.parse::<ApprovalProcessAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CommitteeAction {
    Create,
    AddMember,
    RemoveMember,
    Read,
    List,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum PolicyAction {
    Create,
    Read,
    List,
    UpdatePolicyRules,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ApprovalProcessAction {
    Create,
    Read,
    List,
    Approve,
    Deny,
    Conclude,
}

pub type CommitteeAllOrOne = AllOrOne<CommitteeId>;
pub type PolicyAllOrOne = AllOrOne<PolicyId>;
pub type ApprovalProcessAllOrOne = AllOrOne<ApprovalProcessId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum GovernanceObject {
    Committee(CommitteeAllOrOne),
    Policy(PolicyAllOrOne),
    ApprovalProcess(ApprovalProcessAllOrOne),
}

impl GovernanceObject {
    pub fn all_committees() -> Self {
        GovernanceObject::Committee(AllOrOne::All)
    }

    pub fn committee(id: CommitteeId) -> Self {
        GovernanceObject::Committee(AllOrOne::ById(id))
    }

    pub fn all_policies() -> Self {
        GovernanceObject::Policy(AllOrOne::All)
    }

    pub fn policy(id: PolicyId) -> Self {
        GovernanceObject::Policy(AllOrOne::ById(id))
    }

    pub fn all_approval_processes() -> Self {
        GovernanceObject::ApprovalProcess(AllOrOne::All)
    }

    pub fn approval_process(id: ApprovalProcessId) -> Self {
        GovernanceObject::ApprovalProcess(AllOrOne::ById(id))
    }
}

impl Display for GovernanceObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = GovernanceObjectDiscriminants::from(self);
        use GovernanceObject::*;
        match self {
            Committee(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Policy(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            ApprovalProcess(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for GovernanceObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use GovernanceObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Committee => {
                let obj_ref = id.parse().map_err(|_| "could not parse GovernanceObject")?;
                GovernanceObject::Committee(obj_ref)
            }
            Policy => {
                let obj_ref = id.parse().map_err(|_| "could not parse GovernanceObject")?;
                GovernanceObject::Policy(obj_ref)
            }
            ApprovalProcess => {
                let obj_ref = id.parse().map_err(|_| "could not parse GovernanceObject")?;
                GovernanceObject::ApprovalProcess(obj_ref)
            }
        };
        Ok(res)
    }
}

impl From<CommitteeAction> for GovernanceAction {
    fn from(action: CommitteeAction) -> Self {
        GovernanceAction::Committee(action)
    }
}

impl From<PolicyAction> for GovernanceAction {
    fn from(action: PolicyAction) -> Self {
        GovernanceAction::Policy(action)
    }
}

impl From<ApprovalProcessAction> for GovernanceAction {
    fn from(action: ApprovalProcessAction) -> Self {
        GovernanceAction::ApprovalProcess(action)
    }
}
