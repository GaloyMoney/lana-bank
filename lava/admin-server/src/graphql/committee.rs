use async_graphql::*;

use crate::primitives::*;

use super::{loader::LavaDataLoader, user::User};

pub use governance::{committee_cursor::CommitteeByCreatedAtCursor, Committee as DomainCommittee};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Committee {
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
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let users = loader
            .load_many(self.entity.members().into_iter().map(UserId::from))
            .await?
            .into_values()
            .map(User::from)
            .collect();

        Ok(users)
    }
}

#[derive(InputObject)]
pub struct CommitteeCreateInput {
    pub name: String,
}
crate::mutation_payload! { CommitteeCreatePayload, committee: Committee }

#[derive(InputObject)]
pub struct CommitteeAddUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}
crate::mutation_payload! { CommitteeAddUserPayload, committee: Committee }

#[derive(InputObject)]
pub struct CommitteeRemoveUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}
crate::mutation_payload! { CommitteeRemoveUserPayload, committee: Committee }
