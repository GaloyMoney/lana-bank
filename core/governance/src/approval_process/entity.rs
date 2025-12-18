use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use es_entity::*;

use crate::{policy::ApprovalRules, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ApprovalProcessId")]
pub enum ApprovalProcessEvent {
    Initialized {
        id: ApprovalProcessId,
        policy_id: PolicyId,
        process_type: ApprovalProcessType,
        rules: ApprovalRules,
        target_ref: String,
    },
    Approved {
        approver_id: CommitteeMemberId,
    },
    Denied {
        denier_id: CommitteeMemberId,
        reason: String,
    },
    Concluded {
        approved: bool,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ApprovalProcess {
    pub id: ApprovalProcessId,
    pub process_type: ApprovalProcessType,
    pub policy_id: PolicyId,
    pub rules: ApprovalRules,
    events: EntityEvents<ApprovalProcessEvent>,
}

impl ApprovalProcess {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for committee")
    }

    pub fn denied_reason(&self) -> Option<&str> {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                ApprovalProcessEvent::Denied { reason, .. } => Some(reason.as_str()),
                _ => None,
            })
            .next()
    }

    pub fn member_voted_at(
        &self,
        member_id: CommitteeMemberId,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        self.events
            .iter_persisted()
            .filter_map(|event| match event.event {
                ApprovalProcessEvent::Approved { approver_id, .. } if approver_id == member_id => {
                    Some(event.recorded_at)
                }
                ApprovalProcessEvent::Denied { denier_id, .. } if denier_id == member_id => {
                    Some(event.recorded_at)
                }
                _ => None,
            })
            .next()
    }
    pub fn target_ref(&self) -> &str {
        match self.events.iter_all().next().expect("No events") {
            ApprovalProcessEvent::Initialized { target_ref, .. } => target_ref,
            _ => {
                panic!("No events")
            }
        }
    }

    pub fn committee_id(&self) -> Option<CommitteeId> {
        self.rules.committee_id()
    }

    pub fn can_member_vote(
        &self,
        member_id: CommitteeMemberId,
        eligible: HashSet<CommitteeMemberId>,
    ) -> bool {
        eligible.contains(&member_id)
            && !self.approvers().contains(&member_id)
            && !self.deniers().contains(&member_id)
    }

    pub(crate) fn check_concluded(
        &mut self,
        eligible: HashSet<CommitteeMemberId>,
    ) -> Idempotent<(bool, Option<String>)> {
        idempotency_guard!(
            self.events.iter_all(),
            ApprovalProcessEvent::Concluded { .. },
        );
        if let Some(approved) =
            self.rules
                .is_approved_or_denied(&eligible, &self.approvers(), &self.deniers())
        {
            let reason = self
                .events
                .iter_all()
                .filter_map(|event| match event {
                    ApprovalProcessEvent::Denied { reason, .. } => Some(reason.clone()),
                    _ => None,
                })
                .next();

            self.events
                .push(ApprovalProcessEvent::Concluded { approved });
            return Idempotent::Executed((approved, reason));
        }
        Idempotent::AlreadyApplied
    }

    pub fn status(&self) -> ApprovalProcessStatus {
        for event in self.events.iter_all().rev() {
            match event {
                ApprovalProcessEvent::Concluded { approved: true, .. } => {
                    return ApprovalProcessStatus::Approved;
                }
                ApprovalProcessEvent::Concluded {
                    approved: false, ..
                } => return ApprovalProcessStatus::Denied,
                _ => {}
            }
        }
        ApprovalProcessStatus::InProgress
    }

    pub(crate) fn approve(
        &mut self,
        eligible_members: &HashSet<CommitteeMemberId>,
        approver_id: CommitteeMemberId,
    ) -> Idempotent<()> {
        use ApprovalProcessEvent::*;
        idempotency_guard!(
            self.events.iter_all(),
            Concluded {..},
            Approved {approver_id: id, ..} | Denied {denier_id: id,..} if id == &approver_id,
        );

        if !eligible_members.contains(&approver_id) {
            return Idempotent::AlreadyApplied;
        }

        self.events
            .push(ApprovalProcessEvent::Approved { approver_id });

        Idempotent::Executed(())
    }

    pub(crate) fn deny(
        &mut self,
        eligible_members: &HashSet<CommitteeMemberId>,
        denier_id: CommitteeMemberId,
        reason: String,
    ) -> Idempotent<()> {
        use ApprovalProcessEvent::*;
        idempotency_guard!(
            self.events.iter_all(),
            Concluded {..},
            Approved {approver_id: id, ..} | Denied {denier_id: id,..} if id == &denier_id,
        );

        if !eligible_members.contains(&denier_id) {
            return Idempotent::AlreadyApplied;
        }

        self.events
            .push(ApprovalProcessEvent::Denied { denier_id, reason });

        Idempotent::Executed(())
    }

    pub fn approvers(&self) -> HashSet<CommitteeMemberId> {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                ApprovalProcessEvent::Approved { approver_id, .. } => Some(*approver_id),
                _ => None,
            })
            .collect()
    }

    pub fn deniers(&self) -> HashSet<CommitteeMemberId> {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                ApprovalProcessEvent::Denied { denier_id, .. } => Some(*denier_id),
                _ => None,
            })
            .collect()
    }
}

impl TryFromEvents<ApprovalProcessEvent> for ApprovalProcess {
    fn try_from_events(events: EntityEvents<ApprovalProcessEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ApprovalProcessBuilder::default();
        for event in events.iter_all() {
            match event {
                ApprovalProcessEvent::Initialized {
                    id,
                    process_type,
                    policy_id,
                    rules,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .process_type(process_type.clone())
                        .policy_id(*policy_id)
                        .rules(*rules);
                }
                ApprovalProcessEvent::Approved { .. } => {}
                ApprovalProcessEvent::Denied { .. } => {}
                ApprovalProcessEvent::Concluded { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewApprovalProcess {
    #[builder(setter(into))]
    pub(super) id: ApprovalProcessId,
    pub(super) policy_id: PolicyId,
    pub(super) process_type: ApprovalProcessType,
    pub(super) rules: ApprovalRules,
    #[builder(setter(into))]
    pub(super) target_ref: String,
}

impl NewApprovalProcess {
    pub fn builder() -> NewApprovalProcessBuilder {
        NewApprovalProcessBuilder::default()
    }

    pub fn committee_id(&self) -> Option<CommitteeId> {
        self.rules.committee_id()
    }
}

impl IntoEvents<ApprovalProcessEvent> for NewApprovalProcess {
    fn into_events(self) -> EntityEvents<ApprovalProcessEvent> {
        EntityEvents::init(
            self.id,
            [ApprovalProcessEvent::Initialized {
                id: self.id,
                policy_id: self.policy_id,
                process_type: self.process_type,
                rules: self.rules,
                target_ref: self.target_ref,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_events(rules: ApprovalRules) -> EntityEvents<ApprovalProcessEvent> {
        EntityEvents::init(
            ApprovalProcessId::new(),
            [ApprovalProcessEvent::Initialized {
                id: ApprovalProcessId::new(),
                policy_id: PolicyId::new(),
                process_type: ApprovalProcessType::from_owned("type".to_string()),
                rules,
                target_ref: "target_ref".to_string(),
            }],
        )
    }

    #[test]
    fn approve() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::CommitteeThreshold {
                threshold: 2,
                committee_id: CommitteeId::new(),
            }))
            .expect("Could not build approval process");
        let approver = CommitteeMemberId::new();
        let eligible = [approver].iter().copied().collect();
        assert!(process.approve(&eligible, approver).did_execute());
        assert!(process.approvers().contains(&approver));
    }

    #[test]
    fn approve_not_eligible() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::CommitteeThreshold {
                threshold: 2,
                committee_id: CommitteeId::new(),
            }))
            .expect("Could not build approval process");
        let approver = CommitteeMemberId::new();
        assert!(process.approve(&HashSet::new(), approver).was_already_applied());
        assert!(process.approvers().is_empty());
    }

    #[test]
    fn approve_already_voted() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::CommitteeThreshold {
                threshold: 2,
                committee_id: CommitteeId::new(),
            }))
            .expect("Could not build approval process");
        let approver = CommitteeMemberId::new();
        let eligible: HashSet<_> = [approver].iter().copied().collect();
        assert!(process.approve(&eligible, approver).did_execute());
        assert!(process.approve(&eligible, approver).was_already_applied());
    }

    #[test]
    fn approve_already_concluded() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::SystemAutoApprove))
                .expect("Could not build approval process");
        let _ = process.check_concluded(HashSet::new());
        let approver = CommitteeMemberId::new();
        let eligible: HashSet<_> = [approver].iter().copied().collect();
        assert!(process.approve(&eligible, approver).was_already_applied());
    }

    #[test]
    fn deny() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::CommitteeThreshold {
                threshold: 2,
                committee_id: CommitteeId::new(),
            }))
            .expect("Could not build approval process");
        let denier = CommitteeMemberId::new();
        let reason = String::new();
        let eligible = [denier].iter().copied().collect();
        assert!(process.deny(&eligible, denier, reason).did_execute());
        assert!(process.deniers().contains(&denier));
    }

    #[test]
    fn deny_not_eligible() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::CommitteeThreshold {
                threshold: 2,
                committee_id: CommitteeId::new(),
            }))
            .expect("Could not build approval process");
        let denier = CommitteeMemberId::new();
        let reason = String::new();
        assert!(process.deny(&HashSet::new(), denier, reason).was_already_applied());
        assert!(process.deniers().is_empty());
    }

    #[test]
    fn deny_already_voted() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::CommitteeThreshold {
                threshold: 2,
                committee_id: CommitteeId::new(),
            }))
            .expect("Could not build approval process");
        let denier = CommitteeMemberId::new();
        let eligible: HashSet<_> = [denier].iter().copied().collect();
        assert!(process.approve(&eligible, denier).did_execute());
        assert!(process.deny(&eligible, denier, String::new()).was_already_applied());
    }

    #[test]
    fn deny_already_concluded() {
        let mut process =
            ApprovalProcess::try_from_events(init_events(ApprovalRules::SystemAutoApprove))
                .expect("Could not build approval process");
        let _ = process.check_concluded(HashSet::new());
        let denier = CommitteeMemberId::new();
        let eligible: HashSet<_> = [denier].iter().copied().collect();
        assert!(process.deny(&eligible, denier, String::new()).was_already_applied());
    }
}
