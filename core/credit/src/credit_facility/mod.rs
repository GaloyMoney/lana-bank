mod entity;
pub mod error;
pub mod interest_accrual_cycle;
mod jobs;
mod repo;

use std::sync::Arc;
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::{CorePriceEvent, Price};
use es_entity::clock::ClockHandle;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CoreCreditEvent, PublicIds,
    disbursal::Disbursals,
    ledger::{CreditFacilityInterestAccrual, CreditFacilityInterestAccrualCycle, CreditLedger},
    pending_credit_facility::{PendingCreditFacilities, PendingCreditFacilityCompletionOutcome},
    primitives::*,
};

use core_credit_collection::CoreCreditCollection;

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
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
    repo: Arc<CreditFacilityRepo<E>>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    disbursals: Arc<Disbursals<Perms, E>>,
    authz: Arc<Perms>,
    ledger: Arc<CreditLedger>,
    price: Arc<Price>,
    governance: Arc<Governance<Perms, E>>,
    public_ids: Arc<PublicIds>,
    credit_facility_maturity_job_spawner:
        jobs::credit_facility_maturity::CreditFacilityMaturityJobSpawner<E>,
    interest_accrual_job_spawner: jobs::interest_accrual::InterestAccrualJobSpawner<Perms, E>,
}

impl<Perms, E> Clone for CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            collections: self.collections.clone(),
            pending_credit_facilities: self.pending_credit_facilities.clone(),
            disbursals: self.disbursals.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            governance: self.governance.clone(),
            public_ids: self.public_ids.clone(),
            credit_facility_maturity_job_spawner: self.credit_facility_maturity_job_spawner.clone(),
            interest_accrual_job_spawner: self.interest_accrual_job_spawner.clone(),
        }
    }
}

pub(super) enum CompletionOutcome {
    AlreadyApplied(CreditFacility),
    Completed((CreditFacility, crate::CreditFacilityCompletion)),
}

impl<Perms, E> CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        collections: Arc<CoreCreditCollection<Perms, E>>,
        pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
        disbursals: Arc<Disbursals<Perms, E>>,
        ledger: Arc<CreditLedger>,
        price: Arc<Price>,
        jobs: &mut Jobs,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: Arc<Governance<Perms, E>>,
        public_ids: Arc<PublicIds>,
        outbox: &Outbox<E>,
        clock: ClockHandle,
    ) -> Result<Self, CreditFacilityError> {
        let repo = CreditFacilityRepo::new(pool, publisher, clock);
        let repo_arc = Arc::new(repo);

        let collateralization_from_events_spawner = jobs.add_initializer(
            jobs::collateralization_from_events::CreditFacilityCollateralizationFromEventsInit::<
                Perms,
                E,
            >::new(
                outbox,
                repo_arc.clone(),
                price.clone(),
                ledger.clone(),
                authz.clone(),
            ),
        );

        collateralization_from_events_spawner
            .spawn_unique(
                job::JobId::new(),
                jobs::collateralization_from_events::CreditFacilityCollateralizationFromEventsJobConfig::<E> {
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        let credit_facility_maturity_job_spawner = jobs.add_initializer(
            jobs::credit_facility_maturity::CreditFacilityMaturityInit::new(repo_arc.clone()),
        );

        let interest_accrual_job_spawner = jobs.add_initializer(
            jobs::interest_accrual::InterestAccrualJobInit::<Perms, E>::new(
                ledger.clone(),
                collections.clone(),
                repo_arc.clone(),
                authz.clone(),
            ),
        );

        Ok(Self {
            repo: repo_arc,
            collections,
            pending_credit_facilities,
            disbursals,
            authz,
            ledger,
            price,
            governance,
            public_ids,
            credit_facility_maturity_job_spawner,
            interest_accrual_job_spawner,
        })
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
            .record_system_entry_in_op(
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
            .expect("start_interest_accrual_cycle always returns Executed")
            .expect("first accrual");

        self.repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        self.credit_facility_maturity_job_spawner
            .spawn_at_in_op(
                &mut db,
                JobId::new(),
                // FIXME: I don't think this is updated if/when the facility is updated
                // if the credit product is closed earlier than expected or if is liquidated
                jobs::credit_facility_maturity::CreditFacilityMaturityJobConfig::<E> {
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

        self.interest_accrual_job_spawner
            .spawn_at_in_op(
                &mut db,
                accrual_id,
                jobs::interest_accrual::InterestAccrualJobConfig::<Perms, E> {
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
            .handle_activation_in_op(
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

#[derive(Clone)]
pub(crate) struct ConfirmedAccrual {
    pub(super) accrual: CreditFacilityInterestAccrual,
    pub(super) next_period: Option<InterestPeriod>,
    pub(super) accrual_idx: InterestAccrualCycleIdx,
    pub(super) accrued_count: usize,
}

pub(crate) struct CompletedAccrualCycle {
    pub(crate) facility_accrual_cycle_data: CreditFacilityInterestAccrualCycle,
    pub(crate) new_cycle_data: Option<NewInterestAccrualCycleData>,
}
