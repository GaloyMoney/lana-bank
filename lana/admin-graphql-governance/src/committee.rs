use admin_graphql_access::User;
use admin_graphql_shared::primitives::*;
use async_graphql::*;

pub use lana_app::governance::{
    Committee as DomainCommittee, committee_cursor::CommitteesByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Committee {
    id: ID,
    committee_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub entity: Arc<DomainCommittee>,
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
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let member_ids: Vec<UserId> = self
            .entity
            .members()
            .into_iter()
            .map(UserId::from)
            .collect();
        let users = app.access().users().find_all::<User>(&member_ids).await?;
        Ok(users.into_values().collect())
    }
}

#[derive(InputObject)]
pub struct CommitteeCreateInput {
    pub name: String,
}
mutation_payload! { CommitteeCreatePayload, committee: Committee }

#[derive(InputObject)]
pub struct CommitteeAddUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}
mutation_payload! { CommitteeAddUserPayload, committee: Committee }

#[derive(InputObject)]
pub struct CommitteeRemoveUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}
mutation_payload! { CommitteeRemoveUserPayload, committee: Committee }
