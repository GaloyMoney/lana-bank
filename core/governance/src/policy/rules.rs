use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::primitives::CommitteeId;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApprovalRules {
    #[default]
    SystemAutoApprove,
    Committee {
        committee_id: CommitteeId,
    },
}

impl ApprovalRules {
    pub fn committee_id(&self) -> Option<CommitteeId> {
        match self {
            ApprovalRules::Committee { committee_id } => Some(*committee_id),
            ApprovalRules::SystemAutoApprove => None,
        }
    }

    pub fn is_approved_or_denied<Id: Eq + std::hash::Hash>(
        &self,
        eligible_members: &HashSet<Id>,
        approving_members: &HashSet<Id>,
        denying_members: &HashSet<Id>,
    ) -> Option<bool> {
        if !denying_members.is_empty() {
            return Some(false);
        }
        match self {
            ApprovalRules::SystemAutoApprove => Some(true),
            ApprovalRules::Committee { .. } if eligible_members.is_empty() => Some(false),
            ApprovalRules::Committee { .. }
                if eligible_members.intersection(approving_members).count()
                    == eligible_members.len() =>
            {
                Some(true)
            }
            ApprovalRules::Committee { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_set(ids: &[u32]) -> HashSet<u32> {
        ids.iter().copied().collect()
    }

    #[test]
    fn test_committee_all_approved() {
        let rules = ApprovalRules::Committee {
            committee_id: CommitteeId::new(),
        };

        let eligible = make_set(&[1, 2, 3]);
        let approving = make_set(&[1, 2, 3]);
        let denying = HashSet::new();

        let result = rules.is_approved_or_denied(&eligible, &approving, &denying);

        assert_eq!(
            result,
            Some(true),
            "Should be approved when all eligible members approve"
        );
    }

    #[test]
    fn test_committee_denial() {
        let rules = ApprovalRules::Committee {
            committee_id: CommitteeId::new(),
        };

        let eligible = make_set(&[1, 2, 3]);
        let approving = make_set(&[2, 3]);
        let denying = make_set(&[1]);

        let result = rules.is_approved_or_denied(&eligible, &approving, &denying);

        assert_eq!(
            result,
            Some(false),
            "Should be denied as soon as 1 denial exists"
        );
    }

    #[test]
    fn test_committee_pending() {
        let rules = ApprovalRules::Committee {
            committee_id: CommitteeId::new(),
        };

        let eligible = make_set(&[1, 2, 3]);
        let approving = make_set(&[1, 2]);
        let denying = HashSet::new();

        let result = rules.is_approved_or_denied(&eligible, &approving, &denying);

        assert_eq!(
            result, None,
            "Should be pending when not all members have approved"
        );
    }

    #[test]
    fn test_automatic() {
        let rules = ApprovalRules::SystemAutoApprove;

        assert_eq!(
            rules.is_approved_or_denied(&make_set(&[1, 2, 3]), &HashSet::new(), &HashSet::new()),
            Some(true),
            "Automatic rules should always approve regardless of inputs"
        );
    }

    #[test]
    fn test_empty_eligible_denied() {
        let rules = ApprovalRules::Committee {
            committee_id: CommitteeId::new(),
        };

        let empty: HashSet<u32> = HashSet::new();
        assert_eq!(
            rules.is_approved_or_denied(&empty, &empty, &empty),
            Some(false),
            "Empty eligible set should result in denial"
        );
    }
}
