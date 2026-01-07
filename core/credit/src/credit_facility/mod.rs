mod entity;
pub mod error;
pub mod interest_accrual_cycle;
mod repo;

use std::sync::Arc;
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::{JobId, Jobs};
use obix::out::OutboxEventMarker;

use crate::{
    PublicIds,
    disbursal::Disbursals,
    event::CoreCreditEvent,
    jobs::{credit_facility_maturity, interest_accruals},
    ledger::{CreditFacilityInterestAccrual, CreditFacilityInterestAccrualCycle, CreditLedger},
    obligation::Obligations,
    pending_credit_facility::{PendingCreditFacilities, PendingCreditFacilityCompletionOutcome},
    primitives::*,
    terms::InterestPeriod,
};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

pub use entity::CreditFacility;
pub(crate) use entity::*;
use interest_accrual_cycle::NewInterestAccrualCycleData;

#[cfg(feature = "json-schema")]
pub use entity::CreditFacilityEvent;
use error::CreditFacilityError;
pub use repo::{
    CreditFacilitiesFilter, CreditFacilitiesSortBy, CreditFacilityRepo, ListDirection, Sort,
    credit_facility_cursor::*,
};

pub struct CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
    repo: Arc<CreditFacilityRepo<E>>,
    obligations: Arc<Obligations<Perms, E>>,
    disbursals: Arc<Disbursals<Perms, E>>,
    authz: Arc<Perms>,
    ledger: Arc<CreditLedger>,
    price: Arc<Price>,
    jobs: Arc<Jobs>,
    governance: Arc<Governance<Perms, E>>,
    public_ids: Arc<PublicIds>,
}

impl<Perms, E> Clone for CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            obligations: self.obligations.clone(),
            pending_credit_facilities: self.pending_credit_facilities.clone(),
            disbursals: self.disbursals.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            jobs: self.jobs.clone(),
            governance: self.governance.clone(),
            public_ids: self.public_ids.clone(),
        }
    }
}

pub(super) enum CompletionOutcome {
    AlreadyApplied(CreditFacility),
    Completed((CreditFacility, crate::CreditFacilityCompletion)),
}

#[derive(Clone)]
pub(super) struct ConfirmedAccrual {
    pub(super) accrual: CreditFacilityInterestAccrual,
    pub(super) next_period: Option<InterestPeriod>,
    pub(super) accrual_idx: InterestAccrualCycleIdx,
    pub(super) accrued_count: usize,
}

impl<Perms, E> CreditFacilities<Perms, E>
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
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        obligations: Arc<Obligations<Perms, E>>,
        pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
        disbursals: Arc<Disbursals<Perms, E>>,
        ledger: Arc<CreditLedger>,
        price: Arc<Price>,
        jobs: Arc<Jobs>,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: Arc<Governance<Perms, E>>,
        public_ids: Arc<PublicIds>,
    ) -> Self {
        let repo = CreditFacilityRepo::new(pool, publisher);

        Self {
            repo: Arc::new(repo),
            obligations,
            pending_credit_facilities,
            disbursals,
            authz,
            ledger,
            price,
            jobs,
            governance,
            public_ids,
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'static>, CreditFacilityError> {
        Ok(self.repo.begin_op().await?)
    }

    #[record_error_severity]
    #[instrument(name = "credit.credit_facility.activate", skip(self), fields(credit_facility_id = %credit_facility_id))]
    pub(super) async fn activate(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<(), CreditFacilityError> {
        let mut db = self.repo.begin_op().await?.with_db_time().await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut db,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_ACTIVATE,
            )
            .await?;

        let (mut new_credit_facility_builder, initial_disbursal) = match self
            .pending_credit_facilities
            .complete_in_op(&mut db, credit_facility_id.into())
            .await?
        {
            PendingCreditFacilityCompletionOutcome::Completed {
                new_credit_facility,
                initial_disbursal,
            } => (new_credit_facility, initial_disbursal),
            PendingCreditFacilityCompletionOutcome::Ignored => {
                return Ok(());
            }
        };

        let public_id = self
            .public_ids
            .create_in_op(&mut db, CREDIT_FACILITY_REF_TARGET, credit_facility_id)
            .await?;

        let new_credit_facility = new_credit_facility_builder
            .public_id(public_id.id)
            .build()
            .expect("Could not build NewCreditFacility");

        let mut credit_facility = self.repo.create_in_op(&mut db, new_credit_facility).await?;

        let periods = credit_facility
            .start_interest_accrual_cycle()?
            .expect("first accrual");

        self.repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        self.jobs
            .create_and_spawn_at_in_op(
                &mut db,
                JobId::new(),
                // FIXME: I don't think this is updated if/when the facility is updated
                // if the credit product is closed earlier than expected or if is liquidated
                credit_facility_maturity::CreditFacilityMaturityJobConfig::<Perms, E> {
                    credit_facility_id: credit_facility.id,
                    _phantom: std::marker::PhantomData,
                },
                credit_facility.matures_at(),
            )
            .await?;

        let accrual_id = credit_facility
            .interest_accrual_cycle_in_progress()
            .expect("First accrual not found")
            .id;

        self.jobs
            .create_and_spawn_at_in_op(
                &mut db,
                accrual_id,
                interest_accruals::InterestAccrualJobConfig::<Perms, E> {
                    credit_facility_id,
                    _phantom: std::marker::PhantomData,
                },
                periods.accrual.end,
            )
            .await?;

        let activation_data = if let Some(mut new_disbursal_builder) = initial_disbursal {
            let public_id = self
                .public_ids
                .create_in_op(
                    &mut db,
                    DISBURSAL_REF_TARGET,
                    new_disbursal_builder.unwrap_id(),
                )
                .await?;
            let new_disbursal = new_disbursal_builder
                .public_id(public_id.id)
                .build()
                .expect("could not build new disbursal");
            let disbursal = self
                .disbursals
                .create_pre_approved_disbursal_in_op(&mut db, new_disbursal)
                .await?;

            credit_facility.activation_data(Some(crate::InitialDisbursalOnActivation {
                id: disbursal.id,
                amount: disbursal.amount,
            }))
        } else {
            credit_facility.activation_data(None)
        };

        self.ledger
            .handle_activation(
                &mut db,
                activation_data,
                core_accounting::LedgerTransactionInitiator::System,
            )
            .await?;
        db.commit().await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.confirm_interest_accrual_in_op",
        skip(self, op),
        fields(credit_facility_id = %credit_facility_id)
    )]
    pub(super) async fn confirm_interest_accrual_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        credit_facility_id: CreditFacilityId,
    ) -> Result<ConfirmedAccrual, CreditFacilityError> {
        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let mut credit_facility = self.repo.find_by_id(credit_facility_id).await?;

        let confirmed_accrual = {
            let account_ids = credit_facility.account_ids;
            let balances = self.ledger.get_credit_facility_balance(account_ids).await?;

            let accrual = credit_facility
                .interest_accrual_cycle_in_progress_mut()
                .expect("Accrual in progress should exist for scheduled job");

            let interest_accrual = accrual.record_accrual(balances.disbursed_outstanding());

            ConfirmedAccrual {
                accrual: (interest_accrual, account_ids).into(),
                next_period: accrual.next_accrual_period(),
                accrual_idx: accrual.idx,
                accrued_count: accrual.count_accrued(),
            }
        };

        self.repo.update_in_op(op, &mut credit_facility).await?;

        Ok(confirmed_accrual)
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.complete_in_op",
        skip(self, db),
        fields(credit_facility_id = %credit_facility_id)
    )]
    pub(super) async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<CompletionOutcome, CreditFacilityError> {
        let price = self.price.usd_cents_per_btc().await;

        let mut credit_facility = self.repo.find_by_id(credit_facility_id).await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        let completion = if let es_entity::Idempotent::Executed(completion) =
            credit_facility.complete(price, upgrade_buffer_cvl_pct, balances)?
        {
            completion
        } else {
            return Ok(CompletionOutcome::AlreadyApplied(credit_facility));
        };

        self.repo.update_in_op(db, &mut credit_facility).await?;

        Ok(CompletionOutcome::Completed((credit_facility, completion)))
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.complete_liquidation_in_op",
        skip(self, db),
        fields(credit_facility_id = %credit_facility_id, liquidation_id = %liquidation_id)
    )]
    pub(super) async fn complete_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        liquidation_id: LiquidationId,
    ) -> Result<(), CreditFacilityError> {
        let mut credit_facility = self.repo.find_by_id_in_op(db, credit_facility_id).await?;

        if credit_facility
            .complete_liquidation(liquidation_id)?
            .did_execute()
        {
            self.repo.update_in_op(db, &mut credit_facility).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.facility.complete_interest_cycle_and_maybe_start_new_cycle",
        skip(self, db)
        fields(credit_facility_id = %credit_facility_id),
    )]
    pub(super) async fn complete_interest_cycle_and_maybe_start_new_cycle(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
    ) -> Result<CompletedAccrualCycle, CreditFacilityError> {
        let mut credit_facility = self.repo.find_by_id(credit_facility_id).await?;

        let (accrual_cycle_data, new_obligation) = if let es_entity::Idempotent::Executed(res) =
            credit_facility.record_interest_accrual_cycle()?
        {
            res
        } else {
            unreachable!(
                "record_interest_accrual_cycle returned Idempotent::AlreadyApplied, \
                 but this should only execute when there is an accrual cycle to record"
            );
        };

        if let Some(new_obligation) = new_obligation {
            self.obligations
                .create_with_jobs_in_op(db, new_obligation)
                .await?;
        };

        let res = credit_facility.start_interest_accrual_cycle()?;
        self.repo.update_in_op(db, &mut credit_facility).await?;

        let new_cycle_data = res.map(|periods| {
            let new_accrual_cycle_id = credit_facility
                .interest_accrual_cycle_in_progress()
                .expect("First accrual cycle not found")
                .id;

            NewInterestAccrualCycleData {
                id: new_accrual_cycle_id,
                first_accrual_end_date: periods.accrual.end,
            }
        });

        Ok(CompletedAccrualCycle {
            facility_accrual_cycle_data: (accrual_cycle_data, credit_facility.account_ids).into(),
            new_cycle_data,
        })
    }

    pub async fn find_by_id_without_audit(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        self.repo.find_by_id(id.into()).await
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.find_by_id",
        skip(self, credit_facility_id),
        fields(credit_facility_id = tracing::field::Empty)
    )]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        let id = credit_facility_id.into();
        tracing::Span::current().record("credit_facility_id", id.to_string());
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        self.repo.maybe_find_by_id(id).await
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.mark_as_matured",
        skip(self),
        fields(credit_facility_id = %credit_facility_id)
    )]
    pub(super) async fn mark_facility_as_matured(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<(), CreditFacilityError> {
        let mut facility = self.repo.find_by_id(credit_facility_id).await?;

        if facility.mature().did_execute() {
            self.repo.update(&mut facility).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.find_by_public_id",
        skip(self, public_id),
        fields(public_id = tracing::field::Empty)
    )]
    pub async fn find_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        let public_id = public_id.into();
        tracing::Span::current().record("public_id", public_id.to_string());
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        self.repo.maybe_find_by_public_id(public_id).await
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_price_event",
        skip(self)
    )]
    pub(super) async fn update_collateralization_from_price_event(
        &self,
        upgrade_buffer_cvl_pct: CVLPct,
        price: PriceOfOneBTC,
    ) -> Result<(), CreditFacilityError> {
        let mut has_next_page = true;
        let mut after: Option<CreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facilities =
                self
                    .list_by_collateralization_ratio_without_audit(
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
                    .update_collateralization(price, upgrade_buffer_cvl_pct, balances)
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
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<(), CreditFacilityError> {
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
            .update_collateralization(price, upgrade_buffer_cvl_pct, balances)
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
    #[instrument(name = "credit.credit_facility.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesCursor>,
        filter: CreditFacilitiesFilter,
        sort: impl Into<Sort<CreditFacilitiesSortBy>> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        self.repo.list_for_filter(filter, sort.into(), query).await
    }

    pub(super) async fn list_by_collateralization_ratio_without_audit(
        &self,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCollateralizationRatioCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            CreditFacility,
            CreditFacilitiesByCollateralizationRatioCursor,
        >,
        CreditFacilityError,
    > {
        self.repo
            .list_by_collateralization_ratio(query, direction.into())
            .await
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.list_by_collateralization_ratio",
        skip(self)
    )]
    pub async fn list_by_collateralization_ratio(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCollateralizationRatioCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            CreditFacility,
            CreditFacilitiesByCollateralizationRatioCursor,
        >,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;

        self.list_by_collateralization_ratio_without_audit(query, direction.into())
            .await
    }

    #[record_error_severity]
    #[instrument(name = "credit.credit_facility.find_all", skip(self))]
    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<std::collections::HashMap<CreditFacilityId, T>, CreditFacilityError> {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "credit.credit_facility.list_for_customer", skip(self),fields(customer_id = %customer_id))]
    pub(super) async fn list_for_customer(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: CustomerId,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: ListDirection,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .audit()
            .record_entry(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
                true,
            )
            .await?;

        self.repo
            .list_for_customer_id_by_created_at(customer_id, query, direction)
            .await
    }

    #[record_error_severity]
    #[instrument(name = "credit.credit_facility.balance", skip(self, credit_facility_id), fields(credit_facility_id = tracing::field::Empty))]
    pub async fn balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<crate::CreditFacilityBalanceSummary, CreditFacilityError> {
        let id = credit_facility_id.into();
        tracing::Span::current().record("credit_facility_id", id.to_string());

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        let credit_facility = self.repo.find_by_id(id).await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        Ok(balances)
    }

    pub async fn has_outstanding_obligations(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
    ) -> Result<bool, CreditFacilityError> {
        let id = credit_facility_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        let credit_facility = self.repo.find_by_id(id).await?;

        if credit_facility
            .interest_accrual_cycle_in_progress()
            .is_some()
        {
            return Ok(true);
        }

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        Ok(balances.any_outstanding_or_defaulted())
    }
}

pub(crate) struct CompletedAccrualCycle {
    pub(crate) facility_accrual_cycle_data: CreditFacilityInterestAccrualCycle,
    pub(crate) new_cycle_data: Option<NewInterestAccrualCycleData>,
}
