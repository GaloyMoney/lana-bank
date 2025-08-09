use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::primitives::*;

use crate::{event::CoreCreditEvent, publisher::CreditFacilityPublisher};

use super::{entity::*, error::ObligationInstallmentError};

#[derive(EsRepo)]
#[es_repo(
    entity = "ObligationInstallment",
    err = "ObligationInstallmentError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        payment_id(ty = "PaymentId", list_for, update(persist = false)),
        obligation_id(ty = "ObligationId", update(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct ObligationInstallmentRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for ObligationInstallmentRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}
impl<E> ObligationInstallmentRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &ObligationInstallment,
        new_events: es_entity::LastPersisted<'_, ObligationInstallmentEvent>,
    ) -> Result<(), ObligationInstallmentError> {
        self.publisher
            .publish_obligation_installment(op, entity, new_events)
            .await
    }
}
