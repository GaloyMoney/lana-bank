use async_graphql::{connection::*, *};
use es_entity::{EsEntity as _, Sort};

use crate::primitives::*;

use super::{
    access::User,
    event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
    loader::LanaDataLoader,
    primitives::SortDirection,
};

pub use lana_app::governance::{
    Committee as DomainCommittee, CommitteesSortBy as DomainCommitteesSortBy,
};

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

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        event_timeline::events_to_connection(self.entity.events(), first, after)
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
pub struct CommitteeCreateInput {
    pub name: String,
    pub member_user_ids: Vec<UUID>,
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

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommitteesSortBy {
    #[default]
    CreatedAt,
    Name,
}

impl From<CommitteesSortBy> for DomainCommitteesSortBy {
    fn from(by: CommitteesSortBy) -> Self {
        match by {
            CommitteesSortBy::CreatedAt => DomainCommitteesSortBy::CreatedAt,
            CommitteesSortBy::Name => DomainCommitteesSortBy::Name,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct CommitteesSort {
    #[graphql(default)]
    pub by: CommitteesSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<CommitteesSort> for Sort<DomainCommitteesSortBy> {
    fn from(sort: CommitteesSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<CommitteesSort> for DomainCommitteesSortBy {
    fn from(sort: CommitteesSort) -> Self {
        sort.by.into()
    }
}
