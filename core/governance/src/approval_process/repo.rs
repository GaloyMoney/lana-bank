use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "ApprovalProcess",
    err = "ApprovalProcessError",
    columns(
        process_type(ty = "ApprovalProcessType"),
        committee_id(
            ty = "Option<CommitteeId>",
            list_for,
            create(accessor = "committee_id()"),
            update(accessor = "committee_id()")
        ),
        policy_id(ty = "PolicyId")
    )
)]
pub(crate) struct ApprovalProcessRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl ApprovalProcessRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
