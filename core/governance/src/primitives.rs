pub use shared_primitives::{CommitteeId, UserId};

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum GovernanceAction {
    Committee(CommitteeAction),
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CommitteeAction {
    Create,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum GovernanceObject {
    Committee,
}

impl From<CommitteeAction> for GovernanceAction {
    fn from(action: CommitteeAction) -> Self {
        GovernanceAction::Committee(action)
    }
}

pub(crate) fn g_action(a: impl Into<GovernanceAction>) -> GovernanceAction {
    a.into()
}
