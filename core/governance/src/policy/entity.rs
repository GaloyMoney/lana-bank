use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use super::rules::ApprovalRules;
use crate::{approval_process::NewApprovalProcess, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PolicyId")]
pub enum PolicyEvent {
    Initialized {
        id: PolicyId,
        process_type: ApprovalProcessType,
        rules: ApprovalRules,
        audit_info: AuditInfo,
    },
    ApprovalRulesUpdated {
        rules: ApprovalRules,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Policy {
    pub id: PolicyId,
    pub process_type: ApprovalProcessType,
    pub rules: ApprovalRules,
    pub(super) events: EntityEvents<PolicyEvent>,
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
        audit_info: AuditInfo,
    ) -> NewApprovalProcess {
        NewApprovalProcess::builder()
            .id(id)
            .target_ref(target_ref)
            .policy_id(self.id)
            .process_type(self.process_type.clone())
            .rules(self.rules)
            .audit_info(audit_info)
            .build()
            .expect("failed to build new approval process")
    }

    pub fn assign_committee(
        &mut self,
        committee_id: CommitteeId,
        threshold: usize,
        audit_info: AuditInfo,
    ) {
        self.rules = ApprovalRules::CommitteeThreshold {
            threshold,
            committee_id,
        };
        self.events.push(PolicyEvent::ApprovalRulesUpdated {
            rules: self.rules,
            audit_info,
        });
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
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
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
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use audit::{AuditEntryId, AuditInfo};

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn init_events() -> EntityEvents<PolicyEvent> {
        EntityEvents::init(
            PolicyId::new(),
            [PolicyEvent::Initialized {
                id: PolicyId::new(),
                process_type: ApprovalProcessType::new("test"),
                rules: ApprovalRules::SystemAutoApprove,
                audit_info: dummy_audit_info(),
            }],
        )
    }

    #[test]
    fn update_policy() {
        let mut policy = Policy::try_from_events(init_events()).unwrap();
        let committee_id = CommitteeId::new();
        let threshold = 1;
        let audit_info = dummy_audit_info();
        policy.assign_committee(committee_id, threshold, audit_info.clone());
        assert_eq!(policy.committee_id(), Some(committee_id));
        assert_eq!(
            policy.rules,
            ApprovalRules::CommitteeThreshold {
                threshold,
                committee_id
            }
        );
    }
}
