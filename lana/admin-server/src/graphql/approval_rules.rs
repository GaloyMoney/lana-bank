use async_graphql::*;

use super::{committee::Committee, loader::LanaDataLoader};

use lana_app::governance::{ApprovalRules as DomainApprovalRules, CommitteeId};

#[derive(async_graphql::Union)]
pub(super) enum ApprovalRules {
    System(SystemApproval),
    CommitteeThreshold(CommitteeThreshold),
}

impl From<DomainApprovalRules> for ApprovalRules {
    fn from(rules: DomainApprovalRules) -> Self {
        match rules {
            DomainApprovalRules::CommitteeThreshold {
                threshold,
                committee_id,
            } => ApprovalRules::CommitteeThreshold(CommitteeThreshold {
                threshold,
                committee_id,
            }),
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

#[derive(SimpleObject)]
#[graphql(complex)]
pub(super) struct CommitteeThreshold {
    threshold: usize,
    #[graphql(skip)]
    committee_id: CommitteeId,
}

#[ComplexObject]
impl CommitteeThreshold {
    async fn committee(&self, ctx: &Context<'_>) -> async_graphql::Result<Committee> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let committee = loader
            .load_one(self.committee_id)
            .await?
            .expect("committee not found");
        Ok(committee)
    }
}
