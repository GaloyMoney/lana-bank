use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo, ledger::CreditLedger, primitives::*, CoreCreditAction,
    CoreCreditEvent, CoreCreditObject, CreditFacilitiesByCollateralizationRatioCursor,
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityCollateralizationFromPriceJobConfig<Perms, E> {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub upgrade_buffer_cvl_pct: CVLPct,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>;
}
pub struct CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    credit_facility_repo: CreditFacilityRepo<E>,
    ledger: CreditLedger,
    audit: Perms::Audit,
    price: Price,
}

impl<Perms, E> CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        credit_facility_repo: CreditFacilityRepo<E>,
        ledger: &CreditLedger,
        price: &Price,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            credit_facility_repo,
            ledger: ledger.clone(),
            price: price.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_COLLATERALZIATION_FROM_PRICE_JOB: JobType =
    JobType::new("credit-facility-collateralization-from-price");
impl<Perms, E> JobInitializer for CreditFacilityCollateralizationFromPriceJobInitializer<Perms, E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_COLLATERALZIATION_FROM_PRICE_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            CreditFacilityCollateralizationFromPriceJobRunner::<Perms, E> {
                config: job.config()?,
                credit_facility_repo: self.credit_facility_repo.clone(),
                ledger: self.ledger.clone(),
                price: self.price.clone(),
                audit: self.audit.clone(),
            },
        ))
    }
}

pub struct CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityCollateralizationFromPriceJobConfig<Perms, E>,
    ledger: CreditLedger,
    credit_facility_repo: CreditFacilityRepo<E>,
    price: Price,
    audit: Perms::Audit,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.price.usd_cents_per_btc().await?;
        let mut has_next_page = true;
        let mut after: Option<CreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facilities =
                self.credit_facility_repo
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
            let mut db = self.credit_facility_repo.begin_op().await?;
            let audit_info = self
                .audit
                .record_system_entry_in_tx(
                    db.tx(),
                    CoreCreditObject::all_credit_facilities(),
                    CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
                )
                .await?;

            let mut at_least_one = false;

            for facility in credit_facilities.entities.iter_mut() {
                if facility.status() == CreditFacilityStatus::Closed {
                    continue;
                }
                let balances = self
                    .ledger
                    .get_credit_facility_balance(facility.account_ids)
                    .await?;
                if facility
                    .update_collateralization(
                        price,
                        self.config.upgrade_buffer_cvl_pct,
                        balances,
                        &audit_info,
                    )
                    .did_execute()
                {
                    self.credit_facility_repo
                        .update_in_op(&mut db, facility)
                        .await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                db.commit().await?;
            } else {
                break;
            }
        }

        Ok(JobCompletion::RescheduleIn(self.config.job_interval))
    }
}
