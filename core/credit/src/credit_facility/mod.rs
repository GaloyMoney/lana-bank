mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditObject, CreditFacilityActivation, CreditLedger, InterestPeriod,
    Obligation, Obligations, Price, event::CoreCreditEvent, primitives::*,
};

pub use entity::CreditFacility;
pub(crate) use entity::*;

#[cfg(feature = "json-schema")]
pub use entity::CreditFacilityEvent;
use error::CreditFacilityError;
pub use repo::{
    CreditFacilitiesSortBy, CreditFacilityRepo, FindManyCreditFacilities, ListDirection, Sort,
    credit_facility_cursor::*,
};

pub struct CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: CreditFacilityRepo<E>,
    obligations: Obligations<Perms, E>,
    authz: Perms,
    ledger: CreditLedger,
    price: Price,
    governance: Governance<Perms, E>,
}

impl<Perms, E> Clone for CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            obligations: self.obligations.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            governance: self.governance.clone(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub(super) enum ActivationOutcome {
    Ignored(CreditFacility),
    Activated(ActivationData),
}

pub(super) struct ActivationData {
    pub credit_facility: CreditFacility,
    pub credit_facility_activation: CreditFacilityActivation,
    pub next_accrual_period: InterestPeriod,
    pub audit_info: audit::AuditInfo,
}

#[allow(clippy::large_enum_variant)]
pub(super) enum CompletionOutcome {
    Ignored(CreditFacility),
    Completed((CreditFacility, crate::CreditFacilityCompletion)),
}

#[derive(Clone)]
pub(super) struct ConfirmedAccrual {
    pub(super) accrual: super::CreditFacilityInterestAccrual,
    pub(super) next_period: Option<InterestPeriod>,
    pub(super) accrual_idx: InterestAccrualCycleIdx,
    pub(super) accrued_count: usize,
}

impl<Perms, E> CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        obligations: &Obligations<Perms, E>,
        ledger: &CreditLedger,
        price: &Price,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: &Governance<Perms, E>,
    ) -> Self {
        let repo = CreditFacilityRepo::new(pool, publisher);
        let _ = governance
            .init_policy(crate::APPROVE_CREDIT_FACILITY_PROCESS)
            .await;

        Self {
            repo,
            obligations: obligations.clone(),
            authz: authz.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            governance: governance.clone(),
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CreditFacilityError> {
        Ok(self.repo.begin_op().await?)
    }

    pub(super) async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_credit_facility: NewCreditFacility,
    ) -> Result<CreditFacility, CreditFacilityError> {
        self.governance
            .start_process(
                db,
                new_credit_facility.id,
                new_credit_facility.id.to_string(),
                crate::APPROVE_CREDIT_FACILITY_PROCESS,
            )
            .await?;
        self.repo.create_in_op(db, new_credit_facility).await
    }

    pub(super) async fn activate_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityId,
    ) -> Result<ActivationOutcome, CreditFacilityError> {
        let mut credit_facility = self.repo.find_by_id_in_tx(db.tx(), id).await?;
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_ACTIVATE,
            )
            .await?;
        let price = self.price.usd_cents_per_btc().await?;
        let now = db.now();
        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        let Ok(es_entity::Idempotent::Executed((credit_facility_activation, next_accrual_period))) =
            credit_facility.activate(now, price, balances, audit_info.clone())
        else {
            return Ok(ActivationOutcome::Ignored(credit_facility));
        };

        self.repo.update_in_op(db, &mut credit_facility).await?;

        Ok(ActivationOutcome::Activated(ActivationData {
            credit_facility,
            credit_facility_activation,
            next_accrual_period,
            audit_info,
        }))
    }

    pub(super) async fn approve(
        &self,
        id: CreditFacilityId,
        approved: bool,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut credit_facility = self.repo.find_by_id(id).await?;

        if credit_facility.is_approval_process_concluded() {
            return Ok(credit_facility);
        }

        let mut db = self.repo.begin_op().await?;
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::credit_facility(credit_facility.id),
                CoreCreditAction::CREDIT_FACILITY_CONCLUDE_APPROVAL_PROCESS,
            )
            .await?;

        if credit_facility
            .approval_process_concluded(approved, audit_info)
            .was_ignored()
        {
            return Ok(credit_facility);
        }

        self.repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        db.commit().await?;

        Ok(credit_facility)
    }

    pub(super) async fn confirm_interest_accrual_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityId,
    ) -> Result<ConfirmedAccrual, CreditFacilityError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let mut credit_facility = self.repo.find_by_id(id).await?;

        let confirmed_accrual = {
            let balances = self
                .ledger
                .get_credit_facility_balance(credit_facility.account_ids)
                .await?;

            let account_ids = credit_facility.account_ids;

            let accrual = credit_facility
                .interest_accrual_cycle_in_progress_mut()
                .expect("Accrual in progress should exist for scheduled job");

            let interest_accrual =
                accrual.record_accrual(balances.disbursed_outstanding(), audit_info);

            ConfirmedAccrual {
                accrual: (interest_accrual, account_ids).into(),
                next_period: accrual.next_accrual_period(),
                accrual_idx: accrual.idx,
                accrued_count: accrual.count_accrued(),
            }
        };

        self.repo.update_in_op(db, &mut credit_facility).await?;

        Ok(confirmed_accrual)
    }

    pub(super) async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityId,
        upgrade_buffer_cvl_pct: CVLPct,
        audit_info: &audit::AuditInfo,
    ) -> Result<CompletionOutcome, CreditFacilityError> {
        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self.repo.find_by_id(id).await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        let completion = if let es_entity::Idempotent::Executed(completion) =
            credit_facility.complete(audit_info.clone(), price, upgrade_buffer_cvl_pct, balances)?
        {
            completion
        } else {
            return Ok(CompletionOutcome::Ignored(credit_facility));
        };

        self.repo.update_in_op(db, &mut credit_facility).await?;

        Ok(CompletionOutcome::Completed((credit_facility, completion)))
    }

    pub(super) async fn complete_interest_cycle_and_maybe_start_new_cycle(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityId,
        audit_info: &audit::AuditInfo,
    ) -> Result<
        (
            Obligation,
            Option<(InterestAccrualCycleId, chrono::DateTime<chrono::Utc>)>,
        ),
        CreditFacilityError,
    > {
        let mut credit_facility = self.repo.find_by_id(id).await?;

        let new_obligation = if let es_entity::Idempotent::Executed(new_obligation) =
            credit_facility.record_interest_accrual_cycle(audit_info.clone())?
        {
            new_obligation
        } else {
            unreachable!("Should not be possible");
        };

        let obligation = self
            .obligations
            .create_with_jobs_in_op(db, new_obligation)
            .await?;

        let res = credit_facility.start_interest_accrual_cycle(audit_info.clone())?;
        self.repo.update_in_op(db, &mut credit_facility).await?;

        let new_cycle_data = res.map(|periods| {
            let new_accrual_cycle_id = credit_facility
                .interest_accrual_cycle_in_progress()
                .expect("First accrual cycle not found")
                .id;

            (new_accrual_cycle_id, periods.accrual.end)
        });

        Ok((obligation, new_cycle_data))
    }

    pub async fn find_by_id_without_audit(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        self.repo.find_by_id(id.into()).await
    }

    #[instrument(name = "core_credit.credit_facility.find", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(credit_facility) => Ok(Some(credit_facility)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub(super) async fn update_collateralization_from_price(
        &self,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<(), CreditFacilityError> {
        let price = self.price.usd_cents_per_btc().await?;
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
            let mut db = self.repo.begin_op().await?;
            let audit_info = self
                .authz
                .audit()
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
                    .update_collateralization(price, upgrade_buffer_cvl_pct, balances, &audit_info)
                    .did_execute()
                {
                    self.repo.update_in_op(&mut db, facility).await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                db.commit().await?;
            } else {
                break;
            }
        }
        Ok(())
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: CreditFacilityId,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut db = self.repo.begin_op().await?;
        let mut credit_facility = self.repo.find_by_id_in_tx(db.tx(), id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
            )
            .await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        let price = self.price.usd_cents_per_btc().await?;

        if credit_facility
            .update_collateralization(price, upgrade_buffer_cvl_pct, balances, &audit_info)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;

            db.commit().await?;
        }
        Ok(credit_facility)
    }

    #[instrument(name = "core_credit.credit_facility.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesCursor>,
        filter: FindManyCreditFacilities,
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
        self.repo.find_many(filter, sort.into(), query).await
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

    #[instrument(
        name = "core_credit.credit_facility.list_by_collateralization_ratio",
        skip(self),
        err
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

    #[instrument(name = "core_credit.credit_facility.find_all", skip(self), err)]
    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<std::collections::HashMap<CreditFacilityId, T>, CreditFacilityError> {
        self.repo.find_all(ids).await
    }

    #[instrument(
        name = "core_credit.credit_facility.list_for_customer",
        skip(self),
        err
    )]
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

    #[instrument(name = "core_credit.credit_facility.balance", skip(self), err)]
    pub async fn balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<crate::CreditFacilityBalanceSummary, CreditFacilityError> {
        let id = id.into();
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
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
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
