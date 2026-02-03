use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::GovernanceEvent;
use job::*;
use obix::EventSequence;
use obix::out::{
    EphemeralOutboxEvent, Outbox, OutboxEvent, OutboxEventMarker, PersistentOutboxEvent,
};

use core_custody::CoreCustodyEvent;
use core_price::{CorePriceEvent, Price};

use crate::{
    credit_facility::{
        CreditFacilitiesByCollateralizationRatioCursor, CreditFacilityRepo, CreditFacilityStatus,
    },
    event::CoreCreditEvent,
    ledger::*,
    primitives::*,
};

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityCollateralizationFromEventsJobConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for CreditFacilityCollateralizationFromEventsJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct CreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<CreditFacilityRepo<E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

impl<Perms, E> CreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        repo: Arc<CreditFacilityRepo<E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            repo,
            price,
            ledger,
            authz,
        }
    }
}

const CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.credit-facility-collateralization");

impl<Perms, E> JobInitializer for CreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = CreditFacilityCollateralizationFromEventsJobConfig<E>;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityCollateralizationFromEventsRunner::<
            Perms,
            E,
        > {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
            price: self.price.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct CreditFacilityCollateralizationFromEventsData {
    sequence: EventSequence,
}

pub struct CreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<CreditFacilityRepo<E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

impl<Perms, E> CreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.collateralization_job.process_persistent_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn process_persistent_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(
                event @ CoreCreditEvent::FacilityCollateralUpdated {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ CoreCreditEvent::ObligationCreated {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ CoreCreditEvent::FacilityPaymentAllocated {
                    credit_facility_id: id,
                    ..
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("credit_facility_id", tracing::field::display(id));

                self.update_collateralization_from_events(*id).await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[instrument(name = "core_credit.credit_facility_collateralization_job.process_ephemeral_message", parent = None, skip(self, message), fields(handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_ephemeral_message(
        &self,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.payload.as_event() {
            Some(CorePriceEvent::PriceUpdated { price, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", tracing::field::display(&message.event_type));

                self.update_collateralization_from_price_event(*price)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_events",
        skip(self),
        fields(credit_facility_id = %credit_facility_id),
    )]
    #[es_entity::retry_on_concurrent_modification]
    pub(super) async fn update_collateralization_from_events(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<(), crate::credit_facility::error::CreditFacilityError> {
        let mut op = self.repo.begin_op().await?;
        // if the pending facility is not collateralized enough to be activated there will be no
        // credit facility to update the collateralization state for
        let Some(mut credit_facility) = self.repo.maybe_find_by_id(credit_facility_id).await?
        else {
            return Ok(());
        };

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
            )
            .await?;

        tracing::Span::current().record("credit_facility_id", credit_facility.id.to_string());

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        let price = self.price.usd_cents_per_btc().await;

        if credit_facility
            .update_collateralization(price, CVLPct::UPGRADE_BUFFER, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut credit_facility)
                .await?;

            op.commit().await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_price_event",
        skip(self)
    )]
    pub(super) async fn update_collateralization_from_price_event(
        &self,
        price: PriceOfOneBTC,
    ) -> Result<(), crate::credit_facility::error::CreditFacilityError> {
        let mut has_next_page = true;
        let mut after: Option<CreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facilities =
                self.repo
                    .list_by_collateralization_ratio(
                        es_entity::PaginatedQueryArgs::<
                            CreditFacilitiesByCollateralizationRatioCursor,
                        > {
                            first: 10,
                            after,
                        },
                        es_entity::ListDirection::Ascending,
                    )
                    .await?;
            (after, has_next_page) = (
                credit_facilities.end_cursor,
                credit_facilities.has_next_page,
            );
            let mut op = self.repo.begin_op().await?;
            self.authz
                .audit()
                .record_system_entry_in_tx(
                    &mut op,
                    CoreCreditObject::all_credit_facilities(),
                    CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
                )
                .await?;

            let mut at_least_one = false;

            for facility in credit_facilities.entities.iter_mut() {
                tracing::Span::current().record("credit_facility_id", facility.id.to_string());

                if facility.status() == CreditFacilityStatus::Closed {
                    continue;
                }
                let balances = self
                    .ledger
                    .get_credit_facility_balance(facility.account_ids)
                    .await?;
                if facility
                    .update_collateralization(price, CVLPct::UPGRADE_BUFFER, balances)
                    .did_execute()
                {
                    self.repo.update_in_op(&mut op, facility).await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                op.commit().await?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for CreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityCollateralizationFromEventsData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_all(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                event = stream.next() => {
                    match event {
                        Some(event) => {
                            match event {
                                OutboxEvent::Persistent(e) => {
                                    self.process_persistent_message(&e).await?;
                                    state.sequence = e.sequence;
                                    current_job.update_execution_state(state).await?;
                                }
                                OutboxEvent::Ephemeral(e) => {
                                    self.process_ephemeral_message(e.as_ref()).await?;
                                }
                            }
                        } None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
