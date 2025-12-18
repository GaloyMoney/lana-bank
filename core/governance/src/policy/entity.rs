use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::{error::PolicyError, rules::ApprovalRules};
use crate::{approval_process::NewApprovalProcess, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PolicyId")]
pub enum PolicyEvent {
    Initialized {
        id: PolicyId,
        process_type: ApprovalProcessType,
        rules: ApprovalRules,
    },
    ApprovalRulesUpdated {
        rules: ApprovalRules,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Policy {
    pub id: PolicyId,
    pub process_type: ApprovalProcessType,
    pub rules: ApprovalRules,
    events: EntityEvents<PolicyEvent>,
}

impl Policy {
    pub fn committee_id(&self) -> Option<CommitteeId> {
        self.rules.committee_id()
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for policy")
    }

    pub(crate) fn spawn_process(
        &self,
        id: ApprovalProcessId,
        target_ref: String,
    ) -> NewApprovalProcess {
        NewApprovalProcess::builder()
            .id(id)
            .target_ref(target_ref)
            .policy_id(self.id)
            .process_type(self.process_type.clone())
            .rules(self.rules)
            .build()
            .expect("failed to build new approval process")
    }

    pub fn assign_committee(
        &mut self,
        committee_id: CommitteeId,
        n_committee_members: usize,
        threshold: usize,
    ) -> Result<Idempotent<()>, PolicyError> {
        if threshold < 1 {
            return Err(PolicyError::PolicyThresholdTooLow(committee_id, threshold));
        }
        let rules = ApprovalRules::CommitteeThreshold {
            committee_id,
            threshold,
        };

        if self.rules == rules {
            return Ok(Idempotent::AlreadyApplied);
        }

        if threshold > n_committee_members {
            return Err(PolicyError::PolicyThresholdTooHigh(committee_id, threshold));
        }

        self.rules = ApprovalRules::CommitteeThreshold {
            threshold,
            committee_id,
        };

        self.events
            .push(PolicyEvent::ApprovalRulesUpdated { rules: self.rules });
        Ok(Idempotent::Executed(()))
    }
}

impl TryFromEvents<PolicyEvent> for Policy {
    fn try_from_events(events: EntityEvents<PolicyEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PolicyBuilder::default();
        for event in events.iter_all() {
            match event {
                PolicyEvent::Initialized {
                    id,
                    process_type,
                    rules,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .process_type(process_type.clone())
                        .rules(*rules)
                }
                PolicyEvent::ApprovalRulesUpdated { rules, .. } => builder = builder.rules(*rules),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewPolicy {
    #[builder(setter(into))]
    pub(super) id: PolicyId,
    pub(super) process_type: ApprovalProcessType,
    pub(super) rules: ApprovalRules,
}

impl NewPolicy {
    pub fn builder() -> NewPolicyBuilder {
        NewPolicyBuilder::default()
    }

    pub fn committee_id(&self) -> Option<CommitteeId> {
        self.rules.committee_id()
    }
}

impl IntoEvents<PolicyEvent> for NewPolicy {
    fn into_events(self) -> EntityEvents<PolicyEvent> {
        EntityEvents::init(
            self.id,
            [PolicyEvent::Initialized {
                id: self.id,
                process_type: self.process_type,
                rules: self.rules,
            }],
        )
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn init_events() -> EntityEvents<PolicyEvent> {
        EntityEvents::init(
            PolicyId::new(),
            [PolicyEvent::Initialized {
                id: PolicyId::new(),
                process_type: ApprovalProcessType::new("test"),
                rules: ApprovalRules::SystemAutoApprove,
            }],
        )
    }

    #[test]
    fn update_policy() {
        let mut policy = Policy::try_from_events(init_events()).unwrap();
        let committee_id = CommitteeId::new();
        let n_committee_members = 1;
        let threshold = 1;
        let _ = policy
            .assign_committee(committee_id, n_committee_members, threshold)
            .unwrap();
        assert_eq!(policy.committee_id(), Some(committee_id));
        assert_eq!(
            policy.rules,
            ApprovalRules::CommitteeThreshold {
                threshold,
                committee_id
            }
        );
    }

    #[test]
    fn error_when_threshold_greater_than_n_committee_members() {
        let mut policy = Policy::try_from_events(init_events()).unwrap();
        let committee_id = CommitteeId::new();
        let n_committee_members = 1;
        let threshold = 2;
        let res = policy.assign_committee(committee_id, n_committee_members, threshold);

        assert!(matches!(
            res,
            Err(PolicyError::PolicyThresholdTooHigh(_, _))
        ));
    }

    #[test]
    fn error_when_threshold_less_than_one() {
        let mut policy = Policy::try_from_events(init_events()).unwrap();
        let committee_id = CommitteeId::new();
        let n_committee_members = 1;
        let threshold = 0;
        let res = policy.assign_committee(committee_id, n_committee_members, threshold);

        assert!(matches!(res, Err(PolicyError::PolicyThresholdTooLow(_, _))));
    }
}
