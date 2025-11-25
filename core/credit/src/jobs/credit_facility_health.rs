//! Credit Facility Health listens to changes in collateralization
//! state of credit facilities and initiates a partial liquidation of
//! credit facility whose CVL drops below liquidation threshold
//! (i. e. became unhealthy), unless this credit facility is already
//! in an active liquidation.
//!
//! All other state changes are ignored by this job.

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::PriceOfOneBTC;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CollateralizationState, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilities,
    CreditFacilityId, liquidation_process::LiquidationProcessRepo,
};

pub struct CreditFacilityHealthJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
    facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> CreditFacilityHealthJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
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
                    match self
                        .liquidation_process_repo
                        .find_by_credit_facility_id(*id)
                        .await
                    {
                        Err(e) if e.was_not_found() => {
                            self.initiate_liquidation(*id, *price).await;
                        }
                        Err(e) => {}
                        Ok(_) => {
                            // liquidation process already running for this facility
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn initiate_liquidation(
        &self,
        credit_facility_id: CreditFacilityId,
        price: PriceOfOneBTC,
    ) {
        let mut facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await
            .unwrap();
        let x = facility.initiate_partial_liquidation(price);
    }
}
