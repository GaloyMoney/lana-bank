use async_graphql::*;

use super::{committee::Committee, loader::LanaDataLoader};

use lana_app::governance::{ApprovalRules as DomainApprovalRules, CommitteeId};

#[derive(async_graphql::Union)]
pub(super) enum ApprovalRules {
    System(SystemApproval),
    CommitteeApproval(CommitteeApproval),
}

impl From<DomainApprovalRules> for ApprovalRules {
    fn from(rules: DomainApprovalRules) -> Self {
        match rules {
            DomainApprovalRules::Committee { committee_id } => {
                ApprovalRules::CommitteeApproval(CommitteeApproval { committee_id })
            }
            DomainApprovalRules::SystemAutoApprove => {
                ApprovalRules::System(SystemApproval { auto_approve: true })
            }
        }
    }
}

#[derive(SimpleObject)]
pub(super) struct SystemApproval {
    auto_approve: bool,
}

pub(super) struct CommitteeApproval {
    committee_id: CommitteeId,
}

#[Object]
impl CommitteeApproval {
    async fn committee(&self, ctx: &Context<'_>) -> async_graphql::Result<Committee> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let committee = loader
            .load_one(self.committee_id)
            .await?
            .expect("committee not found");
        Ok(committee)
    }
}
