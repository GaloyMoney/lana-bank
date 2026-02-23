use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    primitives::{BeneficiaryId, ObligationId},
    public::CoreCreditCollectionEvent,
    publisher::CollectionPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Obligation",
    err = "ObligationError",
    columns(
        beneficiary_id(
            ty = "BeneficiaryId",
            list_for(by(created_at)),
            update(persist = false)
        ),
        reference(ty = "String", create(accessor = "reference()")),
        next_transition_date(
            ty = "Option<chrono::NaiveDate>",
            create(accessor = "next_transition_date()"),
            update(accessor = "next_transition_date()")
        ),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct ObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pool: PgPool,
    publisher: CollectionPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for ObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> ObligationRepo<E>
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CollectionPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "obligation.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Obligation,
        new_events: es_entity::LastPersisted<'_, ObligationEvent>,
    ) -> Result<(), ObligationError> {
        self.publisher
            .publish_obligation_in_op(op, entity, new_events)
            .await
    }

    pub async fn list_ids_needing_transition(
        &self,
        day: chrono::NaiveDate,
        after_id: Option<ObligationId>,
        limit: i64,
    ) -> Result<Vec<ObligationId>, ObligationError> {
        let rows = sqlx::query_scalar!(
            r#"SELECT id AS "id: ObligationId"
               FROM core_obligations
               WHERE next_transition_date IS NOT NULL
                 AND next_transition_date <= $1
                 AND ($2::uuid IS NULL OR id > $2)
               ORDER BY id
               LIMIT $3"#,
            day,
            after_id as Option<ObligationId>,
            limit,
        )
        .fetch_all(self.pool())
        .await?;
        Ok(rows)
    }
}
