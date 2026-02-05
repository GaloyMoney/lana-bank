#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

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
pub mod ledger;
mod liquidation;
mod pending_credit_facility;
mod primitives;
mod processes;
mod publisher;
mod repayment_plan;

use std::sync::Arc;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use core_accounting::LedgerTransactionInitiator;
use core_custody::{
    CoreCustody, CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject, CustodianId,
};
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_price::{CorePriceEvent, Price};
use domain_config::{
    ExposedDomainConfigsReadOnly, InternalDomainConfigs, RequireVerifiedCustomerForAccount,
};
use es_entity::clock::ClockHandle;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use obix::out::{Outbox, OutboxEventMarker};
use public_id::PublicIds;
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use chart_of_accounts_integration::{
    ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrations,
    error::ChartOfAccountsIntegrationError,
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
pub use ledger::*;
pub use liquidation::{liquidation_cursor::*, *};
pub use pending_credit_facility::*;
pub use primitives::*;
pub use processes::{
    activate_credit_facility::*, allocate_credit_facility_payment::*,
    approve_credit_facility_proposal::*, approve_disbursal::*,
};
use publisher::CreditFacilityPublisher;
pub use repayment_plan::*;

use core_credit_collection::{CoreCreditCollection, PaymentLedgerAccountIds};

#[cfg(feature = "json-schema")]
pub use core_credit_collection::{ObligationEvent, PaymentAllocationEvent, PaymentEvent};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::{
        ObligationEvent, PaymentAllocationEvent, PaymentEvent, collateral::CollateralEvent,
        credit_facility::CreditFacilityEvent,
        credit_facility_proposal::CreditFacilityProposalEvent, disbursal::DisbursalEvent,
        interest_accrual_cycle::InterestAccrualCycleEvent, liquidation::LiquidationEvent,
        pending_credit_facility::PendingCreditFacilityEvent,
    };
}

pub struct CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Arc<Perms>,
    credit_facility_proposals: Arc<CreditFacilityProposals<Perms, E>>,
    pending_credit_facilities: Arc<PendingCreditFacilities<Perms, E>>,
    facilities: Arc<CreditFacilities<Perms, E>>,
    disbursals: Arc<Disbursals<Perms, E>>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    repayment_plans: Arc<RepaymentPlans<Perms>>,
    governance: Arc<Governance<Perms, E>>,
    customer: Arc<Customers<Perms, E>>,
    ledger: Arc<CreditLedger>,
    collateral_ledger: Arc<collateral::ledger::CollateralLedger>,
    price: Arc<Price>,
    config: Arc<CreditConfig>,
    domain_configs: ExposedDomainConfigsReadOnly,
    approve_disbursal: Arc<ApproveDisbursal<Perms, E>>,
    approve_proposal: Arc<ApproveCreditFacilityProposal<Perms, E>>,
    cala: Arc<CalaLedger>,
    activate_credit_facility: Arc<ActivateCreditFacility<Perms, E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    custody: Arc<CoreCustody<Perms, E>>,
    chart_of_accounts_integrations: Arc<ChartOfAccountsIntegrations<Perms>>,
    public_ids: Arc<PublicIds>,
    liquidations: Arc<Liquidations<Perms, E>>,
    histories: Arc<Histories<Perms>>,
    clock: ClockHandle,
}

impl<Perms, E> Clone for CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            clock: self.clock.clone(),
            authz: self.authz.clone(),
            credit_facility_proposals: self.credit_facility_proposals.clone(),
            pending_credit_facilities: self.pending_credit_facilities.clone(),
            facilities: self.facilities.clone(),
            collections: self.collections.clone(),
            collaterals: self.collaterals.clone(),
            custody: self.custody.clone(),
            disbursals: self.disbursals.clone(),
            histories: self.histories.clone(),
            repayment_plans: self.repayment_plans.clone(),
            governance: self.governance.clone(),
            customer: self.customer.clone(),
            ledger: self.ledger.clone(),
            collateral_ledger: self.collateral_ledger.clone(),
            price: self.price.clone(),
            config: self.config.clone(),
            domain_configs: self.domain_configs.clone(),
            cala: self.cala.clone(),
            approve_disbursal: self.approve_disbursal.clone(),
            approve_proposal: self.approve_proposal.clone(),
            activate_credit_facility: self.activate_credit_facility.clone(),
            chart_of_accounts_integrations: self.chart_of_accounts_integrations.clone(),
            public_ids: self.public_ids.clone(),
            liquidations: self.liquidations.clone(),
        }
    }
}

impl<Perms, E> CoreCredit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CustomerObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    #[record_error_severity]
    #[instrument(name = "credit.init", skip_all, fields(journal_id = %journal_id))]
    pub async fn init(
        pool: &sqlx::PgPool,
        config: CreditConfig,
        governance: &Governance<Perms, E>,
        jobs: &mut Jobs,
        authz: &Perms,
        customer: &Customers<Perms, E>,
        custody: &CoreCustody<Perms, E>,
        price: &Price,
        outbox: &Outbox<E>,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
        public_ids: &PublicIds,
        domain_configs: &ExposedDomainConfigsReadOnly,
        internal_domain_configs: &InternalDomainConfigs,
    ) -> Result<Self, CoreCreditError> {
        let clock = jobs.clock().clone();

        // Create Arc-wrapped versions of parameters once
        let authz_arc = Arc::new(authz.clone());
        let governance_arc = Arc::new(governance.clone());
        // let jobs_arc = Arc::new(jobs.clone());
        let price_arc = Arc::new(price.clone());
        let public_ids_arc = Arc::new(public_ids.clone());
        let customer_arc = Arc::new(customer.clone());
        let custody_arc = Arc::new(custody.clone());
        let cala_arc = Arc::new(cala.clone());
        let config_arc = Arc::new(config);
        let internal_domain_configs_arc = Arc::new(internal_domain_configs.clone());

        let publisher = CreditFacilityPublisher::new(outbox);
        let collections_publisher = core_credit_collection::CollectionPublisher::new(outbox);
        let ledger = CreditLedger::init(cala, journal_id, clock.clone()).await?;
        let ledger_arc = Arc::new(ledger);

        let collateral_ledger = collateral::ledger::CollateralLedger::init(
            cala,
            journal_id,
            clock.clone(),
            ledger_arc.collateral_omnibus_account_ids().clone(),
        )
        .await?;
        let collateral_ledger_arc = Arc::new(collateral_ledger);

        let collections = CoreCreditCollection::init(
            pool,
            authz_arc.clone(),
            cala,
            journal_id,
            ledger_arc.payments_made_omnibus_account_ids().account_id,
            jobs,
            &collections_publisher,
            clock.clone(),
        )
        .await?;
        let collections_arc = Arc::new(collections);

        let liquidations = Liquidations::new(pool, authz_arc.clone(), &publisher, clock.clone());
        let liquidations_arc = Arc::new(liquidations);

        let credit_facility_proposals = CreditFacilityProposals::init(
            pool,
            authz_arc.clone(),
            &publisher,
            governance_arc.clone(),
            clock.clone(),
        )
        .await?;
        let proposals_arc = Arc::new(credit_facility_proposals);

        let collaterals = Collaterals::init(
            pool,
            authz_arc.clone(),
            &publisher,
            collateral_ledger_arc.clone(),
            outbox,
            jobs,
            ledger_arc.liquidation_proceeds_omnibus_account_ids(),
            collections_arc.clone(),
        )
        .await?;
        let collaterals_arc = Arc::new(collaterals);

        let pending_credit_facilities = PendingCreditFacilities::init(
            pool,
            proposals_arc.clone(),
            custody_arc.clone(),
            collaterals_arc.clone(),
            authz_arc.clone(),
            ledger_arc.clone(),
            price_arc.clone(),
            &publisher,
            governance_arc.clone(),
            jobs,
            outbox,
            clock.clone(),
        )
        .await?;
        let pending_credit_facilities_arc = Arc::new(pending_credit_facilities);

        let disbursals = Disbursals::init(
            pool,
            authz_arc.clone(),
            &publisher,
            collections_arc.clone(),
            governance_arc.clone(),
            clock.clone(),
        )
        .await?;
        let disbursals_arc = Arc::new(disbursals);

        let credit_facilities = CreditFacilities::init(
            pool,
            authz_arc.clone(),
            collections_arc.clone(),
            pending_credit_facilities_arc.clone(),
            disbursals_arc.clone(),
            ledger_arc.clone(),
            price_arc.clone(),
            jobs,
            &publisher,
            governance_arc.clone(),
            public_ids_arc.clone(),
            outbox,
            clock.clone(),
        )
        .await?;
        let facilities_arc = Arc::new(credit_facilities);

        let histories_arc = Arc::new(Histories::init(pool, outbox, jobs, authz_arc.clone()).await?);

        let repayment_plans_arc =
            Arc::new(RepaymentPlans::init(pool, outbox, jobs, authz_arc.clone()).await?);

        let audit_arc = Arc::new(authz.audit().clone());

        let approve_disbursal = ApproveDisbursal::new(
            disbursals_arc.clone(),
            facilities_arc.clone(),
            governance_arc.clone(),
            ledger_arc.clone(),
        );
        let approve_disbursal_arc = Arc::new(approve_disbursal);

        let approve_proposal = ApproveCreditFacilityProposal::new(
            proposals_arc.clone(),
            pending_credit_facilities_arc.clone(),
            audit_arc.clone(),
            governance_arc.clone(),
        );
        let approve_proposal_arc = Arc::new(approve_proposal);

        let activate_credit_facility = ActivateCreditFacility::new(
            facilities_arc.clone(),
            disbursals_arc.clone(),
            ledger_arc.clone(),
            price_arc.clone(),
            audit_arc.clone(),
            public_ids_arc.clone(),
        );
        let activate_credit_facility_arc = Arc::new(activate_credit_facility);

        let allocate_credit_facility_payment =
            AllocateCreditFacilityPayment::new(collections_arc.clone());
        let allocate_credit_facility_payment_arc = Arc::new(allocate_credit_facility_payment);

        let allocate_payment_job_spawner =
            jobs.add_initializer(AllocateCreditFacilityPaymentInit::new(
                outbox,
                allocate_credit_facility_payment_arc.as_ref(),
            ));
        allocate_payment_job_spawner
            .spawn_unique(
                job::JobId::new(),
                AllocateCreditFacilityPaymentJobConfig::<Perms, E>::new(),
            )
            .await?;

        let chart_of_accounts_integrations = ChartOfAccountsIntegrations::new(
            authz_arc.clone(),
            ledger_arc.clone(),
            internal_domain_configs_arc.clone(),
        );
        let chart_of_accounts_integrations_arc = Arc::new(chart_of_accounts_integrations);

        let approve_disbursal_job_spawner = jobs.add_initializer(DisbursalApprovalInit::new(
            outbox,
            approve_disbursal_arc.as_ref(),
        ));
        approve_disbursal_job_spawner
            .spawn_unique(
                job::JobId::new(),
                DisbursalApprovalJobConfig::<Perms, E>::new(),
            )
            .await?;

        let credit_facility_activation_job_spawner = jobs.add_initializer(
            CreditFacilityActivationInit::new(outbox, activate_credit_facility_arc.as_ref()),
        );
        credit_facility_activation_job_spawner
            .spawn_unique(
                job::JobId::new(),
                CreditFacilityActivationJobConfig::<Perms, E>::new(),
            )
            .await?;

        let credit_facility_proposal_approval_job_spawner = jobs.add_initializer(
            CreditFacilityProposalApprovalInit::new(outbox, approve_proposal_arc.as_ref()),
        );
        credit_facility_proposal_approval_job_spawner
            .spawn_unique(
                job::JobId::new(),
                CreditFacilityProposalApprovalJobConfig::<Perms, E>::new(),
            )
            .await?;

        Ok(Self {
            clock,
            authz: authz_arc,
            customer: customer_arc,
            credit_facility_proposals: proposals_arc,
            pending_credit_facilities: pending_credit_facilities_arc,
            facilities: facilities_arc,
            collections: collections_arc,
            collaterals: collaterals_arc,
            custody: custody_arc,
            disbursals: disbursals_arc,
            histories: histories_arc,
            repayment_plans: repayment_plans_arc,
            governance: governance_arc,
            ledger: ledger_arc,
            collateral_ledger: collateral_ledger_arc,
            price: price_arc,
            config: config_arc,
            domain_configs: domain_configs.clone(),
            cala: cala_arc,
            approve_disbursal: approve_disbursal_arc,
            approve_proposal: approve_proposal_arc,
            activate_credit_facility: activate_credit_facility_arc,
            chart_of_accounts_integrations: chart_of_accounts_integrations_arc,
            public_ids: public_ids_arc,
            liquidations: liquidations_arc,
        })
    }

    pub fn collections(&self) -> &CoreCreditCollection<Perms, E> {
        self.collections.as_ref()
    }

    pub fn collaterals(&self) -> &Collaterals<Perms, E> {
        self.collaterals.as_ref()
    }

    pub fn disbursals(&self) -> &Disbursals<Perms, E> {
        self.disbursals.as_ref()
    }

    pub fn liquidations(&self) -> &Liquidations<Perms, E> {
        self.liquidations.as_ref()
    }

    pub fn proposals(&self) -> &CreditFacilityProposals<Perms, E> {
        self.credit_facility_proposals.as_ref()
    }

    pub fn pending_credit_facilities(&self) -> &PendingCreditFacilities<Perms, E> {
        self.pending_credit_facilities.as_ref()
    }

    pub fn facilities(&self) -> &CreditFacilities<Perms, E> {
        self.facilities.as_ref()
    }

    pub fn chart_of_accounts_integrations(&self) -> &ChartOfAccountsIntegrations<Perms> {
        self.chart_of_accounts_integrations.as_ref()
    }

    pub fn histories(&self) -> &Histories<Perms> {
        self.histories.as_ref()
    }

    pub fn repayment_plans(&self) -> &RepaymentPlans<Perms> {
        self.repayment_plans.as_ref()
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
            &self.collections,
            &self.disbursals,
            &self.histories,
            &self.repayment_plans,
            &self.ledger,
        ))
    }

    #[record_error_severity]
    #[instrument(name = "credit.find_credit_facility", skip(self))]
    pub async fn find_credit_facility(
        &self,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CoreCreditError> {
        Ok(self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "credit.create_proposal", skip(self),fields(credit_facility_proposal_id = tracing::field::Empty))]
    pub async fn create_facility_proposal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug + Copy,
        deposit_account_id: impl Into<CalaAccountId> + std::fmt::Debug + Copy,
        amount: UsdCents,
        terms: TermValues,
        custodian_id: Option<impl Into<CustodianId> + std::fmt::Debug + Copy>,
    ) -> Result<CreditFacilityProposal, CoreCreditError> {
        self.subject_can_create(sub, true)
            .await?
            .expect("audit info missing");

        let customer = self.customer.find_by_id_without_audit(customer_id).await?;
        let require_verified = self
            .domain_configs
            .get_without_audit::<RequireVerifiedCustomerForAccount>()
            .await?
            .value();
        if require_verified && !customer.kyc_verification.is_verified() {
            return Err(CoreCreditError::CustomerNotVerified);
        }

        let proposal_id = CreditFacilityProposalId::new();
        tracing::Span::current().record(
            "credit_facility_proposal_id",
            tracing::field::display(proposal_id),
        );

        let mut db = self.pending_credit_facilities.begin_op().await?;

        let new_facility_proposal = NewCreditFacilityProposal::builder()
            .id(proposal_id)
            .customer_id(customer.id)
            .customer_type(customer.customer_type)
            .custodian_id(custodian_id.map(|id| id.into()))
            .disbursal_credit_account_id(deposit_account_id)
            .terms(terms)
            .amount(amount)
            .build()
            .expect("could not build new credit facility proposal");

        let credit_facility_proposal = self
            .credit_facility_proposals
            .create_in_op(&mut db, new_facility_proposal)
            .await?;

        db.commit().await?;

        Ok(credit_facility_proposal)
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

    #[record_error_severity]
    #[instrument(name = "credit.initiate_disbursal", skip(self),fields(credit_facility_id = %credit_facility_id))]
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
        let require_verified = self
            .domain_configs
            .get_without_audit::<RequireVerifiedCustomerForAccount>()
            .await?
            .value();
        if require_verified && !customer.kyc_verification.is_verified() {
            return Err(CoreCreditError::CustomerNotVerified);
        }

        let now = self.clock.now();
        if facility.is_single_disbursal() {
            return Err(CreditFacilityError::OnlyOneDisbursalAllowed.into());
        }
        if !facility.check_disbursal_date(now) {
            return Err(CreditFacilityError::DisbursalPastMaturityDate.into());
        }
        let balance = self
            .ledger
            .get_credit_facility_balance(facility.account_ids)
            .await?;

        let price = self.price.usd_cents_per_btc().await;
        let cvl = balance.with_added_disbursal(amount).current_cvl(price);
        if !facility.terms.is_disbursal_allowed(cvl) {
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
            .account_ids(facility.account_ids.into())
            .disbursal_credit_account_id(facility.disbursal_credit_account_id)
            .due_date(due_date)
            .overdue_date(overdue_date)
            .liquidation_date(liquidation_date)
            .public_id(public_id.id)
            .build()?;

        let disbursal = self.disbursals.create_in_op(&mut db, new_disbursal).await?;

        self.ledger
            .initiate_disbursal_in_op(
                &mut db,
                disbursal.id,
                disbursal.initiated_tx_id,
                disbursal.amount,
                facility.account_ids,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;

        db.commit().await?;

        Ok(disbursal)
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

    #[record_error_severity]
    #[instrument(name = "credit.update_pending_facility_collateral", skip(self, pending_credit_facility_id), fields(pending_credit_facility_id = tracing::field::Empty))]
    pub async fn update_pending_facility_collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        pending_credit_facility_id: impl Into<PendingCreditFacilityId> + std::fmt::Debug + Copy,
        updated_collateral: Satoshis,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<PendingCreditFacility, CoreCreditError> {
        let effective = effective.into();

        self.subject_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let pending_facility = self
            .pending_credit_facilities()
            .find_by_id_without_audit(pending_credit_facility_id.into())
            .await?;

        tracing::Span::current().record(
            "pending_credit_facility_id",
            tracing::field::display(pending_facility.id),
        );

        let mut db = self.facilities.begin_op().await?;

        let collateral_update = if let Some(collateral_update) = self
            .collaterals
            .record_collateral_update_via_manual_input_in_op(
                &mut db,
                pending_facility.collateral_id,
                updated_collateral,
                effective,
            )
            .await?
        {
            collateral_update
        } else {
            return Ok(pending_facility);
        };

        self.collateral_ledger
            .update_collateral_amount_in_op(
                &mut db,
                collateral_update,
                pending_facility.account_ids.collateral_account_id,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;

        db.commit().await?;

        Ok(pending_facility)
    }

    #[record_error_severity]
    #[instrument(name = "credit.update_collateral", skip(self))]
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
        self.collateral_ledger
            .update_collateral_amount_in_op(
                &mut db,
                collateral_update,
                credit_facility.account_ids.collateral_account_id,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;

        db.commit().await?;

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

    #[record_error_severity]
    #[instrument(name = "credit.record_payment", skip(self, credit_facility_id), fields(credit_facility_id = tracing::field::Empty))]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn record_payment(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl es_entity::RetryableInto<CreditFacilityId>,
        payment_source_account_id: impl es_entity::RetryableInto<PaymentSourceAccountId>,
        amount: UsdCents,
    ) -> Result<CreditFacility, CoreCreditError> {
        self.subject_can_record_payment(sub, true)
            .await?
            .expect("audit info missing");

        let credit_facility_id = credit_facility_id.into();
        let payment_source_account_id = payment_source_account_id.into();

        tracing::Span::current().record(
            "credit_facility_id",
            tracing::field::display(credit_facility_id),
        );

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let payment_id = PaymentId::new();
        let effective = self.clock.today();
        let initiated_by = LedgerTransactionInitiator::try_from_subject(sub)?;
        self.collections
            .payments()
            .record(
                payment_id,
                credit_facility_id.into(),
                PaymentLedgerAccountIds {
                    facility_payment_holding_account_id: credit_facility
                        .payment_holding_account_id(),
                    facility_uncovered_outstanding_account_id: credit_facility
                        .uncovered_outstanding_account_id(),
                    payment_source_account_id,
                },
                amount,
                effective,
                initiated_by,
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

    #[record_error_severity]
    #[instrument(name = "credit.record_payment_with_date", skip(self))]
    #[es_entity::retry_on_concurrent_modification(any_error = true, max_retries = 15)]
    pub async fn record_payment_with_date(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl es_entity::RetryableInto<CreditFacilityId>,
        payment_source_account_id: impl es_entity::RetryableInto<PaymentSourceAccountId>,
        amount: UsdCents,
        effective: impl es_entity::RetryableInto<chrono::NaiveDate>,
    ) -> Result<CreditFacility, CoreCreditError> {
        self.subject_can_record_payment_with_date(sub, true)
            .await?
            .expect("audit info missing");

        let credit_facility_id = credit_facility_id.into();
        let payment_source_account_id = payment_source_account_id.into();

        let credit_facility = self
            .facilities
            .find_by_id_without_audit(credit_facility_id)
            .await?;

        let payment_id = PaymentId::new();
        let initiated_by = LedgerTransactionInitiator::try_from_subject(sub)?;
        self.collections
            .payments()
            .record(
                payment_id,
                credit_facility_id.into(),
                PaymentLedgerAccountIds {
                    facility_payment_holding_account_id: credit_facility
                        .payment_holding_account_id(),
                    facility_uncovered_outstanding_account_id: credit_facility
                        .uncovered_outstanding_account_id(),
                    payment_source_account_id,
                },
                amount,
                effective.into(),
                initiated_by,
            )
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

    #[record_error_severity]
    #[instrument(name = "credit.complete_facility", skip(self))]
    #[es_entity::retry_on_concurrent_modification(any_error = true, max_retries = 15)]
    pub async fn complete_facility(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl es_entity::RetryableInto<CreditFacilityId>,
    ) -> Result<CreditFacility, CoreCreditError> {
        let id = credit_facility_id.into();

        self.subject_can_complete(sub, true)
            .await?
            .expect("audit info missing");

        let mut db = self.facilities.begin_op().await?;

        let credit_facility = match self
            .facilities
            .complete_in_op(&mut db, id, CVLPct::UPGRADE_BUFFER)
            .await?
        {
            CompletionOutcome::AlreadyApplied(facility) => facility,

            CompletionOutcome::Completed((facility, completion)) => {
                self.collaterals
                    .record_collateral_update_via_manual_input_in_op(
                        &mut db,
                        facility.collateral_id,
                        Satoshis::ZERO,
                        self.clock.today(),
                    )
                    .await?;

                self.ledger
                    .complete_credit_facility_in_op(
                        &mut db,
                        completion,
                        LedgerTransactionInitiator::try_from_subject(sub)?,
                    )
                    .await?;
                db.commit().await?;

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
        let price = self.price.usd_cents_per_btc().await;
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
