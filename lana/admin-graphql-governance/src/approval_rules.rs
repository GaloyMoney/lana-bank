use async_graphql::*;

use lana_app::governance::{ApprovalRules as DomainApprovalRules, CommitteeId};

use super::committee::Committee;

#[derive(async_graphql::Union)]
pub enum ApprovalRules {
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
pub struct SystemApproval {
    auto_approve: bool,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CommitteeThreshold {
    threshold: usize,
    #[graphql(skip)]
    committee_id: CommitteeId,
}

#[ComplexObject]
impl CommitteeThreshold {
    async fn committee(&self, ctx: &Context<'_>) -> async_graphql::Result<Committee> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let committee = app
            .governance()
            .find_committee_by_id(sub, self.committee_id)
            .await?
            .expect("committee not found");
        Ok(Committee::from(committee))
    }
}
