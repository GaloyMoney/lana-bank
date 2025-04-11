mod chart_of_accounts_integration;
mod config;
mod credit_facility;
mod disbursal;
pub mod error;
mod event;
mod for_subject;
mod interest_accrual_cycle;
mod jobs;
pub mod ledger;
mod obligation;
mod payment;
mod payment_allocator;
mod primitives;
mod processes;
mod publisher;
mod terms;
mod time;

use std::collections::HashMap;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use core_accounting::Chart;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_price::Price;
use es_entity::Idempotent;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};
use payment_allocator::{ObligationDataForAllocation, PaymentAllocationResult, PaymentAllocator};
use tracing::instrument;

pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
pub use config::*;
use credit_facility::error::CreditFacilityError;
pub use credit_facility::*;
pub use disbursal::{disbursal_cursor::*, *};
use error::*;
pub use event::*;
use for_subject::CreditFacilitiesForSubject;
pub use interest_accrual_cycle::*;
use jobs::*;
pub use ledger::*;
pub use obligation::{obligation_cursor::*, *};
pub use payment::*;
pub use primitives::*;
use processes::activate_credit_facility::*;
pub use processes::approve_credit_facility::*;
pub use processes::approve_disbursal::*;
use publisher::CreditFacilityPublisher;
pub use terms::*;

pub struct CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Perms,
    credit_facility_repo: CreditFacilityRepo<E>,
    obligation_repo: ObligationRepo<E>,
    disbursal_repo: DisbursalRepo,
    payment_repo: PaymentRepo,
    governance: Governance<Perms, E>,
    customer: Customers<Perms, E>,
    ledger: CreditLedger,
    price: Price,
    config: CreditConfig,
    approve_disbursal: ApproveDisbursal<Perms, E>,
    cala: CalaLedger,
    approve_credit_facility: ApproveCreditFacility<Perms, E>,
    obligations: Obligations<Perms, E>,
}

impl<Perms, E> Clone for CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            obligation_repo: self.obligation_repo.clone(),
            obligations: self.obligations.clone(),
            disbursal_repo: self.disbursal_repo.clone(),
            payment_repo: self.payment_repo.clone(),
            governance: self.governance.clone(),
            customer: self.customer.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            config: self.config.clone(),
            cala: self.cala.clone(),
            approve_disbursal: self.approve_disbursal.clone(),
            approve_credit_facility: self.approve_credit_facility.clone(),
        }
    }
}

impl<Perms, E> CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CustomerObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        pool: &sqlx::PgPool,
        config: CreditConfig,
        governance: &Governance<Perms, E>,
        jobs: &Jobs,
        authz: &Perms,
        customer: &Customers<Perms, E>,
        price: &Price,
        outbox: &Outbox<E>,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Result<Self, CoreCreditError> {
        let publisher = CreditFacilityPublisher::new(outbox);
        let credit_facility_repo = CreditFacilityRepo::new(pool, &publisher);
        let disbursal_repo = DisbursalRepo::new(pool);
        let obligation_repo = ObligationRepo::new(pool, &publisher);
        let obligations = Obligations::new(pool, authz, cala, &publisher);
        let payment_repo = PaymentRepo::new(pool);
        let ledger = CreditLedger::init(cala, journal_id).await?;
        let approve_disbursal = ApproveDisbursal::new(
            &disbursal_repo,
            &obligation_repo,
            &credit_facility_repo,
            jobs,
            authz.audit(),
            governance,
            &ledger,
        );

        let approve_credit_facility =
            ApproveCreditFacility::new(&credit_facility_repo, authz.audit(), governance);
        let activate_credit_facility = ActivateCreditFacility::new(
            &obligation_repo,
            &credit_facility_repo,
            &disbursal_repo,
            &ledger,
            price,
            jobs,
            authz.audit(),
        );

        jobs.add_initializer_and_spawn_unique(
            cvl::CreditFacilityProcessingJobInitializer::<Perms, E>::new(
                credit_facility_repo.clone(),
                &ledger,
                price,
                authz.audit(),
            ),
            cvl::CreditFacilityJobConfig {
                job_interval: std::time::Duration::from_secs(30),
                upgrade_buffer_cvl_pct: config.upgrade_buffer_cvl_pct,
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer(interest_accruals::CreditFacilityProcessingJobInitializer::<
            Perms,
            E,
        >::new(
            &ledger, credit_facility_repo.clone(), authz.audit(), jobs
        ));
        jobs.add_initializer(
            interest_accrual_cycles::CreditFacilityProcessingJobInitializer::<Perms, E>::new(
                &ledger,
                obligation_repo.clone(),
                credit_facility_repo.clone(),
                jobs,
                authz.audit(),
            ),
        );
        jobs.add_initializer(obligation_due::CreditFacilityProcessingJobInitializer::<
            Perms,
            E,
        >::new(
            &ledger, obligation_repo.clone(), jobs, authz.audit()
        ));
        jobs.add_initializer(
            obligation_overdue::CreditFacilityProcessingJobInitializer::<Perms, E>::new(
                &ledger,
                obligation_repo.clone(),
                authz.audit(),
            ),
        );
        jobs.add_initializer_and_spawn_unique(
            CreditFacilityApprovalJobInitializer::new(outbox, &approve_credit_facility),
            CreditFacilityApprovalJobConfig::<Perms, E>::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            DisbursalApprovalJobInitializer::new(outbox, &approve_disbursal),
            DisbursalApprovalJobConfig::<Perms, E>::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CreditFacilityActivationJobInitializer::new(outbox, &activate_credit_facility),
            CreditFacilityActivationJobConfig::<Perms, E>::new(),
        )
        .await?;
        let _ = governance
            .init_policy(APPROVE_CREDIT_FACILITY_PROCESS)
            .await;
        let _ = governance.init_policy(APPROVE_DISBURSAL_PROCESS).await;

        Ok(Self {
            authz: authz.clone(),
            customer: customer.clone(),
            credit_facility_repo,
            obligation_repo,
            obligations,
            disbursal_repo,
            payment_repo,
            governance: governance.clone(),
            ledger,
            price: price.clone(),
            config,
            cala: cala.clone(),
            approve_disbursal,
            approve_credit_facility,
        })
    }

    pub fn obligations(&self) -> &Obligations<Perms, E> {
        &self.obligations
    }

    pub async fn subject_can_create(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_CREATE,
                enforce,
            )
            .await?)
    }

    pub fn for_subject<'s>(
        &'s self,
        sub: &'s <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<CreditFacilitiesForSubject<'s, Perms, E>, CoreCreditError>
    where
        CustomerId: for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let customer_id =
            CustomerId::try_from(sub).map_err(|_| CoreCreditError::SubjectIsNotCustomer)?;
        Ok(CreditFacilitiesForSubject::new(
            sub,
            customer_id,
            &self.authz,
            &self.obligation_repo,
            &self.credit_facility_repo,
            &self.disbursal_repo,
            &self.payment_repo,
            &self.ledger,
        ))
    }

    #[instrument(name = "credit_facility.initiate", skip(self), err)]
    pub async fn initiate(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug + Copy,
        disbursal_credit_account_id: impl Into<CalaAccountId> + std::fmt::Debug,
        amount: UsdCents,
        terms: TermValues,
    ) -> Result<CreditFacility, CoreCreditError> {
        let audit_info = self
            .subject_can_create(sub, true)
            .await?
            .expect("audit info missing");

        let customer = self
            .customer
            .find_by_id(sub, customer_id)
            .await?
            .ok_or(CoreCreditError::CustomerNotFound)?;

        if self.config.customer_active_check_enabled && customer.status.is_inactive() {
            return Err(CoreCreditError::CustomerNotActive);
        }

        let id = CreditFacilityId::new();
        let new_credit_facility = NewCreditFacility::builder()
            .id(id)
            .approval_process_id(id)
            .customer_id(customer_id)
            .terms(terms)
            .amount(amount)
            .account_ids(CreditFacilityAccountIds::new())
            .disbursal_credit_account_id(disbursal_credit_account_id.into())
            .audit_info(audit_info.clone())
            .build()
            .expect("could not build new credit facility");

        let mut db = self.credit_facility_repo.begin_op().await?;
        self.governance
            .start_process(&mut db, id, id.to_string(), APPROVE_CREDIT_FACILITY_PROCESS)
            .await?;
        let credit_facility = self
            .credit_facility_repo
            .create_in_op(&mut db, new_credit_facility)
            .await?;

        let mut op = self.cala.ledger_operation_from_db_op(db);
        self.ledger
            .create_accounts_for_credit_facility(
                &mut op,
                credit_facility.id,
                credit_facility.account_ids,
                customer.customer_type,
                terms.duration.duration_type(),
            )
            .await?;

        self.ledger
            .add_credit_facility_control_to_account(
                &mut op,
                credit_facility.account_ids.facility_account_id,
            )
            .await?;

        op.commit().await?;

        Ok(credit_facility)
    }

    #[instrument(name = "credit_facility.find", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacility>, CoreCreditError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        match self.credit_facility_repo.find_by_id(id).await {
            Ok(credit_facility) => Ok(Some(credit_facility)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "credit_facility.balance", skip(self), err)]
    pub async fn balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacilityBalanceSummary, CoreCreditError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        let credit_facility = self.credit_facility_repo.find_by_id(id).await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        Ok(balances)
    }

    pub async fn subject_can_initiate_disbursal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::DISBURSAL_INITIATE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "credit_facility.initiate_disbursal", skip(self), err)]
    #[es_entity::retry_on_concurrent_modification]
    pub async fn initiate_disbursal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<Disbursal, CoreCreditError> {
        let audit_info = self
            .subject_can_initiate_disbursal(sub, true)
            .await?
            .expect("audit info missing");

        let facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let customer_id = facility.customer_id;
        let customer = self
            .customer
            .find_by_id(sub, customer_id)
            .await?
            .ok_or(CoreCreditError::CustomerNotFound)?;
        if self.config.customer_active_check_enabled && customer.status.is_inactive() {
            return Err(CoreCreditError::CustomerNotActive);
        }

        let now = crate::time::now();
        if !facility.can_initiate_disbursal(now) {
            return Err(CreditFacilityError::DisbursalPastMaturityDate.into());
        }
        let balance = self
            .ledger
            .get_credit_facility_balance(facility.account_ids)
            .await?;

        let outstanding = CreditFacilityReceivable::from(balance);

        let price = self.price.usd_cents_per_btc().await?;
        outstanding
            .with_added_disbursal_amount(amount)
            .facility_cvl_data(facility.collateral(), balance.facility_remaining)
            .cvl(price)
            .check_disbursal_allowed(facility.terms)?;

        let mut db = self.credit_facility_repo.begin_op().await?;
        let new_disbursal = NewDisbursal::builder()
            .id(DisbursalId::new())
            .approval_process_id(ApprovalProcessId::new())
            .credit_facility_id(credit_facility_id)
            .amount(amount)
            .account_ids(facility.account_ids)
            .disbursal_credit_account_id(facility.disbursal_credit_account_id)
            .disbursal_due_date(facility.activated_at().expect("Facility is not active"))
            .audit_info(audit_info)
            .build()
            .expect("could not build new disbursal");

        self.governance
            .start_process(
                &mut db,
                new_disbursal.approval_process_id,
                new_disbursal.approval_process_id.to_string(),
                APPROVE_DISBURSAL_PROCESS,
            )
            .await?;
        let disbursal = self
            .disbursal_repo
            .create_in_op(&mut db, new_disbursal)
            .await?;

        self.ledger
            .initiate_disbursal(
                db,
                disbursal.id,
                disbursal.amount,
                disbursal.account_ids.facility_account_id,
            )
            .await?;

        Ok(disbursal)
    }

    #[instrument(name = "credit_facility.find_disbursal_by_id", skip(self), err)]
    pub async fn find_disbursal_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DisbursalId> + std::fmt::Debug,
    ) -> Result<Option<Disbursal>, CoreCreditError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        match self.disbursal_repo.find_by_id(id.into()).await {
            Ok(loan) => Ok(Some(loan)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn find_disbursal_by_concluded_tx_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        tx_id: impl Into<crate::primitives::LedgerTxId> + std::fmt::Debug,
    ) -> Result<Disbursal, CoreCreditError> {
        let tx_id = tx_id.into();
        let disbursal = self
            .disbursal_repo
            .find_by_concluded_tx_id(Some(tx_id))
            .await?;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        Ok(disbursal)
    }

    pub async fn ensure_up_to_date_disbursal_status(
        &self,
        disbursal: &Disbursal,
    ) -> Result<Option<Disbursal>, CoreCreditError> {
        self.approve_disbursal.execute_from_svc(disbursal).await
    }

    pub async fn ensure_up_to_date_status(
        &self,
        credit_facility: &CreditFacility,
    ) -> Result<Option<CreditFacility>, CoreCreditError> {
        self.approve_credit_facility
            .execute_from_svc(credit_facility)
            .await
    }

    pub async fn subject_can_update_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERAL,
                enforce,
            )
            .await?)
    }

    #[es_entity::retry_on_concurrent_modification]
    #[instrument(name = "credit_facility.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: CreditFacilityId,
        updated_collateral: Satoshis,
    ) -> Result<CreditFacility, CoreCreditError> {
        let audit_info = self
            .subject_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        let mut db = self.credit_facility_repo.begin_op().await?;
        let credit_facility_collateral_update = credit_facility.record_collateral_update(
            updated_collateral,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
            balances,
        )?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        self.ledger
            .update_credit_facility_collateral(db, credit_facility_collateral_update)
            .await?;

        Ok(credit_facility)
    }

    pub async fn subject_can_record_payment(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_obligations(),
                CoreCreditAction::OBLIGATION_RECORD_PAYMENT,
                enforce,
            )
            .await?)
    }

    async fn list_obligations_for_credit_facility(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Vec<Obligation>, CoreCreditError> {
        let mut obligations = vec![];
        let mut query = Default::default();
        loop {
            let mut res = self
                .obligation_repo
                .list_for_credit_facility_id_by_created_at(
                    credit_facility_id,
                    query,
                    ListDirection::Ascending,
                )
                .await?;

            obligations.append(&mut res.entities);

            if let Some(q) = res.into_next_query() {
                query = q;
            } else {
                break;
            };
        }

        Ok(obligations)
    }

    #[instrument(name = "credit_facility.record_payment", skip(self), err)]
    pub async fn record_payment(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<CreditFacility, CoreCreditError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let obligations = self
            .list_obligations_for_credit_facility(credit_facility_id)
            .await?;

        let mut db = self.obligation_repo.begin_op().await?;
        let audit_info = self
            .subject_can_record_payment(sub, true)
            .await?
            .expect("audit info missing");

        let new_payment = NewPayment::builder()
            .id(PaymentId::new())
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .build()
            .expect("could not build new payment");
        let mut payment = self.payment_repo.create_in_op(&mut db, new_payment).await?;

        let PaymentAllocationResult {
            new_allocations,
            disbursal_amount,
            interest_amount,
        } = PaymentAllocator::new(payment.id, amount).allocate(
            obligations
                .iter()
                .map(ObligationDataForAllocation::from)
                .collect::<Vec<_>>(),
        )?;

        payment
            .record_allocated(disbursal_amount, interest_amount, audit_info.clone())
            .did_execute();
        self.payment_repo
            .update_in_op(&mut db, &mut payment)
            .await?;

        let now = crate::time::now();
        let mut updated_obligations = vec![];
        for mut obligation in obligations {
            if let Some(new_allocation) = new_allocations
                .iter()
                .find(|new_allocation| new_allocation.obligation_id == obligation.id)
            {
                obligation
                    .record_payment(
                        new_allocation.id,
                        new_allocation.amount,
                        now,
                        audit_info.clone(),
                    )
                    .did_execute();
                credit_facility
                    .update_balance_from_payment(
                        new_allocation.id,
                        obligation.obligation_type(),
                        new_allocation.amount,
                        now,
                        audit_info.clone(),
                    )
                    .did_execute();
                updated_obligations.push(obligation);
            }
        }

        // TODO: remove n+1
        for mut obligation in updated_obligations {
            self.obligation_repo
                .update_in_op(&mut db, &mut obligation)
                .await?;
        }

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        self.ledger
            .record_obligation_repayments(db, new_allocations)
            .await?;

        Ok(credit_facility)
    }

    #[instrument(name = "credit_facility.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesCursor>,
        filter: FindManyCreditFacilities,
        sort: impl Into<Sort<CreditFacilitiesSortBy>> + std::fmt::Debug,
    ) -> Result<es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesCursor>, CoreCreditError>
    {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        Ok(self
            .credit_facility_repo
            .find_many(filter, sort.into(), query)
            .await?)
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
        CoreCreditError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        Ok(self
            .credit_facility_repo
            .list_for_status_by_created_at(status, query, direction.into())
            .await?)
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
        CoreCreditError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        Ok(self
            .credit_facility_repo
            .list_for_collateralization_state_by_created_at(
                collateralization_state,
                query,
                direction.into(),
            )
            .await?)
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
        CoreCreditError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        Ok(self
            .credit_facility_repo
            .list_by_collateralization_ratio(query, direction.into())
            .await?)
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
        CoreCreditError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        Ok(self
            .credit_facility_repo
            .list_for_status_by_collateralization_ratio(status, query, direction.into())
            .await?)
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
        CoreCreditError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;
        Ok(self
            .credit_facility_repo
            .list_for_collateralization_state_by_collateralization_ratio(
                collateralization_state,
                query,
                direction.into(),
            )
            .await?)
    }

    pub async fn subject_can_complete(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_COMPLETE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "credit_facility.complete", skip(self), err)]
    pub async fn complete_facility(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CoreCreditError> {
        let credit_facility_id = credit_facility_id.into();

        let audit_info = self
            .subject_can_complete(sub, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        let completion = if let Idempotent::Executed(completion) = credit_facility.complete(
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
            balances,
        )? {
            completion
        } else {
            return Ok(credit_facility);
        };

        let mut db = self.credit_facility_repo.begin_op().await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        self.ledger.complete_credit_facility(db, completion).await?;

        Ok(credit_facility)
    }

    pub async fn find_payment_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        payment_id: impl Into<PaymentId> + std::fmt::Debug,
    ) -> Result<Payment, CoreCreditError> {
        let payment = self.payment_repo.find_by_id(payment_id.into()).await?;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        Ok(payment)
    }

    pub async fn list_disbursals(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<DisbursalsCursor>,
        filter: FindManyDisbursals,
        sort: impl Into<Sort<DisbursalsSortBy>>,
    ) -> Result<es_entity::PaginatedQueryRet<Disbursal, DisbursalsCursor>, CoreCreditError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::DISBURSAL_LIST,
            )
            .await?;

        let disbursals = self
            .disbursal_repo
            .find_many(filter, sort.into(), query)
            .await?;
        Ok(disbursals)
    }

    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<HashMap<CreditFacilityId, T>, CoreCreditError> {
        Ok(self.credit_facility_repo.find_all(ids).await?)
    }

    pub async fn find_all_disbursals<T: From<Disbursal>>(
        &self,
        ids: &[DisbursalId],
    ) -> Result<HashMap<DisbursalId, T>, CoreCreditError> {
        Ok(self.disbursal_repo.find_all(ids).await?)
    }

    pub async fn can_be_completed(&self, entity: &CreditFacility) -> Result<bool, CoreCreditError> {
        Ok(self.outstanding(entity).await?.is_zero())
    }

    pub async fn facility_cvl(
        &self,
        entity: &CreditFacility,
    ) -> Result<FacilityCVL, CoreCreditError> {
        let balances = self
            .ledger
            .get_credit_facility_balance(entity.account_ids)
            .await?;
        let price = self.price.usd_cents_per_btc().await?;
        Ok(entity.facility_cvl_data(balances).cvl(price))
    }

    pub async fn outstanding(&self, entity: &CreditFacility) -> Result<UsdCents, CoreCreditError> {
        let balances = self
            .ledger
            .get_credit_facility_balance(entity.account_ids)
            .await?;
        Ok(balances.total_outstanding())
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, CoreCreditError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::chart_of_accounts_integration(),
                CoreCreditAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ,
            )
            .await?;
        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?)
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, CoreCreditError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(CoreCreditError::ChartIdMismatch);
        }

        if self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?
            .is_some()
        {
            return Err(CoreCreditError::CreditConfigAlreadyExists);
        }

        let facility_omnibus_parent_account_set_id = chart
            .account_set_id_from_code(&config.chart_of_account_facility_omnibus_parent_code)?;
        let collateral_omnibus_parent_account_set_id = chart
            .account_set_id_from_code(&config.chart_of_account_collateral_omnibus_parent_code)?;
        let facility_parent_account_set_id =
            chart.account_set_id_from_code(&config.chart_of_account_facility_parent_code)?;
        let collateral_parent_account_set_id =
            chart.account_set_id_from_code(&config.chart_of_account_collateral_parent_code)?;
        let interest_income_parent_account_set_id =
            chart.account_set_id_from_code(&config.chart_of_account_interest_income_parent_code)?;
        let fee_income_parent_account_set_id =
            chart.account_set_id_from_code(&config.chart_of_account_fee_income_parent_code)?;

        let short_term_individual_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_short_term_individual_disbursed_receivable_parent_code,
            )?;
        let short_term_government_entity_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
            )?;
        let short_term_private_company_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
            )?;
        let short_term_bank_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_short_term_bank_disbursed_receivable_parent_code,
            )?;
        let short_term_financial_institution_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
        )?;
        let short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            )?;
        let short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,
        )?;

        let long_term_individual_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_long_term_individual_disbursed_receivable_parent_code,
            )?;
        let long_term_government_entity_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
            )?;
        let long_term_private_company_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
            )?;
        let long_term_bank_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_long_term_bank_disbursed_receivable_parent_code,
            )?;
        let long_term_financial_institution_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
        )?;
        let long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            )?;
        let long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,
        )?;

        let short_term_individual_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_short_term_individual_interest_receivable_parent_code,
            )?;
        let short_term_government_entity_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_short_term_government_entity_interest_receivable_parent_code,
            )?;
        let short_term_private_company_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_short_term_private_company_interest_receivable_parent_code,
            )?;
        let short_term_bank_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_short_term_bank_interest_receivable_parent_code,
            )?;
        let short_term_financial_institution_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
        )?;
        let short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            )?;
        let short_term_non_domiciled_company_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,
        )?;

        let long_term_individual_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_long_term_individual_interest_receivable_parent_code,
            )?;
        let long_term_government_entity_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_long_term_government_entity_interest_receivable_parent_code,
            )?;
        let long_term_private_company_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_long_term_private_company_interest_receivable_parent_code,
            )?;
        let long_term_bank_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_long_term_bank_interest_receivable_parent_code,
            )?;
        let long_term_financial_institution_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
        )?;
        let long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            )?;
        let long_term_non_domiciled_company_interest_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config
                .chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,
        )?;

        let overdue_individual_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_overdue_individual_disbursed_receivable_parent_code,
            )?;
        let overdue_government_entity_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
            )?;
        let overdue_private_company_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
            )?;
        let overdue_bank_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_overdue_bank_disbursed_receivable_parent_code,
            )?;
        let overdue_financial_institution_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config.chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
        )?;
        let overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id = chart
        .account_set_id_from_code(
            &config
                .chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
        )?;
        let overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id = chart
            .account_set_id_from_code(
            &config.chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
        )?;

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCreditObject::chart_of_accounts_integration(),
                CoreCreditAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        let charts_integration_meta = ChartOfAccountsIntegrationMeta {
            audit_info,
            config: config.clone(),

            facility_omnibus_parent_account_set_id,
            collateral_omnibus_parent_account_set_id,
            facility_parent_account_set_id,
            collateral_parent_account_set_id,
            interest_income_parent_account_set_id,
            fee_income_parent_account_set_id,

            short_term_disbursed_integration_meta: ShortTermDisbursedIntegrationMeta {
                short_term_individual_disbursed_receivable_parent_account_set_id,
                short_term_government_entity_disbursed_receivable_parent_account_set_id,
                short_term_private_company_disbursed_receivable_parent_account_set_id,
                short_term_bank_disbursed_receivable_parent_account_set_id,
                short_term_financial_institution_disbursed_receivable_parent_account_set_id,
                short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
                short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id,
            },

            long_term_disbursed_integration_meta: LongTermDisbursedIntegrationMeta {
                long_term_individual_disbursed_receivable_parent_account_set_id,
                long_term_government_entity_disbursed_receivable_parent_account_set_id,
                long_term_private_company_disbursed_receivable_parent_account_set_id,
                long_term_bank_disbursed_receivable_parent_account_set_id,
                long_term_financial_institution_disbursed_receivable_parent_account_set_id,
                long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
                long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id,
            },

            short_term_interest_integration_meta: ShortTermInterestIntegrationMeta {
                short_term_individual_interest_receivable_parent_account_set_id,
                short_term_government_entity_interest_receivable_parent_account_set_id,
                short_term_private_company_interest_receivable_parent_account_set_id,
                short_term_bank_interest_receivable_parent_account_set_id,
                short_term_financial_institution_interest_receivable_parent_account_set_id,
                short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id,
                short_term_non_domiciled_company_interest_receivable_parent_account_set_id,
            },

            long_term_interest_integration_meta: LongTermInterestIntegrationMeta {
                long_term_individual_interest_receivable_parent_account_set_id,
                long_term_government_entity_interest_receivable_parent_account_set_id,
                long_term_private_company_interest_receivable_parent_account_set_id,
                long_term_bank_interest_receivable_parent_account_set_id,
                long_term_financial_institution_interest_receivable_parent_account_set_id,
                long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id,
                long_term_non_domiciled_company_interest_receivable_parent_account_set_id,
            },

            overdue_disbursed_integration_meta: OverdueDisbursedIntegrationMeta {
                overdue_individual_disbursed_receivable_parent_account_set_id,
                overdue_government_entity_disbursed_receivable_parent_account_set_id,
                overdue_private_company_disbursed_receivable_parent_account_set_id,
                overdue_bank_disbursed_receivable_parent_account_set_id,
                overdue_financial_institution_disbursed_receivable_parent_account_set_id,
                overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
                overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id,
            },
        };

        self.ledger
            .attach_chart_of_accounts_account_sets(charts_integration_meta)
            .await?;

        Ok(config)
    }
}
