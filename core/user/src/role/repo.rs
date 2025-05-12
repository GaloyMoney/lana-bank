use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreUserEvent, primitives::*};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(entity = "Role", err = "RoleError", tbl_prefix = "core")]
pub(crate) struct RoleRepo<E>
where
    E: OutboxEventMarker<CoreUserEvent>,
{
    pool: PgPool,
    publisher: std::marker::PhantomData<E>,
}

impl<E> Clone for RoleRepo<E>
where
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
        }
    }
}

impl<E> RoleRepo<E>
where
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub fn new(pool: &PgPool) -> Self {
        Self {
            pool: pool.clone(),
            publisher: Default::default(),
        }
    }
}
