mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent, primitives::*, CoreCreditAction, CoreCreditObject,
    CreditFacilityActivation, CreditLedger, InterestPeriod, Price,
};

pub use entity::CreditFacility;
pub(crate) use entity::*;
use error::CreditFacilityError;
pub use repo::{
    credit_facility_cursor::*, CreditFacilitiesSortBy, CreditFacilityRepo,
    FindManyCreditFacilities, ListDirection, Sort,
};

pub struct CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: CreditFacilityRepo<E>,
    authz: Perms,
    ledger: CreditLedger,
    price: Price,
}

impl<Perms, E> Clone for CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
        }
    }
}

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

impl<Perms, E> CreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        publisher: &crate::CreditFacilityPublisher<E>,
        ledger: &CreditLedger,
        price: &Price,
    ) -> Self {
        let repo = CreditFacilityRepo::new(pool, publisher);

        Self {
            repo,
            authz: authz.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CreditFacilityError> {
        Ok(self.repo.begin_op().await?)
    }

    pub(super) async fn find_by_id_without_audit(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        self.repo.find_by_id(id.into()).await
    }

    pub(super) async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_credit_facility: NewCreditFacility,
    ) -> Result<CreditFacility, CreditFacilityError> {
        self.repo.create_in_op(db, new_credit_facility).await
    }

    pub(super) async fn update_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility: &mut CreditFacility,
    ) -> Result<(), CreditFacilityError> {
        self.repo.update_in_op(db, credit_facility).await?;
        Ok(())
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

    #[instrument(name = "credit_facility.find", skip(self), err)]
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

    #[instrument(name = "credit_facility.list", skip(self), err)]
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

    #[instrument(
        name = "credit_facility.list_by_created_at_for_status",
        skip(self),
        err
    )]
    pub async fn list_by_created_at_for_status(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        status: CreditFacilityStatus,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        self.repo
            .list_for_status_by_created_at(status, query, direction.into())
            .await
    }

    #[instrument(
        name = "credit_facility.list_by_created_at_for_collateralization_state",
        skip(self),
        err
    )]
    pub async fn list_by_created_at_for_collateralization_state(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateralization_state: CollateralizationState,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        self.repo
            .list_for_collateralization_state_by_created_at(
                collateralization_state,
                query,
                direction.into(),
            )
            .await
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
        name = "credit_facility.list_by_collateralization_ratio",
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

    #[instrument(
        name = "credit_facility.list_by_collateralization_ratio_for_status",
        skip(self),
        err
    )]
    pub async fn list_by_collateralization_ratio_for_status(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        status: CreditFacilityStatus,
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
        self.repo
            .list_for_status_by_collateralization_ratio(status, query, direction.into())
            .await
    }

    #[instrument(
        name = "credit_facility.list_by_collateralization_ratio_for_collateralization_state",
        skip(self),
        err
    )]
    pub async fn list_by_collateralization_ratio_for_collateralization_state(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        collateralization_state: CollateralizationState,
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
        self.repo
            .list_for_collateralization_state_by_collateralization_ratio(
                collateralization_state,
                query,
                direction.into(),
            )
            .await
    }

    #[instrument(name = "credit_facility.find_all", skip(self), err)]
    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<std::collections::HashMap<CreditFacilityId, T>, CreditFacilityError> {
        self.repo.find_all(ids).await
    }

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
}
