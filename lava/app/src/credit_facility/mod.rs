mod activate;
mod config;
mod disbursement;
mod entity;
pub mod error;
mod history;
mod interest_accrual;
mod jobs;
mod repo;

use std::collections::HashMap;

use authz::PermissionCheck;
use governance::ApprovalProcessType;

use crate::{
    audit::{Audit, AuditInfo, AuditSvc},
    authorization::{Authorization, CreditFacilityAction, CustomerAllOrOne, Object},
    customer::Customers,
    data_export::Export,
    entity::EntityError,
    governance::Governance,
    job::{error::JobError, *},
    ledger::{credit_facility::*, Ledger},
    outbox::Outbox,
    price::Price,
    primitives::{
        CreditFacilityId, CustomerId, DisbursementIdx, PriceOfOneBTC, Satoshis, Subject, UsdCents,
    },
    terms::TermValues,
};

pub use config::*;
pub use disbursement::*;
pub use entity::*;
use error::*;
pub use history::*;
pub use interest_accrual::*;
use jobs::*;
pub use repo::cursor::*;
use repo::CreditFacilityRepo;
use tracing::instrument;

pub const APPROVE_CREDIT_FACILITY_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility");
const APPROVE_DISBURSEMENT_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursement");

#[derive(Clone)]
pub struct CreditFacilities {
    pool: sqlx::PgPool,
    authz: Authorization,
    customers: Customers,
    credit_facility_repo: CreditFacilityRepo,
    disbursement_repo: DisbursementRepo,
    interest_accrual_repo: InterestAccrualRepo,
    governance: Governance,
    jobs: Jobs,
    ledger: Ledger,
    price: Price,
    config: CreditFacilityConfig,
}

impl CreditFacilities {
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        pool: &sqlx::PgPool,
        config: CreditFacilityConfig,
        governance: &Governance,
        jobs: &Jobs,
        export: &Export,
        authz: &Authorization,
        audit: &Audit,
        customers: &Customers,
        ledger: &Ledger,
        price: &Price,
        outbox: &Outbox,
    ) -> Result<Self, CreditFacilityError> {
        let credit_facility_repo = CreditFacilityRepo::new(pool, export);
        let disbursement_repo = DisbursementRepo::new(pool, export);
        let interest_accrual_repo = InterestAccrualRepo::new(pool, export);
        jobs.add_initializer_and_spawn_unique(
            cvl::CreditFacilityProcessingJobInitializer::new(
                credit_facility_repo.clone(),
                price,
                audit,
            ),
            cvl::CreditFacilityJobConfig {
                job_interval: std::time::Duration::from_secs(30),
                upgrade_buffer_cvl_pct: config.upgrade_buffer_cvl_pct,
            },
        )
        .await?;
        jobs.add_initializer(interest::CreditFacilityProcessingJobInitializer::new(
            ledger,
            credit_facility_repo.clone(),
            interest_accrual_repo.clone(),
            audit,
        ));
        jobs.add_initializer_and_spawn_unique(
            approve_credit_facility::CreditFacilityApprovalJobInitializer::new(
                pool,
                &credit_facility_repo,
                &interest_accrual_repo,
                price,
                ledger,
                jobs,
                authz.audit(),
                outbox,
            ),
            approve_credit_facility::CreditFacilityApprovalJobConfig,
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            approve_disbursement::DisbursementApprovalJobInitializer::new(
                pool,
                &disbursement_repo,
                authz.audit(),
                outbox,
            ),
            approve_disbursement::DisbursementApprovalJobConfig,
        )
        .await?;
        let _ = governance
            .init_policy(APPROVE_CREDIT_FACILITY_PROCESS)
            .await;
        let _ = governance.init_policy(APPROVE_DISBURSEMENT_PROCESS).await;

        Ok(Self {
            pool: pool.clone(),
            authz: authz.clone(),
            customers: customers.clone(),
            credit_facility_repo,
            disbursement_repo,
            governance: governance.clone(),
            jobs: jobs.clone(),
            interest_accrual_repo,
            ledger: ledger.clone(),
            price: price.clone(),
            config,
        })
    }

    pub async fn user_can_create(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::Create,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.credit_facility.initiate", skip(self), err)]
    pub async fn initiate(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        facility: UsdCents,
        terms: TermValues,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let customer_id = customer_id.into();

        let audit_info = self
            .user_can_create(sub, true)
            .await?
            .expect("audit info missing");

        let customer = match self.customers.find_by_id(Some(sub), customer_id).await? {
            Some(customer) => customer,
            None => return Err(CreditFacilityError::CustomerNotFound(customer_id)),
        };

        let id = CreditFacilityId::new();
        let new_credit_facility = NewCreditFacility::builder()
            .id(id)
            .approval_process_id(id)
            .customer_id(customer_id)
            .terms(terms)
            .facility(facility)
            .account_ids(CreditFacilityAccountIds::new())
            .customer_account_ids(customer.account_ids)
            .audit_info(audit_info)
            .build()
            .expect("could not build new credit facility");

        let mut db_tx = self.pool.begin().await?;
        self.governance
            .start_process(
                &mut db_tx,
                id,
                id.to_string(),
                APPROVE_CREDIT_FACILITY_PROCESS,
            )
            .await?;
        let credit_facility = self
            .credit_facility_repo
            .create_in_tx(&mut db_tx, new_credit_facility)
            .await?;
        self.ledger
            .create_accounts_for_credit_facility(credit_facility.id, credit_facility.account_ids)
            .await?;

        db_tx.commit().await?;

        Ok(credit_facility)
    }

    #[instrument(name = "lava.credit_facility.find", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: Option<&Subject>,
        id: CreditFacilityId,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        if let Some(sub) = sub {
            self.authz
                .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::Read)
                .await?;
        }

        match self.credit_facility_repo.find_by_id(id).await {
            Ok(loan) => Ok(Some(loan)),
            Err(CreditFacilityError::EntityError(EntityError::NoEntityEventsPresent)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn user_can_initiate_disbursement(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::InitiateDisbursement,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.credit_facility.initiate_disbursement", skip(self), err)]
    pub async fn initiate_disbursement(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<Disbursement, CreditFacilityError> {
        let audit_info = self
            .user_can_initiate_disbursement(sub, true)
            .await?
            .expect("audit info missing");

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;
        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;
        balances.check_disbursement_amount(amount)?;

        let mut db_tx = self.pool.begin().await?;
        let new_disbursement =
            credit_facility.initiate_disbursement(amount, chrono::Utc::now(), audit_info)?;
        self.governance
            .start_process(
                &mut db_tx,
                new_disbursement.approval_process_id,
                new_disbursement.approval_process_id.to_string(),
                APPROVE_DISBURSEMENT_PROCESS,
            )
            .await?;
        self.credit_facility_repo
            .update_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        let disbursement = self
            .disbursement_repo
            .create_in_tx(&mut db_tx, new_disbursement)
            .await?;

        db_tx.commit().await?;
        Ok(disbursement)
    }

    #[instrument(name = "lava.credit_facility.confirm_disbursement", skip(self), err)]
    pub async fn confirm_disbursement(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        disbursement_idx: DisbursementIdx,
    ) -> Result<Disbursement, CreditFacilityError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let disbursement_id = credit_facility
            .disbursement_id_from_idx(disbursement_idx)
            .ok_or_else(|| disbursement::error::DisbursementError::NotFound)?;

        let mut disbursement = self.disbursement_repo.find_by_id(disbursement_id).await?;

        let mut db_tx = self.pool.begin().await?;

        if let Ok(disbursement_data) = disbursement.disbursement_data() {
            let audit_info = self
                .authz
                .audit()
                .record_system_entry_in_tx(
                    &mut db_tx,
                    Object::CreditFacility,
                    CreditFacilityAction::ConfirmDisbursement,
                )
                .await?;

            let executed_at = self
                .ledger
                .record_disbursement(disbursement_data.clone())
                .await?;
            disbursement.confirm(&disbursement_data, executed_at, audit_info.clone());

            credit_facility.confirm_disbursement(
                &disbursement,
                disbursement_data.tx_id,
                executed_at,
                audit_info.clone(),
            );
        }

        self.disbursement_repo
            .update_in_tx(&mut db_tx, &mut disbursement)
            .await?;
        self.credit_facility_repo
            .update_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        db_tx.commit().await?;

        Ok(disbursement)
    }

    pub async fn user_can_update_collateral(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::UpdateCollateral,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.credit_facility.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        updated_collateral: Satoshis,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let audit_info = self
            .user_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let credit_facility_collateral_update =
            credit_facility.initiate_collateral_update(updated_collateral)?;

        let mut db_tx = self.pool.begin().await?;
        let executed_at = self
            .ledger
            .update_credit_facility_collateral(credit_facility_collateral_update.clone())
            .await?;

        credit_facility.confirm_collateral_update(
            credit_facility_collateral_update,
            executed_at,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
        );

        activate::execute(
            &mut credit_facility,
            &mut db_tx,
            &self.ledger,
            self.authz.audit(),
            self.interest_accrual_repo.clone(),
            &self.jobs,
            price,
        )
        .await?;
        self.credit_facility_repo
            .update_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        db_tx.commit().await?;
        Ok(credit_facility)
    }

    pub async fn user_can_record_payment(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::RecordPayment,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.credit_facility.record_payment", skip(self), err)]
    pub async fn record_payment(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut db_tx = self.pool.begin().await?;

        let audit_info = self
            .user_can_record_payment(sub, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let facility_balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?
            .into();

        if credit_facility.outstanding() != facility_balances {
            return Err(CreditFacilityError::ReceivableBalanceMismatch);
        }

        let customer = self
            .customers
            .repo()
            .find_by_id(credit_facility.customer_id)
            .await?;
        self.ledger
            .get_customer_balance(customer.account_ids)
            .await?
            .check_withdraw_amount(amount)?;

        let repayment = credit_facility.initiate_repayment(amount)?;
        let executed_at = self
            .ledger
            .record_credit_facility_repayment(repayment.clone())
            .await?;
        credit_facility.confirm_repayment(
            repayment,
            executed_at,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
        );
        self.credit_facility_repo
            .update_in_tx(&mut db_tx, &mut credit_facility)
            .await?;

        self.credit_facility_repo
            .update_in_tx(&mut db_tx, &mut credit_facility)
            .await?;

        db_tx.commit().await?;

        Ok(credit_facility)
    }

    #[instrument(name = "lava.credit_facility.list_for_customer", skip(self), err)]
    pub async fn list_for_customer(
        &self,
        sub: Option<&Subject>,
        customer_id: CustomerId,
    ) -> Result<Vec<CreditFacility>, CreditFacilityError> {
        if let Some(sub) = sub {
            self.authz
                .enforce_permission(
                    sub,
                    Object::Customer(CustomerAllOrOne::ById(customer_id)),
                    CreditFacilityAction::List,
                )
                .await?;
        }

        Ok(self
            .credit_facility_repo
            .list_for_customer_id_by_created_at(
                customer_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[instrument(name = "lava.credit_facility.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilityByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilityByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await
    }

    pub async fn user_can_complete(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::Complete,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.credit_facility.complete", skip(self), err)]
    pub async fn complete_facility(
        &self,
        sub: &Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let credit_facility_id = credit_facility_id.into();

        let audit_info = self
            .user_can_complete(sub, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let completion = credit_facility.initiate_completion()?;

        let executed_at = self
            .ledger
            .complete_credit_facility(completion.clone())
            .await?;
        credit_facility.confirm_completion(
            completion,
            executed_at,
            audit_info,
            price,
            self.config.upgrade_buffer_cvl_pct,
        );

        let mut db_tx = self.pool.begin().await?;
        self.credit_facility_repo
            .update_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        db_tx.commit().await?;

        Ok(credit_facility)
    }

    pub async fn list_disbursements(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Vec<Disbursement>, CreditFacilityError> {
        self.authz
            .enforce_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::ListDisbursement,
            )
            .await?;

        let disbursements = self
            .disbursement_repo
            .list_for_credit_facility_id_by_created_at(
                credit_facility_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities;
        Ok(disbursements)
    }

    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<HashMap<CreditFacilityId, T>, CreditFacilityError> {
        self.credit_facility_repo.find_all(ids).await
    }
}
