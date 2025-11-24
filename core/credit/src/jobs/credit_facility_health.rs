//! CVL Monitor listens to changes in collateralization state of
//! credit facilities and initiates a partial liquidation of credit
//! facility whose CVL drops below liquidation threshold, unless this
//! credit facility is already in an active liquidation.
//!
//! All other state changes are ignored.

use job::Jobs;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{CollateralizationState, CoreCreditEvent, liquidation_process::LiquidationProcessRepo};

pub struct CreditFacilityHealthJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
}

impl<E: OutboxEventMarker<CoreCreditEvent>> CreditFacilityHealthJobRunner<E> {
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        if let Some(event) = message.as_event() {
            match event {
                FacilityCollateralizationChanged {
                    id,
                    state,
                    collateral,
                    outstanding,
                    cvl,
                    price,
                    ..
                } if *state == CollateralizationState::UnderLiquidationThreshold => {
                    let x = self
                        .liquidation_process_repo
                        .list_for_credit_facility_id_by_created_at(
                            *id,
                            Default::default(),
                            Default::default(),
                        )
                        .await
                        .unwrap();

                    if !x.entities.is_empty() {
                        // initiate liquidation
                        todo!()
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
