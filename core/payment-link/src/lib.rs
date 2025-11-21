#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod error;
mod event;
mod funding_link;
mod primitives;
mod publisher;

pub mod jobs;

use std::sync::Arc;

use tracing::instrument;

use outbox::{Outbox, OutboxEventMarker};

pub use error::PaymentLinkError;
pub use event::CorePaymentLinkEvent;
pub use funding_link::{FundingLink, NewFundingLink, error::FundingLinkError};
pub use primitives::*;

use funding_link::FundingLinkRepo;
use publisher::PaymentLinkPublisher;

use core_credit::CreditFacilityId;
use core_customer::CustomerId;
use core_deposit::DepositAccountId;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::funding_link::FundingLinkEvent;
}

pub struct CorePaymentLink<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    funding_links: Arc<FundingLinkRepo<E>>,
}

impl<E> Clone for CorePaymentLink<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    fn clone(&self) -> Self {
        Self {
            funding_links: self.funding_links.clone(),
        }
    }
}

impl<E> CorePaymentLink<E>
where
    E: OutboxEventMarker<CorePaymentLinkEvent>,
{
    #[instrument(name = "payment_link.init", skip_all, err)]
    pub async fn init(
        pool: &sqlx::PgPool,
        outbox: &Outbox<E>,
    ) -> Result<Self, PaymentLinkError> {
        let publisher = PaymentLinkPublisher::new(outbox);
        let funding_links = FundingLinkRepo::new(pool, &publisher);

        Ok(Self {
            funding_links: Arc::new(funding_links),
        })
    }

    #[instrument(name = "payment_link.create_funding_link", skip(self), err)]
    pub async fn create_funding_link(
        &self,
        customer_id: CustomerId,
        deposit_account_id: DepositAccountId,
        credit_facility_id: CreditFacilityId,
    ) -> Result<FundingLink, PaymentLinkError> {
        let funding_link_id = FundingLinkId::new();

        let new_link = NewFundingLink::builder()
            .id(funding_link_id)
            .customer_id(customer_id)
            .deposit_account_id(deposit_account_id)
            .credit_facility_id(credit_facility_id)
            .build()
            .expect("all fields provided for new funding link");

        let mut db = self.funding_links.begin_op().await?;
        let mut link = self.funding_links.create_in_op(&mut db, new_link).await?;

        // Activate immediately upon creation
        link.activate();
        self.funding_links.update_in_op(&mut db, &mut link).await?;

        db.commit().await?;

        Ok(link)
    }

    #[instrument(name = "payment_link.find_by_credit_facility", skip(self), err)]
    pub async fn find_by_credit_facility(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Option<FundingLink>, PaymentLinkError> {
        Ok(Some(self
            .funding_links
            .find_by_credit_facility_id(credit_facility_id)
            .await?))
    }

    #[instrument(name = "payment_link.break_links_by_deposit_account", skip(self), err)]
    pub async fn break_links_by_deposit_account(
        &self,
        deposit_account_id: DepositAccountId,
        reason: BrokenReason,
    ) -> Result<(), PaymentLinkError> {
        let links = self
            .funding_links
            .find_all_by_deposit_account_id(deposit_account_id)
            .await?;

        for mut link in links {
            if link.status != LinkStatus::Broken {
                let mut db = self.funding_links.begin_op().await?;
                link.mark_broken(reason)?;
                self.funding_links.update_in_op(&mut db, &mut link).await?;
                db.commit().await?;
            }
        }

        Ok(())
    }

    #[instrument(name = "payment_link.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        id: FundingLinkId,
    ) -> Result<FundingLink, PaymentLinkError> {
        Ok(self.funding_links.find_by_id(id).await?)
    }
}

