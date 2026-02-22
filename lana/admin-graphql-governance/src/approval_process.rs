use async_graphql::*;

use admin_graphql_shared::primitives::*;

pub use admin_graphql_shared::governance::ApprovalProcess;

pub use lana_app::governance::approval_process_cursor::ApprovalProcessesByCreatedAtCursor;

#[derive(InputObject)]
pub struct ApprovalProcessApproveInput {
    pub process_id: UUID,
}
mutation_payload! { ApprovalProcessApprovePayload, approval_process: ApprovalProcess }

#[derive(InputObject)]
pub struct ApprovalProcessDenyInput {
    pub process_id: UUID,
}
mutation_payload! { ApprovalProcessDenyPayload, approval_process: ApprovalProcess }
