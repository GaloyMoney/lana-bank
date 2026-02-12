use async_graphql::*;

use crate::primitives::*;

use super::{access::User, loader::LanaDataLoader};

pub(super) use lana_app::governance::{
    Committee as DomainCommittee, committee_cursor::CommitteesByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct Committee {
    id: ID,
    committee_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(super) entity: Arc<DomainCommittee>,
}

impl From<DomainCommittee> for Committee {
    fn from(committee: DomainCommittee) -> Self {
        Self {
            id: committee.id.to_global_id(),
            committee_id: committee.id.into(),
            created_at: committee.created_at().into(),
            entity: Arc::new(committee),
        }
    }
}

#[ComplexObject]
impl Committee {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn current_members(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let users = loader
            .load_many(self.entity.members().into_iter().map(UserId::from))
            .await?
            .into_values()
            .collect();

        Ok(users)
    }
}

#[derive(InputObject)]
pub(super) struct CommitteeCreateInput {
    pub name: String,
}
crate::mutation_payload! { CommitteeCreatePayload, committee: Committee }

#[derive(InputObject)]
pub(super) struct CommitteeAddUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}
crate::mutation_payload! { CommitteeAddUserPayload, committee: Committee }

#[derive(InputObject)]
pub(super) struct CommitteeRemoveUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}
crate::mutation_payload! { CommitteeRemoveUserPayload, committee: Committee }
