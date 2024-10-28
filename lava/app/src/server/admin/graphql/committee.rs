use async_graphql::{dataloader::DataLoader, *};

use crate::{
    primitives::{CommitteeId, UserId},
    server::shared_graphql::{
        convert::ToGlobalId,
        primitives::{Timestamp, UUID},
    },
};

use super::{user::User, LavaDataLoader};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Committee {
    id: ID,
    committee_id: UUID,
    #[graphql(skip)]
    user_ids: Vec<UUID>,
    created_at: Timestamp,
}

#[ComplexObject]
impl Committee {
    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let users = loader
            .load_many(self.user_ids.iter().map(UserId::from))
            .await?
            .into_values()
            .map(User::from)
            .collect();

        Ok(users)
    }
}

impl ToGlobalId for CommitteeId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("committee:{}", self))
    }
}

impl From<governance::Committee> for Committee {
    fn from(committee: governance::Committee) -> Self {
        Self {
            id: committee.id.to_global_id(),
            committee_id: committee.id.into(),
            user_ids: committee.users().iter().map(|user| user.into()).collect(),
            created_at: committee.created_at().into(),
        }
    }
}

#[derive(InputObject)]
pub struct CommitteeCreateInput {
    pub name: String,
}

#[derive(SimpleObject)]
pub struct CommitteeCreatePayload {
    pub committee: Committee,
}

impl From<governance::Committee> for CommitteeCreatePayload {
    fn from(committee: governance::Committee) -> Self {
        Self {
            committee: committee.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CommitteeAddUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}

#[derive(SimpleObject)]
pub struct CommitteeAddUserPayload {
    pub committee: Committee,
}

impl From<governance::Committee> for CommitteeAddUserPayload {
    fn from(committee: governance::Committee) -> Self {
        Self {
            committee: committee.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CommitteeRemoveUserInput {
    pub committee_id: UUID,
    pub user_id: UUID,
}

#[derive(SimpleObject)]
pub struct CommitteeRemoveUserPayload {
    pub committee: Committee,
}

impl From<governance::Committee> for CommitteeRemoveUserPayload {
    fn from(committee: governance::Committee) -> Self {
        Self {
            committee: committee.into(),
        }
    }
}
