mod config;
mod error;

use core_credit::PaymentSourceAccountId;
use sqlx::PgPool;
use tracing::{Instrument, instrument};
use tracing_macros::record_error_severity;

use authz::PermissionCheck;

use rbac_types::{AuditAction, AuditEntityAction, AuditObject};

use crate::{
    access::Access,
    accounting::Accounting,
    accounting_init::{ChartsInit, JournalInit, StatementsInit},
    audit::{Audit, AuditCursor, AuditEntry},
    authorization::{Authorization, seed},
    contract_creation::ContractCreation,
    credit::Credit,
    custody::Custody,
    customer::Customers,
    customer_sync::CustomerSync,
    dashboard::Dashboard,
    deposit::Deposits,
    deposit_sync::DepositSync,
    document::DocumentStorage,
    governance::Governance,
    job::Jobs,
    kyc::CustomerKyc,
    notification::Notification,
    outbox::Outbox,
    price::Price,
    primitives::Subject,
    public_id::PublicIds,
    report::Reports,
    storage::Storage,
    terms_template::TermsTemplates,
    time_events::TimeEvents,
    user_onboarding::UserOnboarding,
};
use domain_config::{ExposedDomainConfigs, ExposedDomainConfigsReadOnly, InternalDomainConfigs};

pub use config::*;
use error::ApplicationError;

#[derive(Clone)]
pub struct LanaApp {
    _pool: PgPool,
    exposed_domain_configs: ExposedDomainConfigs<Authorization>,
    jobs: Jobs,
    audit: Audit,
    authz: Authorization,
    accounting: Accounting,
    customers: Customers,
    deposits: Deposits,
    customer_kyc: CustomerKyc,
    access: Access,
    credit: Credit,
    custody: Custody,
    price: Price,
    outbox: Outbox,
    governance: Governance,
    dashboard: Dashboard,
    public_ids: PublicIds,
    contract_creation: ContractCreation,
    reports: Reports,
    terms_templates: TermsTemplates,
    _time_events: TimeEvents,
    _user_onboarding: UserOnboarding,
    _customer_sync: CustomerSync,
    _deposit_sync: DepositSync,
}

impl LanaApp {
    #[record_error_severity]
    #[instrument(name = "lana_app.init", skip_all)]
    pub async fn init(
        pool: PgPool,
        config: AppConfig,
        clock: es_entity::clock::ClockHandle,
    ) -> Result<Self, ApplicationError> {
        sqlx::migrate!()
            .run(&pool)
            .instrument(tracing::info_span!("lana_app.migrations"))
            .await?;

        let audit = Audit::new(&pool);
        let outbox = Outbox::init(
            &pool,
            obix::MailboxConfig::builder()
                .build()
                .expect("should build with defaults"),
        )
        .await?;
        let authz = Authorization::init(&pool, &audit).await?;
        let internal_domain_configs = InternalDomainConfigs::new(&pool);
        let exposed_domain_configs = ExposedDomainConfigs::new(&pool, &authz);
        let exposed_domain_configs_readonly = ExposedDomainConfigsReadOnly::new(&pool);
        internal_domain_configs.seed_registered().await?;
        exposed_domain_configs.seed_registered().await?;

        let access = Access::init(
            &pool,
            config.access,
            rbac_types::LanaAction::action_descriptions(),
            seed::PREDEFINED_ROLES,
            &authz,
            &outbox,
            clock.clone(),
        )
        .await?;

        let mut jobs = Jobs::init(
            job::JobSvcConfig::builder()
                .pool(pool.clone())
                .clock(clock.clone())
                .poller_config(config.job_poller)
                .build()
                .expect("Couldn't build JobSvcConfig"),
        )
        .await?;

        let dashboard = Dashboard::init(&pool, &authz, &mut jobs, &outbox).await?;
        let governance = Governance::new(&pool, &authz, &outbox, clock.clone());
        let storage = Storage::new(&config.storage);
        let reports =
            Reports::init(&pool, &authz, config.report, &outbox, &storage, &mut jobs).await?;
        let price = Price::init(&mut jobs, &outbox).await?;
        let _time_events =
            TimeEvents::init(&exposed_domain_configs_readonly, &mut jobs, &outbox).await?;
        let documents = DocumentStorage::new(&pool, &storage, clock.clone());
        let public_ids = PublicIds::new(&pool);

        let user_onboarding =
            UserOnboarding::init(&mut jobs, &outbox, config.user_onboarding).await?;

        let cala_config = cala_ledger::CalaLedgerConfig::builder()
            .pool(pool.clone())
            .exec_migrations(false)
            .build()
            .expect("cala config");
        let cala = cala_ledger::CalaLedger::init(cala_config).await?;
        let journal_init = JournalInit::journal(&cala).await?;
        let accounting = Accounting::new(
            &pool,
            &authz,
            &cala,
            journal_init.journal_id,
            documents.clone(),
            &mut jobs,
            &outbox,
        );

        StatementsInit::statements(&accounting).await?;

        let customers = Customers::new(
            &pool,
            &authz,
            &outbox,
            documents.clone(),
            public_ids.clone(),
            clock.clone(),
        );
        let deposits = Deposits::init(
            &pool,
            &authz,
            &outbox,
            &governance,
            &mut jobs,
            &cala,
            journal_init.journal_id,
            &public_ids,
            &customers,
            &exposed_domain_configs_readonly,
            &internal_domain_configs,
        )
        .await?;
        let customer_sync = CustomerSync::init(
            &mut jobs,
            &outbox,
            &customers,
            &deposits,
            config.customer_sync,
        )
        .await?;

        let customer_kyc = CustomerKyc::init(
            &pool,
            &exposed_domain_configs_readonly,
            &authz,
            &customers,
            &mut jobs,
        )
        .await?;

        let deposit_sync = DepositSync::init(
            &mut jobs,
            &outbox,
            &deposits,
            &customers,
            customer_kyc.sumsub_client().clone(),
        )
        .await?;

        let custody = Custody::init(
            &pool,
            &authz,
            config.custody,
            &outbox,
            &mut jobs,
            clock.clone(),
        )
        .await?;

        let credit = Credit::init(
            &pool,
            config.credit,
            &governance,
            &mut jobs,
            &authz,
            &customers,
            &custody,
            &price,
            &outbox,
            &cala,
            journal_init.journal_id,
            &public_ids,
            &exposed_domain_configs_readonly,
            &internal_domain_configs,
        )
        .await?;

        let terms_templates =
            TermsTemplates::new(&pool, std::sync::Arc::new(authz.clone()), clock.clone());

        let contract_creation = ContractCreation::new(
            config.gotenberg,
            &customers,
            &customer_kyc,
            &documents,
            &mut jobs,
            &authz,
        );

        Notification::init(
            config.notification,
            &mut jobs,
            &outbox,
            access.users(),
            &credit,
            &customers,
            &exposed_domain_configs_readonly,
        )
        .await?;

        ChartsInit::charts_of_accounts(&accounting, &credit, &deposits, config.accounting_init)
            .await?;

        jobs.start_poll().await?;

        Ok(Self {
            _pool: pool,
            exposed_domain_configs,
            jobs,
            audit,
            authz,
            accounting,
            customers,
            deposits,
            customer_kyc,
            access,
            price,
            credit,
            custody,
            outbox,
            governance,
            dashboard,
            public_ids,
            contract_creation,
            reports,
            terms_templates,
            _time_events,
            _user_onboarding: user_onboarding,
            _customer_sync: customer_sync,
            _deposit_sync: deposit_sync,
        })
    }

    pub fn dashboard(&self) -> &Dashboard {
        &self.dashboard
    }

    pub fn exposed_domain_configs(&self) -> &ExposedDomainConfigs<Authorization> {
        &self.exposed_domain_configs
    }

    pub fn governance(&self) -> &Governance {
        &self.governance
    }

    pub fn reports(&self) -> &Reports {
        &self.reports
    }

    pub fn customers(&self) -> &Customers {
        &self.customers
    }

    pub fn audit(&self) -> &Audit {
        &self.audit
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    pub fn outbox(&self) -> &Outbox {
        &self.outbox
    }

    #[record_error_severity]
    #[instrument(name = "lana.audit.list_audit", skip(self))]
    pub async fn list_audit(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<AuditCursor>,
        subject_filter: Option<String>,
        authorized_filter: Option<bool>,
        object_filter: Option<String>,
        action_filter: Option<String>,
    ) -> Result<es_entity::PaginatedQueryRet<AuditEntry, AuditCursor>, ApplicationError> {
        use crate::audit::AuditSvc;

        self.authz
            .enforce_permission(
                sub,
                AuditObject::all_audits(),
                AuditAction::from(AuditEntityAction::List),
            )
            .await?;

        self.audit
            .list(query, subject_filter, authorized_filter, object_filter, action_filter)
            .await
            .map_err(ApplicationError::from)
    }

    #[record_error_severity]
    #[instrument(name = "lana.audit.list_audit_subjects", skip(self))]
    pub async fn list_audit_subjects(
        &self,
        sub: &Subject,
    ) -> Result<Vec<String>, ApplicationError> {
        use crate::audit::AuditSvc;

        self.authz
            .enforce_permission(
                sub,
                AuditObject::all_audits(),
                AuditAction::from(AuditEntityAction::List),
            )
            .await?;

        self.audit
            .list_subjects()
            .await
            .map_err(ApplicationError::from)
    }

    pub fn accounting(&self) -> &Accounting {
        &self.accounting
    }

    pub fn deposits(&self) -> &Deposits {
        &self.deposits
    }

    pub fn customer_kyc(&self) -> &CustomerKyc {
        &self.customer_kyc
    }

    pub fn custody(&self) -> &Custody {
        &self.custody
    }

    pub fn credit(&self) -> &Credit {
        &self.credit
    }

    pub fn access(&self) -> &Access {
        &self.access
    }

    pub fn public_ids(&self) -> &PublicIds {
        &self.public_ids
    }

    pub fn contract_creation(&self) -> &ContractCreation {
        &self.contract_creation
    }

    pub fn terms_templates(&self) -> &TermsTemplates {
        &self.terms_templates
    }

    pub async fn get_visible_nav_items(
        &self,
        sub: &Subject,
    ) -> Result<
        crate::authorization::VisibleNavigationItems,
        crate::authorization::error::AuthorizationError,
    > {
        crate::authorization::get_visible_navigation_items(&self.authz, sub).await
    }

    pub async fn shutdown(&self) -> Result<(), ApplicationError> {
        tracing::info!("app.shutdown");

        self.jobs.shutdown().await?;

        // Shutdown tracer to flush all pending spans
        tracing_utils::shutdown_tracer()?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "lana.app.create_proposal", skip(self),fields(credit_facility_proposal_id = tracing::field::Empty))]
    pub async fn create_facility_proposal(
        &self,
        sub: &Subject,
        customer_id: impl Into<crate::primitives::CustomerId> + std::fmt::Debug + Copy,
        amount: core_money::UsdCents,
        terms: core_credit::TermValues,
        custodian_id: Option<impl Into<crate::primitives::CustodianId> + std::fmt::Debug + Copy>,
    ) -> Result<crate::credit::CreditFacilityProposal, ApplicationError> {
        let customer_id = customer_id.into();
        let deposit_account = self
            .deposits()
            .find_account_by_account_holder_without_audit(customer_id)
            .await?;

        if deposit_account.is_closed() || deposit_account.is_frozen() {
            return Err(ApplicationError::CanNotCreateProposalForClosedOrFrozenAccount);
        }

        let ret = self
            .credit()
            .create_facility_proposal(
                sub,
                customer_id,
                deposit_account.id,
                amount,
                terms,
                custodian_id,
            )
            .await?;

        Ok(ret)
    }

    #[record_error_severity]
    #[instrument(name = "lana.app.record_payment", skip(self),fields(credit_facility_proposal_id = tracing::field::Empty))]
    pub async fn record_payment(
        &self,
        sub: &Subject,
        credit_facility_id: impl Into<crate::primitives::CreditFacilityId> + std::fmt::Debug + Copy,
        amount: core_money::UsdCents,
    ) -> Result<crate::credit::CreditFacility, ApplicationError> {
        let facility = self
            .credit()
            .find_credit_facility(credit_facility_id)
            .await?;

        let deposit_account = self
            .deposits()
            .find_account_by_account_holder_without_audit(facility.customer_id)
            .await?;
        if deposit_account.is_closed() || deposit_account.is_frozen() {
            return Err(ApplicationError::CanNotCreateProposalForClosedOrFrozenAccount);
        }

        let payment_source_account_id = PaymentSourceAccountId::new(deposit_account.id.into());
        let ret = self
            .credit()
            .record_payment(sub, credit_facility_id, payment_source_account_id, amount)
            .await?;

        Ok(ret)
    }

    #[record_error_severity]
    #[instrument(name = "lana.app.record_payment_with_date", skip(self),fields(credit_facility_proposal_id = tracing::field::Empty))]
    pub async fn record_payment_with_date(
        &self,
        sub: &Subject,
        credit_facility_id: impl Into<crate::primitives::CreditFacilityId> + std::fmt::Debug + Copy,
        amount: core_money::UsdCents,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<crate::credit::CreditFacility, ApplicationError> {
        let facility = self
            .credit()
            .find_credit_facility(credit_facility_id)
            .await?;

        let deposit_account = self
            .deposits()
            .find_account_by_account_holder_without_audit(facility.customer_id)
            .await?;
        if deposit_account.is_closed() || deposit_account.is_frozen() {
            return Err(ApplicationError::CanNotCreateProposalForClosedOrFrozenAccount);
        }

        let payment_source_account_id = PaymentSourceAccountId::new(deposit_account.id.into());
        let ret = self
            .credit()
            .record_payment_with_date(
                sub,
                credit_facility_id,
                payment_source_account_id,
                amount,
                effective,
            )
            .await?;

        Ok(ret)
    }
}
