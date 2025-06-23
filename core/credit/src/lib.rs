mod chart_of_accounts_integration;
mod collateral;
mod config;
mod credit_facility;
mod disbursal;
pub mod error;
mod event;
mod for_subject;
mod history;
mod interest_accrual_cycle;
mod jobs;
pub mod ledger;
mod liquidation_process;
mod obligation;
mod payment;
mod payment_allocation;
mod primitives;
mod processes;
mod publisher;
mod repayment_plan;
mod terms;
mod terms_template;
mod time;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};
use tracing::instrument;

pub use chart_of_accounts_integration::{
    ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrationConfigBuilderError,
    ChartOfAccountsIntegrations, error::ChartOfAccountsIntegrationError,
};
pub use collateral::*;
pub use config::*;
pub use credit_facility::error::CreditFacilityError;
pub use credit_facility::*;
pub use disbursal::{disbursal_cursor::*, *};
use error::*;
pub use event::*;
use for_subject::CreditFacilitiesForSubject;
pub use history::*;
pub use interest_accrual_cycle::*;
use jobs::*;
pub use ledger::*;
pub use obligation::{error::*, obligation_cursor::*, *};
pub use payment::*;
pub use payment_allocation::*;
pub use primitives::*;
use processes::activate_credit_facility::*;
pub use processes::approve_credit_facility::*;
pub use processes::approve_disbursal::*;
use publisher::CreditFacilityPublisher;
pub use repayment_plan::*;
pub use terms::*;
pub use terms_template::{error as terms_template_error, *};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::{
        TermsTemplateEvent, collateral::CollateralEvent, credit_facility::CreditFacilityEvent,
        disbursal::DisbursalEvent, interest_accrual_cycle::InterestAccrualCycleEvent,
        liquidation_process::LiquidationProcessEvent, obligation::ObligationEvent,
        payment::PaymentEvent, payment_allocation::PaymentAllocationEvent,
    };
}

pub struct CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Perms,
    facilities: CreditFacilities<Perms, E>,
    disbursals: Disbursals<Perms, E>,
    payments: Payments<Perms, E>,
    history_repo: HistoryRepo,
    repayment_plan_repo: RepaymentPlanRepo,
    governance: Governance<Perms, E>,
    customer: Customers<Perms, E>,
    ledger: CreditLedger,
    price: Price,
    config: CreditConfig,
    approve_disbursal: ApproveDisbursal<Perms, E>,
    cala: CalaLedger,
    approve_credit_facility: ApproveCreditFacility<Perms, E>,
    obligations: Obligations<Perms, E>,
    collaterals: Collaterals<Perms, E>,
    chart_of_accounts_integrations: ChartOfAccountsIntegrations<Perms>,
    terms_templates: TermsTemplates<Perms>,
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
            facilities: self.facilities.clone(),
            obligations: self.obligations.clone(),
            collaterals: self.collaterals.clone(),
            disbursals: self.disbursals.clone(),
            payments: self.payments.clone(),
            history_repo: self.history_repo.clone(),
            repayment_plan_repo: self.repayment_plan_repo.clone(),
            governance: self.governance.clone(),
            customer: self.customer.clone(),
            ledger: self.ledger.clone(),
            price: self.price.clone(),
            config: self.config.clone(),
            cala: self.cala.clone(),
            approve_disbursal: self.approve_disbursal.clone(),
            approve_credit_facility: self.approve_credit_facility.clone(),
            chart_of_accounts_integrations: self.chart_of_accounts_integrations.clone(),
            terms_templates: self.terms_templates.clone(),
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
        let ledger = CreditLedger::init(cala, journal_id).await?;
        let obligations = Obligations::new(pool, authz, cala, jobs, &publisher);
        let credit_facilities = CreditFacilities::new(
            pool,
            authz,
            &obligations,
            &ledger,
            price,
            &publisher,
            governance,
        )
        .await;
        let collaterals = Collaterals::new(pool, authz, &publisher);
        let disbursals = Disbursals::new(pool, authz, &publisher, &obligations, governance).await;
        let payments = Payments::new(pool, authz, &obligations, &publisher);
        let history_repo = HistoryRepo::new(pool);
        let repayment_plan_repo = RepaymentPlanRepo::new(pool);
        let approve_disbursal =
            ApproveDisbursal::new(&disbursals, &credit_facilities, jobs, governance, &ledger);

        let approve_credit_facility =
            ApproveCreditFacility::new(&credit_facilities, authz.audit(), governance);
        let activate_credit_facility = ActivateCreditFacility::new(
            &credit_facilities,
            &disbursals,
            &ledger,
            price,
            jobs,
            authz.audit(),
        );
        let chart_of_accounts_integrations = ChartOfAccountsIntegrations::new(authz, &ledger);
        let terms_templates = TermsTemplates::new(pool, authz);

        jobs.add_initializer_and_spawn_unique(
            collateralization_from_price::CreditFacilityCollateralizationFromPriceJobInitializer::<
                Perms,
                E,
            >::new(credit_facilities.clone()),
            collateralization_from_price::CreditFacilityCollateralizationFromPriceJobConfig {
                job_interval: std::time::Duration::from_secs(30),
                upgrade_buffer_cvl_pct: config.upgrade_buffer_cvl_pct,
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            collateralization_from_events::CreditFacilityCollateralizationFromEventsInitializer::<
                Perms,
                E,
            >::new(outbox, &credit_facilities),
            collateralization_from_events::CreditFacilityCollateralizationFromEventsJobConfig {
                upgrade_buffer_cvl_pct: config.upgrade_buffer_cvl_pct,
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            credit_facility_history::HistoryProjectionInitializer::<E>::new(outbox, &history_repo),
            credit_facility_history::HistoryProjectionConfig {
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            credit_facility_repayment_plan::RepaymentPlanProjectionInitializer::<E>::new(
                outbox,
                &repayment_plan_repo,
            ),
            credit_facility_repayment_plan::RepaymentPlanProjectionConfig {
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer(
            interest_accruals::InterestAccrualJobInitializer::<Perms, E>::new(
                &ledger,
                &credit_facilities,
                jobs,
            ),
        );
        jobs.add_initializer(
            interest_accrual_cycles::InterestAccrualCycleJobInitializer::<Perms, E>::new(
                &ledger,
                &obligations,
                &credit_facilities,
                jobs,
                authz.audit(),
            ),
        );
        jobs.add_initializer(
            obligation_due::ObligationDueJobInitializer::<Perms, E>::new(
                &ledger,
                &obligations,
                jobs,
            ),
        );
        jobs.add_initializer(obligation_overdue::ObligationOverdueJobInitializer::<
            Perms,
            E,
        >::new(&ledger, &obligations, jobs));
        jobs.add_initializer(
            obligation_liquidation::ObligationLiquidationJobInitializer::<Perms, E>::new(
                &ledger,
                &obligations,
                jobs,
            ),
        );
        jobs.add_initializer(obligation_defaulted::ObligationDefaultedJobInitializer::<
            Perms,
            E,
        >::new(&ledger, &obligations));
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

        Ok(Self {
            authz: authz.clone(),
            customer: customer.clone(),
            facilities: credit_facilities,
            obligations,
            collaterals,
            disbursals,
            payments,
            history_repo,
            repayment_plan_repo,
            governance: governance.clone(),
            ledger,
            price: price.clone(),
            config,
            cala: cala.clone(),
            approve_disbursal,
            approve_credit_facility,
            chart_of_accounts_integrations,
            terms_templates,
        })
    }

    pub fn obligations(&self) -> &Obligations<Perms, E> {
        &self.obligations
    }

    pub fn collaterals(&self) -> &Collaterals<Perms, E> {
        &self.collaterals
    }

    pub fn disbursals(&self) -> &Disbursals<Perms, E> {
        &self.disbursals
    }

    pub fn facilities(&self) -> &CreditFacilities<Perms, E> {
        &self.facilities
    }

    pub fn payments(&self) -> &Payments<Perms, E> {
        &self.payments
    }

    pub fn chart_of_accounts_integrations(&self) -> &ChartOfAccountsIntegrations<Perms> {
        &self.chart_of_accounts_integrations
    }

    pub fn terms_templates(&self) -> &TermsTemplates<Perms> {
        &self.terms_templates
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
            &self.facilities,
            &self.disbursals,
            &self.payments,
            &self.history_repo,
            &self.repayment_plan_repo,
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
        let collateral_id = CollateralId::new();
        let account_ids = CreditFacilityAccountIds::new();
        let new_credit_facility = NewCreditFacility::builder()
            .id(id)
            .ledger_tx_id(LedgerTxId::new())
            .approval_process_id(id)
            .collateral_id(collateral_id)
            .customer_id(customer_id)
            .terms(terms)
            .amount(amount)
            .account_ids(account_ids)
            .disbursal_credit_account_id(disbursal_credit_account_id.into())
            .audit_info(audit_info.clone())
            .build()
            .expect("could not build new credit facility");

        let mut db = self.facilities.begin_op().await?;

        self.collaterals
            .create_in_op(
                &mut db,
                collateral_id,
                id,
                account_ids.collateral_account_id,
            )
            .await?;

        let credit_facility = self
            .facilities
            .create_in_op(&mut db, new_credit_facility)
            .await?;

        self.ledger
            .handle_facility_create(
                db,
                &credit_facility,
                customer.customer_type,
                terms.duration.duration_type(),
            )
            .await?;

        Ok(credit_facility)
    }

    #[instrument(name = "credit_facility.history", skip(self), err)]
    pub async fn history<T: From<CreditFacilityHistoryEntry>>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CoreCreditError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;
        let history = self.history_repo.load(id).await?;
        Ok(history.entries.into_iter().rev().map(T::from).collect())
    }

    #[instrument(name = "credit_facility.repayment_plan", skip(self), err)]
    pub async fn repayment_plan<T: From<CreditFacilityRepaymentPlanEntry>>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CoreCreditError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;
        let repayment_plan = self.repayment_plan_repo.load(id).await?;
        Ok(repayment_plan.entries.into_iter().map(T::from).collect())
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
                CoreCreditObject::all_disbursals(),
                CoreCreditAction::DISBURSAL_INITIATE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "credit_facility.initiate_disbursal", skip(self), err)]
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
            .facilities
            .find_by_id_without_audit(credit_facility_id)
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

        if !facility.is_activated() {
            return Err(CreditFacilityError::NotActivatedYet.into());
        }
        let now = crate::time::now();
        if !facility.check_disbursal_date(now) {
            return Err(CreditFacilityError::DisbursalPastMaturityDate.into());
        }
        let balance = self
            .ledger
            .get_credit_facility_balance(facility.account_ids)
            .await?;

        let price = self.price.usd_cents_per_btc().await?;
        if !facility.terms.is_disbursal_allowed(balance, amount, price) {
            return Err(CreditFacilityError::BelowMarginLimit.into());
        }

        let mut db = self.facilities.begin_op().await?;
        let disbursal_id = DisbursalId::new();
        let due_date = facility.matures_at.expect("Facility is not active");
        let overdue_date = facility
            .terms
            .obligation_overdue_duration_from_due
            .map(|d| d.end_date(due_date));
        let liquidation_date = facility
            .terms
            .obligation_liquidation_duration_from_due
            .map(|d| d.end_date(due_date));

        let new_disbursal = NewDisbursal::builder()
            .id(disbursal_id)
            .approval_process_id(disbursal_id)
            .credit_facility_id(credit_facility_id)
            .amount(amount)
            .account_ids(facility.account_ids)
            .disbursal_credit_account_id(facility.disbursal_credit_account_id)
            .due_date(due_date)
            .overdue_date(overdue_date)
            .liquidation_date(liquidation_date)
            .audit_info(audit_info)
            .build()?;

        let disbursal = self.disbursals.create_in_op(&mut db, new_disbursal).await?;

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

    #[instrument(name = "credit_facility.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
        updated_collateral: Satoshis,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<CreditFacility, CoreCreditError> {
        let credit_facility_id = credit_facility_id.into();
        let effective = effective.into();

        let audit_info = self
            .subject_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let mut db = self.facilities.begin_op().await?;

        let collateral_update = if let Some(collateral_update) = self
            .collaterals
            .record_collateral_update_in_op(
                &mut db,
                credit_facility.collateral_id,
                updated_collateral,
                effective,
                &audit_info,
            )
            .await?
        {
            collateral_update
        } else {
            return Ok(credit_facility);
        };

        self.ledger
            .update_credit_facility_collateral(db, collateral_update, credit_facility.account_ids)
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

    #[instrument(name = "credit_facility.record_payment", skip(self), err)]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn record_payment(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
        amount: UsdCents,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<CreditFacility, CoreCreditError> {
        let credit_facility_id = credit_facility_id.into();

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let mut db = self.facilities.begin_op().await?;

        let allocations = self
            .payments
            .record_in_op(sub, &mut db, credit_facility_id, amount, effective)
            .await?;

        self.ledger
            .record_obligation_repayments(db, allocations)
            .await?;

        Ok(credit_facility)
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
    #[es_entity::retry_on_concurrent_modification(any_error = true, max_retries = 15)]
    pub async fn complete_facility(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
    ) -> Result<CreditFacility, CoreCreditError> {
        let id = credit_facility_id.into();

        let audit_info = self
            .subject_can_complete(sub, true)
            .await?
            .expect("audit info missing");

        let mut db = self.facilities.begin_op().await?;

        let credit_facility = match self
            .facilities
            .complete_in_op(&mut db, id, self.config.upgrade_buffer_cvl_pct, &audit_info)
            .await?
        {
            CompletionOutcome::Ignored(facility) => facility,

            CompletionOutcome::Completed((facility, completion)) => {
                self.collaterals
                    .record_collateral_update_in_op(
                        &mut db,
                        facility.collateral_id,
                        Satoshis::ZERO,
                        crate::time::now().date_naive(),
                        &audit_info,
                    )
                    .await?;

                self.ledger.complete_credit_facility(db, completion).await?;
                facility
            }
        };

        Ok(credit_facility)
    }

    pub async fn can_be_completed(&self, entity: &CreditFacility) -> Result<bool, CoreCreditError> {
        Ok(self.outstanding(entity).await?.is_zero())
    }

    pub async fn current_cvl(&self, entity: &CreditFacility) -> Result<CVLPct, CoreCreditError> {
        let balances = self
            .ledger
            .get_credit_facility_balance(entity.account_ids)
            .await?;
        let price = self.price.usd_cents_per_btc().await?;
        Ok(balances.current_cvl(price))
    }

    pub async fn outstanding(&self, entity: &CreditFacility) -> Result<UsdCents, CoreCreditError> {
        let balances = self
            .ledger
            .get_credit_facility_balance(entity.account_ids)
            .await?;
        Ok(balances.total_outstanding_payable())
    }
}
