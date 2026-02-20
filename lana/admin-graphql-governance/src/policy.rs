use async_graphql::*;

use admin_graphql_shared::primitives::*;

pub use admin_graphql_shared::governance::Policy;

pub use lana_app::governance::policy_cursor::PoliciesByCreatedAtCursor;

#[derive(InputObject)]
pub struct PolicyAssignCommitteeInput {
    pub policy_id: UUID,
    pub committee_id: UUID,
    pub threshold: usize,
}

mutation_payload! { PolicyAssignCommitteePayload, policy: Policy }
