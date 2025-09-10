mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::{JobId, Jobs};
use outbox::OutboxEventMarker;

use crate::{
    PublicIds,
    credit_facility_proposal::{CreditFacilityProposalCompletionOutcome, CreditFacilityProposals},
    event::CoreCreditEvent,
    interest_accrual_cycle::NewInterestAccrualCycleData,
    jobs::credit_facility_maturity,
    ledger::{CreditFacilityInterestAccrual, CreditFacilityInterestAccrualCycle, CreditLedger},
    obligation::Obligations,
    primitives::*,
    terms::InterestPeriod,
};

pub use entity::CreditFacility;
pub(crate) use entity::*;

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
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: CreditFacilityRepo<E>,
    obligations: Obligations<Perms, E>,
    proposals: CreditFacilityProposals<Perms, E>,
    authz: Perms,
    ledger: CreditLedger,
    price: Price,
    jobs: Jobs,
    governance: Governance<Perms, E>,
    public_ids: PublicIds,
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
            proposals: self.proposals.clone(),
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
    Ignored(CreditFacility),
    Completed((CreditFacility, crate::CreditFacilityCompletion)),
}

pub(super) enum ActivationOutcome {
    Ignored,
    Activated(ActivationData),
}

pub struct ActivationData {
    pub credit_facility: CreditFacility,
    pub next_accrual_period: InterestPeriod,
    pub approval_process_id: ApprovalProcessId,
    pub structuring_fee: UsdCents,
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
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        obligations: &Obligations<Perms, E>,
        proposals: &CreditFacilityProposals<Perms, E>,
        ledger: &CreditLedger,
        price: &Price,
        jobs: &Jobs,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: &Governance<Perms, E>,
        public_ids: &PublicIds,
    ) -> Result<Self, CreditFacilityError> {
        let repo = CreditFacilityRepo::new(pool, publisher);

        Ok(Self {
            repo,
            obligations: obligations.clone(),
            proposals: proposals.clone(),
            authz: authz.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            jobs: jobs.clone(),
            governance: governance.clone(),
            public_ids: public_ids.clone(),
        })
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CreditFacilityError> {
        Ok(self.repo.begin_op().await?)
    }

    #[instrument(name = "credit.credit_facility.activate", skip(self, db), err)]
    pub(super) async fn activate_in_op(
        &self,
        db: &mut es_entity::DbOpWithTime<'_>,
        id: CreditFacilityId,
    ) -> Result<ActivationOutcome, CreditFacilityError> {
        self.authz
            .audit()
            .record_system_entry_in_tx(
                db,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_ACTIVATE,
            )
            .await?;

        let proposal = match self.proposals.complete_in_op(db, id.into()).await? {
            CreditFacilityProposalCompletionOutcome::Completed(proposal) => proposal,
            CreditFacilityProposalCompletionOutcome::Ignored => {
                return Ok(ActivationOutcome::Ignored);
            }
        };

        let public_id = self
            .public_ids
            .create_in_op(db, CREDIT_FACILITY_REF_TARGET, id)
            .await?;

        let new_credit_facility = NewCreditFacility::builder()
            .id(id)
            .ledger_tx_id(LedgerTxId::new())
            .customer_id(proposal.customer_id)
            .customer_type(proposal.customer_type)
            .account_ids(crate::CreditFacilityLedgerAccountIds::from(
                proposal.account_ids,
            ))
            .disbursal_credit_account_id(proposal.disbursal_credit_account_id)
            .collateral_id(proposal.collateral_id)
            .terms(proposal.terms)
            .amount(proposal.amount)
            .activated_at(crate::time::now())
            .maturity_date(proposal.terms.maturity_date(crate::time::now()))
            .public_id(public_id.id)
            .build()
            .expect("could not build new credit facility");

        let mut credit_facility = self.repo.create_in_op(db, new_credit_facility).await?;
        let structuring_fee = credit_facility.structuring_fee();

        let periods = credit_facility
            .start_interest_accrual_cycle()?
            .expect("first accrual");

        self.repo.update_in_op(db, &mut credit_facility).await?;

        self.jobs
            .create_and_spawn_at_in_op(
                db,
                JobId::new(),
                credit_facility_maturity::CreditFacilityMaturityJobConfig::<Perms, E> {
                    credit_facility_id: credit_facility.id,
                    _phantom: std::marker::PhantomData,
                },
                credit_facility.matures_at(),
            )
            .await?;

        Ok(ActivationOutcome::Activated(ActivationData {
            credit_facility,
            next_accrual_period: periods.accrual,
            approval_process_id: proposal.approval_process_id,
            structuring_fee,
        }))
    }

    pub(super) async fn confirm_interest_accrual_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        id: CreditFacilityId,
    ) -> Result<ConfirmedAccrual, CreditFacilityError> {
        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let mut credit_facility = self.repo.find_by_id(id).await?;

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

    pub(super) async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityId,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<CompletionOutcome, CreditFacilityError> {
        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self.repo.find_by_id(id).await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        let completion = if let es_entity::Idempotent::Executed(completion) =
            credit_facility.complete(price, upgrade_buffer_cvl_pct, balances)?
        {
            completion
        } else {
            return Ok(CompletionOutcome::Ignored(credit_facility));
        };

        self.repo.update_in_op(db, &mut credit_facility).await?;

        Ok(CompletionOutcome::Completed((credit_facility, completion)))
    }

    #[instrument(
        name = "credit.facility.complete_interest_cycle_and_maybe_start_new_cycle",
        skip(self, db)
    )]
    pub(super) async fn complete_interest_cycle_and_maybe_start_new_cycle(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityId,
    ) -> Result<CompletedAccrualCycle, CreditFacilityError> {
        let mut credit_facility = self.repo.find_by_id(id).await?;

        let (accrual_cycle_data, new_obligation) = if let es_entity::Idempotent::Executed(res) =
            credit_facility.record_interest_accrual_cycle()?
        {
            res
        } else {
            unreachable!("Should not be possible");
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

    #[instrument(name = "credit.credit_facility.find_by_id", skip(self), err)]
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

    pub(super) async fn mark_facility_as_matured(
        &self,
        id: CreditFacilityId,
    ) -> Result<(), CreditFacilityError> {
        let mut facility = self.repo.find_by_id(id).await?;

        if facility.mature().did_execute() {
            self.repo.update(&mut facility).await?;
        }

        Ok(())
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

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: CreditFacilityId,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut op = self.repo.begin_op().await?;
        let mut credit_facility = self.repo.find_by_id_in_op(&mut op, id).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
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
            .update_collateralization(price, upgrade_buffer_cvl_pct, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut credit_facility)
                .await?;

            op.commit().await?;
        }
        Ok(credit_facility)
    }

    #[instrument(name = "credit.credit_facility.list", skip(self), err)]
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

    #[instrument(
        name = "credit.credit_facility.list_by_collateralization_ratio",
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

    #[instrument(name = "credit.credit_facility.find_all", skip(self), err)]
    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<std::collections::HashMap<CreditFacilityId, T>, CreditFacilityError> {
        self.repo.find_all(ids).await
    }

    #[instrument(name = "credit.credit_facility.list_for_customer", skip(self), err)]
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

    #[instrument(name = "credit.credit_facility.find_by_wallet", skip(self), err)]
    pub async fn find_by_custody_wallet(
        &self,
        custody_wallet_id: impl Into<CustodyWalletId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        self.repo
            .find_by_custody_wallet(custody_wallet_id.into())
            .await
    }

    #[instrument(name = "credit.credit_facility.balance", skip(self), err)]
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

    #[es_entity::retry_on_concurrent_modification(any_error = true, max_retries = 15)]
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
