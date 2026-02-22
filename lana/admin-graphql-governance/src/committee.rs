use async_graphql::*;

use admin_graphql_shared::primitives::*;

pub use admin_graphql_shared::governance::Committee;

pub use lana_app::governance::{
    Committee as DomainCommittee, committee_cursor::CommitteesByCreatedAtCursor,
};

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
