#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
#![cfg_attr(
    feature = "fail-on-warnings",
    allow(clippy::inconsistent_digit_grouping)
)]

mod chart_of_accounts_integration;
mod collateral;
mod config;
mod credit_facility;
mod credit_facility_proposal;
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
mod obligation_installment;
mod payment;
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
use core_custody::{
    CoreCustody, CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject, CustodianId,
};
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};
use public_id::PublicIds;
use tracing::instrument;

pub use chart_of_accounts_integration::{
    ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrationConfigBuilderError,
    ChartOfAccountsIntegrations, error::ChartOfAccountsIntegrationError,
};
pub use collateral::*;
pub use config::*;
pub use credit_facility::error::CreditFacilityError;
pub use credit_facility::*;
pub use credit_facility_proposal::*;
pub use disbursal::{disbursal_cursor::*, *};
use error::*;
pub use event::*;
use for_subject::CreditFacilitiesForSubject;
pub use history::*;
pub use interest_accrual_cycle::*;
use jobs::*;
pub use ledger::*;
pub use obligation::{error::*, obligation_cursor::*, *};
pub use obligation_installment::*;
pub use payment::*;
pub use primitives::*;
use processes::activate_credit_facility::*;
pub use processes::{approve_credit_facility_proposal::*, approve_disbursal::*};
use publisher::CreditFacilityPublisher;
pub use repayment_plan::*;
pub use terms::*;
pub use terms_template::{error as terms_template_error, *};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::{
        TermsTemplateEvent, collateral::CollateralEvent, credit_facility::CreditFacilityEvent,
        credit_facility_proposal::CreditFacilityProposalEvent, disbursal::DisbursalEvent,
        interest_accrual_cycle::InterestAccrualCycleEvent,
        liquidation_process::LiquidationProcessEvent, obligation::ObligationEvent,
        obligation_installment::ObligationInstallmentEvent, payment::PaymentEvent,
    };
}

pub struct CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Perms,
    facilities: CreditFacilities<Perms, E>,
    credit_facility_proposals: CreditFacilityProposals<Perms, E>,
    disbursals: Disbursals<Perms, E>,
    payments: Payments<Perms>,
    history_repo: HistoryRepo,
    repayment_plan_repo: RepaymentPlanRepo,
    governance: Governance<Perms, E>,
    customer: Customers<Perms, E>,
    ledger: CreditLedger,
    price: Price,
    config: CreditConfig,
    approve_disbursal: ApproveDisbursal<Perms, E>,
    cala: CalaLedger,
    activate_credit_facility: ActivateCreditFacility<Perms, E>,
    obligations: Obligations<Perms, E>,
    collaterals: Collaterals<Perms, E>,
    custody: CoreCustody<Perms, E>,
    chart_of_accounts_integrations: ChartOfAccountsIntegrations<Perms>,
    terms_templates: TermsTemplates<Perms>,
    public_ids: PublicIds,
}

impl<Perms, E> Clone for CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            facilities: self.facilities.clone(),
            credit_facility_proposals: self.credit_facility_proposals.clone(),
            obligations: self.obligations.clone(),
            collaterals: self.collaterals.clone(),
            custody: self.custody.clone(),
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
            activate_credit_facility: self.activate_credit_facility.clone(),
            chart_of_accounts_integrations: self.chart_of_accounts_integrations.clone(),
            terms_templates: self.terms_templates.clone(),
            public_ids: self.public_ids.clone(),
        }
    }
}

impl<Perms, E> CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        config: CreditConfig,
        governance: &Governance<Perms, E>,
        jobs: &Jobs,
        authz: &Perms,
        customer: &Customers<Perms, E>,
        custody: &CoreCustody<Perms, E>,
        price: &Price,
        outbox: &Outbox<E>,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
        public_ids: &PublicIds,
    ) -> Result<Self, CoreCreditError> {
        let publisher = CreditFacilityPublisher::new(outbox);
        let ledger = CreditLedger::init(cala, journal_id).await?;
        let obligations = Obligations::new(pool, authz, &ledger, jobs, &publisher);
        let credit_facility_proposals = CreditFacilityProposals::init(
            pool, authz, jobs, &ledger, price, &publisher, governance,
        )
        .await?;
        let credit_facilities = CreditFacilities::init(
            pool,
            authz,
            &obligations,
            &credit_facility_proposals,
            &ledger,
            price,
            jobs,
            &publisher,
            governance,
            public_ids,
        )
        .await?;
        let collaterals = Collaterals::new(pool, authz, &publisher, &ledger);
        let disbursals =
            Disbursals::init(pool, authz, &publisher, &obligations, governance).await?;
        let payments = Payments::new(pool, authz);
        let history_repo = HistoryRepo::new(pool);
        let repayment_plan_repo = RepaymentPlanRepo::new(pool);
        let approve_disbursal =
            ApproveDisbursal::new(&disbursals, &credit_facilities, jobs, governance, &ledger);

        let approve_credit_facility_proposal = ApproveCreditFacilityProposal::new(
            &credit_facility_proposals,
            authz.audit(),
            governance,
        );
        let activate_credit_facility = ActivateCreditFacility::new(
            &credit_facilities,
            &disbursals,
            &ledger,
            price,
            jobs,
            authz.audit(),
            public_ids,
        );
        let chart_of_accounts_integrations = ChartOfAccountsIntegrations::new(authz, &ledger);
        let terms_templates = TermsTemplates::new(pool, authz);

        jobs.add_initializer_and_spawn_unique(
            collateralization_from_price_for_proposal::CreditFacilityProposalCollateralizationFromPriceInit::<
                Perms,
                E,
            >::new(credit_facility_proposals.clone()),
            collateralization_from_price_for_proposal::CreditFacilityProposalCollateralizationFromPriceJobConfig {
                job_interval: std::time::Duration::from_secs(30),
                _phantom: std::marker::PhantomData,
            },
        ).await?;

        jobs
            .add_initializer_and_spawn_unique(
                collateralization_from_price::CreditFacilityCollateralizationFromPriceInit::<
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
        jobs
            .add_initializer_and_spawn_unique(
                collateralization_from_events_for_proposal::CreditFacilityProposalCollateralizationFromEventsInit::<
                    Perms,
                    E,
                >::new(outbox, &credit_facility_proposals),
                collateralization_from_events_for_proposal::CreditFacilityProposalCollateralizationFromEventsJobConfig {
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;
        jobs
            .add_initializer_and_spawn_unique(
                collateralization_from_events::CreditFacilityCollateralizationFromEventsInit::<
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
            credit_facility_history::HistoryProjectionInit::<E>::new(outbox, &history_repo),
            credit_facility_history::HistoryProjectionConfig {
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            credit_facility_repayment_plan::RepaymentPlanProjectionInit::<E>::new(
                outbox,
                &repayment_plan_repo,
            ),
            credit_facility_repayment_plan::RepaymentPlanProjectionConfig {
                _phantom: std::marker::PhantomData,
            },
        )
        .await?;
        jobs.add_initializer(interest_accruals::InterestAccrualInit::<Perms, E>::new(
            &ledger,
            &credit_facilities,
            jobs,
        ));
        jobs.add_initializer(
            interest_accrual_cycles::InterestAccrualCycleInit::<Perms, E>::new(
                &ledger,
                &obligations,
                &credit_facilities,
                jobs,
                authz.audit(),
            ),
        );
        jobs.add_initializer(obligation_due::ObligationDueInit::<Perms, E>::new(
            &ledger,
            &obligations,
            jobs,
        ));
        jobs.add_initializer(obligation_overdue::ObligationOverdueInit::<Perms, E>::new(
            &ledger,
            &obligations,
            jobs,
        ));
        jobs.add_initializer(
            obligation_liquidation::ObligationLiquidationInit::<Perms, E>::new(
                &ledger,
                &obligations,
                jobs,
            ),
        );
        jobs.add_initializer(
            obligation_defaulted::ObligationDefaultedInit::<Perms, E>::new(&ledger, &obligations),
        );
        jobs.add_initializer(credit_facility_maturity::CreditFacilityMaturityInit::<
            Perms,
            E,
        >::new(&credit_facilities));
        jobs.add_initializer_and_spawn_unique(
            DisbursalApprovalInit::new(outbox, &approve_disbursal),
            DisbursalApprovalJobConfig::<Perms, E>::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CreditFacilityActivationInit::new(outbox, &activate_credit_facility),
            CreditFacilityActivationJobConfig::<Perms, E>::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CreditFacilityProposalApprovalInit::new(outbox, &approve_credit_facility_proposal),
            CreditFacilityProposalApprovalJobConfig::<Perms, E>::new(),
        )
        .await?;

        jobs.add_initializer_and_spawn_unique(
            wallet_collateral_sync::WalletCollateralSyncInit::new(
                outbox,
                &credit_facilities,
                &collaterals,
            ),
            wallet_collateral_sync::WalletCollateralSyncJobConfig::<Perms, E>::new(),
        )
        .await?;

        Ok(Self {
            authz: authz.clone(),
            customer: customer.clone(),
            facilities: credit_facilities,
            credit_facility_proposals,
            obligations,
            collaterals,
            custody: custody.clone(),
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
            activate_credit_facility,
            chart_of_accounts_integrations,
            terms_templates,
            public_ids: public_ids.clone(),
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

    pub fn credit_facility_proposals(&self) -> &CreditFacilityProposals<Perms, E> {
        &self.credit_facility_proposals
    }

    pub fn payments(&self) -> &Payments<Perms> {
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
            &self.obligations,
            &self.disbursals,
            &self.history_repo,
            &self.repayment_plan_repo,
            &self.ledger,
        ))
    }

    pub async fn create_facility_proposal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug + Copy,
        disbursal_credit_account_id: impl Into<CalaAccountId> + std::fmt::Debug,
        amount: UsdCents,
        terms: TermValues,
        custodian_id: Option<impl Into<CustodianId> + std::fmt::Debug + Copy>,
    ) -> Result<CreditFacilityProposal, CoreCreditError> {
        self.subject_can_create(sub, true)
            .await?
            .expect("audit info missing");

        let customer = self.customer.find_by_id_without_audit(customer_id).await?;
        if self.config.customer_active_check_enabled && !customer.kyc_verification.is_verified() {
            return Err(CoreCreditError::CustomerNotVerified);
        }

        let proposal_id = CreditFacilityProposalId::new();
        let account_ids = CreditFacilityProposalAccountIds::new();
        let collateral_id = CollateralId::new();

        let mut db = self.credit_facility_proposals.begin_op().await?;

        let wallet_id = if let Some(custodian_id) = custodian_id {
            let custodian_id = custodian_id.into();

            #[cfg(feature = "mock-custodian")]
            if custodian_id.is_mock_custodian() {
                self.custody
                    .ensure_mock_custodian_in_op(&mut db, sub)
                    .await?;
            }

            let wallet = self
                .custody
                .create_wallet_in_op(&mut db, custodian_id, &format!("CF {proposal_id}"))
                .await?;

            Some(wallet.id)
        } else {
            None
        };

        let new_facility_proposal = NewCreditFacilityProposal::builder()
            .id(proposal_id)
            .ledger_tx_id(LedgerTxId::new())
            .approval_process_id(proposal_id)
            .collateral_id(collateral_id)
            .customer_id(customer_id)
            .customer_type(customer.customer_type)
            .terms(terms)
            .amount(amount)
            .account_ids(account_ids)
            .disbursal_credit_account_id(disbursal_credit_account_id.into())
            .build()
            .expect("could not build new credit facility");

        self.collaterals
            .create_in_op(
                &mut db,
                collateral_id,
                proposal_id.into(),
                wallet_id,
                account_ids.collateral_account_id,
            )
            .await?;

        let credit_facility_proposal = self
            .credit_facility_proposals
            .create_in_op(&mut db, new_facility_proposal)
            .await?;

        self.ledger
            .handle_facility_proposal_create(db, &credit_facility_proposal)
            .await?;

        Ok(credit_facility_proposal)
    }

    #[instrument(name = "credit.create_facility", skip(self), err)]
    pub async fn create_facility(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug + Copy,
        disbursal_credit_account_id: impl Into<CalaAccountId> + std::fmt::Debug,
        amount: UsdCents,
        terms: TermValues,
        custodian_id: Option<impl Into<CustodianId> + std::fmt::Debug + Copy>,
    ) -> Result<CreditFacility, CoreCreditError> {
        unimplemented!(" need to be removed eventually");
    }

    #[instrument(name = "credit.history", skip(self), err)]
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

    #[instrument(name = "credit.repayment_plan", skip(self), err)]
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

    #[instrument(name = "credit.initiate_disbursal", skip(self), err)]
    pub async fn initiate_disbursal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<Disbursal, CoreCreditError> {
        self.subject_can_initiate_disbursal(sub, true)
            .await?
            .expect("audit info missing");

        let facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let customer_id = facility.customer_id;
        let customer = self.customer.find_by_id_without_audit(customer_id).await?;
        if self.config.customer_active_check_enabled && !customer.kyc_verification.is_verified() {
            return Err(CoreCreditError::CustomerNotVerified);
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
        let due_date = facility.maturity_date;
        let overdue_date = facility
            .terms
            .obligation_overdue_duration_from_due
            .map(|d| d.end_date(due_date));
        let liquidation_date = facility
            .terms
            .obligation_liquidation_duration_from_due
            .map(|d| d.end_date(due_date));

        let public_id = self
            .public_ids
            .create_in_op(&mut db, DISBURSAL_REF_TARGET, disbursal_id)
            .await?;

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
            .public_id(public_id.id)
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
        _credit_facility: &CreditFacility,
    ) -> Result<Option<CreditFacility>, CoreCreditError> {
        unimplemented!()
        // self.approve_credit_facility
        //     .execute_from_svc(credit_facility)
        //     .await
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

    #[instrument(name = "credit.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
        updated_collateral: Satoshis,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<CreditFacility, CoreCreditError> {
        let credit_facility_id = credit_facility_id.into();
        let effective = effective.into();

        self.subject_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let mut db = self.facilities.begin_op().await?;

        let collateral_update = if let Some(collateral_update) = self
            .collaterals
            .record_collateral_update_via_manual_input_in_op(
                &mut db,
                credit_facility.collateral_id,
                updated_collateral,
                effective,
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

    #[instrument(name = "credit.record_payment", skip(self), err)]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn record_payment(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
        amount: UsdCents,
    ) -> Result<CreditFacility, CoreCreditError> {
        self.subject_can_record_payment(sub, true)
            .await?
            .expect("audit info missing");

        let credit_facility_id = credit_facility_id.into();

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let mut db = self.facilities.begin_op().await?;

        let payment = self
            .payments
            .record_in_op(&mut db, credit_facility_id, amount)
            .await?;

        self.obligations
            .apply_installment_in_op(
                db,
                credit_facility_id,
                payment.id,
                amount,
                crate::time::now().date_naive(),
            )
            .await?;

        Ok(credit_facility)
    }

    pub async fn subject_can_record_payment_with_date(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CoreCreditError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_obligations(),
                CoreCreditAction::OBLIGATION_RECORD_PAYMENT_WITH_DATE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "credit.record_payment_with_date", skip(self), err)]
    #[es_entity::retry_on_concurrent_modification(any_error = true, max_retries = 15)]
    pub async fn record_payment_with_date(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
        amount: UsdCents,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<CreditFacility, CoreCreditError> {
        self.subject_can_record_payment_with_date(sub, true)
            .await?
            .expect("audit info missing");

        let credit_facility_id = credit_facility_id.into();

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let mut db = self.facilities.begin_op().await?;

        let payment = self
            .payments
            .record_in_op(&mut db, credit_facility_id, amount)
            .await?;

        self.obligations
            .apply_installment_in_op(db, credit_facility_id, payment.id, amount, effective.into())
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

    #[instrument(name = "credit.complete_facility", skip(self), err)]
    #[es_entity::retry_on_concurrent_modification(any_error = true, max_retries = 15)]
    pub async fn complete_facility(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug + Copy,
    ) -> Result<CreditFacility, CoreCreditError> {
        let id = credit_facility_id.into();

        self.subject_can_complete(sub, true)
            .await?
            .expect("audit info missing");

        let mut db = self.facilities.begin_op().await?;

        let credit_facility = match self
            .facilities
            .complete_in_op(&mut db, id, self.config.upgrade_buffer_cvl_pct)
            .await?
        {
            CompletionOutcome::Ignored(facility) => facility,

            CompletionOutcome::Completed((facility, completion)) => {
                self.collaterals
                    .record_collateral_update_via_manual_input_in_op(
                        &mut db,
                        facility.collateral_id,
                        Satoshis::ZERO,
                        crate::time::now().date_naive(),
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
