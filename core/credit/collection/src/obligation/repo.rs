use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    primitives::{BeneficiaryId, ObligationId, ObligationStatus},
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
        status(
            ty = "ObligationStatus",
            create(accessor = "initial_status()"),
            update(accessor = "status()")
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

    pub async fn list_needing_transition(
        &self,
        day: chrono::NaiveDate,
    ) -> Result<Vec<Obligation>, ObligationError> {
        let (entities, _) = es_query!(
            tbl_prefix = "core",
            "SELECT id FROM core_obligations WHERE next_transition_date IS NOT NULL AND next_transition_date <= $1",
            day as chrono::NaiveDate,
        )
        .fetch_n(self.pool(), 1000)
        .await?;
        Ok(entities)
    }
}

mod obligation_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::ObligationStatus;

    impl Type<Postgres> for ObligationStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for ObligationStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for ObligationStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for ObligationStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
