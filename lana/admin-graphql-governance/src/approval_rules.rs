use async_graphql::*;

use super::committee::Committee;

use lana_app::governance::{ApprovalRules as DomainApprovalRules, CommitteeId};

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
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let committee = app
            .governance()
            .find_all_committees::<Committee>(&[self.committee_id])
            .await?
            .into_values()
            .next()
            .expect("committee not found");
        Ok(committee)
    }
}
