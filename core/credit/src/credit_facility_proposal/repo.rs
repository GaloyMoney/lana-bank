use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use old_money::UsdCents;

use crate::{CoreCreditEvent, primitives::*, publisher::*};

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "CreditFacilityProposal",
    columns(
        customer_id(ty = "CustomerId", list_for(by(created_at)), update(persist = false)),
        approval_process_id(ty = "Option<ApprovalProcessId>", list_by, create(persist = "false")),
        status(
            ty = "CreditFacilityProposalStatus",
            list_for,
            update(accessor = "status()")
        ),
        amount(ty = "UsdCents", list_by, update(persist = false))
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct CreditFacilityProposalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for CreditFacilityProposalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> CreditFacilityProposalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[tracing::instrument(name = "credit_facility_proposal.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacilityProposal,
        new_events: es_entity::LastPersisted<'_, CreditFacilityProposalEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_proposal_in_op(op, entity, new_events)
            .await
    }
}

impl From<(CreditFacilityProposalsSortBy, &CreditFacilityProposal)>
    for credit_facility_proposal_cursor::CreditFacilityProposalsCursor
{
    fn from(proposal_with_sort: (CreditFacilityProposalsSortBy, &CreditFacilityProposal)) -> Self {
        let (sort, proposal) = proposal_with_sort;
        match sort {
            CreditFacilityProposalsSortBy::CreatedAt => {
                credit_facility_proposal_cursor::CreditFacilityProposalsByCreatedAtCursor::from(
                    proposal,
                )
                .into()
            }
            CreditFacilityProposalsSortBy::Id => {
                credit_facility_proposal_cursor::CreditFacilityProposalsByIdCursor::from(proposal)
                    .into()
            }
            CreditFacilityProposalsSortBy::ApprovalProcessId => {
                credit_facility_proposal_cursor::CreditFacilityProposalsByApprovalProcessIdCursor::from(proposal)
                    .into()
            }
            CreditFacilityProposalsSortBy::Amount => {
                credit_facility_proposal_cursor::CreditFacilityProposalsByAmountCursor::from(
                    proposal,
                )
                .into()
            }
        }
    }
}

mod proposal_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::CreditFacilityProposalStatus;

    impl Type<Postgres> for CreditFacilityProposalStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CreditFacilityProposalStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CreditFacilityProposalStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for CreditFacilityProposalStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
