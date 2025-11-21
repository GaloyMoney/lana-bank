use sqlx::PgPool;

use es_entity::{es_query, EntityEvents, EsEntity, EsEvent, EsRepo};
use outbox::OutboxEventMarker;

use core_credit::CreditFacilityId;
use core_deposit::DepositAccountId;

use crate::{event::CorePaymentLinkEvent, primitives::*, publisher::PaymentLinkPublisher};

use super::{entity::*, error::FundingLinkError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "FundingLink",
    err = "FundingLinkError",
    columns(
        customer_id(ty = "core_customer::CustomerId", update(accessor = "customer_id")),
        deposit_account_id(
            ty = "core_deposit::DepositAccountId",
            update(accessor = "deposit_account_id")
        ),
        credit_facility_id(ty = "core_credit::CreditFacilityId", list_by, update(accessor = "credit_facility_id")),
        status(ty = "crate::primitives::LinkStatus", create(persist = false), update(accessor = "status"))
    ),
    tbl_prefix = "cpl",
    post_persist_hook = "publish_events"
)]
pub(crate) struct FundingLinkRepo<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    pool: PgPool,
    publisher: PaymentLinkPublisher<E>,
}

impl<E> FundingLinkRepo<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    pub fn new(pool: &PgPool, publisher: &PaymentLinkPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    pub async fn find_all_by_deposit_account_id(
        &self,
        deposit_account_id: DepositAccountId,
    ) -> Result<Vec<FundingLink>, FundingLinkError> {
        let mut op = self.pool().begin().await?;
        let ids: Vec<FundingLinkId> = sqlx::query_scalar!(
            r#"
            SELECT id as "id: FundingLinkId" FROM cpl_funding_links 
            WHERE deposit_account_id = $1
            "#,
            deposit_account_id as DepositAccountId
        )
        .fetch_all(&mut *op)
        .await?;

        let mut links = Vec::new();
        for id in ids {
            links.push(self.find_by_id(id).await?);
        }
        Ok(links)
    }

    async fn publish_events(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &FundingLink,
        new_events: es_entity::LastPersisted<'_, FundingLinkEvent>,
    ) -> Result<(), FundingLinkError> {
        let publish_events = new_events.filter_map(|persisted_event| {
            match &persisted_event.event {
                FundingLinkEvent::Initialized {
                    id,
                    customer_id,
                    deposit_account_id,
                    ..
                } => Some(CorePaymentLinkEvent::FundingLinkCreated {
                    id: *id,
                    customer_id: *customer_id,
                    deposit_account_id: *deposit_account_id,
                    created_at: entity.created_at(),
                }),
                FundingLinkEvent::Activated => Some(CorePaymentLinkEvent::FundingLinkActivated {
                    id: entity.id,
                    activated_at: chrono::Utc::now(),
                }),
                FundingLinkEvent::Deactivated => Some(CorePaymentLinkEvent::FundingLinkDeactivated {
                    id: entity.id,
                    deactivated_at: chrono::Utc::now(),
                }),
                FundingLinkEvent::Broken { reason } => Some(CorePaymentLinkEvent::FundingLinkBroken {
                    id: entity.id,
                    reason: *reason,
                    broken_at: chrono::Utc::now(),
                }),
            }
        });

        self.publisher
            .publish_all(op, publish_events)
            .await
    }
}
