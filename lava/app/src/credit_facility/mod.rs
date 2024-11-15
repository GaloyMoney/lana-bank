mod activate;
mod config;
mod disbursal;
mod entity;
pub mod error;
mod history;
mod interest_accrual;
mod jobs;
mod processes;
mod publisher;
mod repo;

use std::collections::HashMap;

use authz::PermissionCheck;

use crate::{
    audit::{AuditInfo, AuditSvc},
    authorization::{Authorization, CreditFacilityAction, Object},
    customer::Customers,
    data_export::Export,
    governance::Governance,
    job::{error::JobError, *},
    ledger::{credit_facility::*, Ledger},
    outbox::Outbox,
    price::Price,
    primitives::{
        CreditFacilityId, CreditFacilityStatus, CustomerId, DisbursalId, PriceOfOneBTC, Satoshis,
        Subject, UsdCents,
    },
    terms::{CollateralizationState, TermValues},
};

pub use config::*;
pub use disbursal::{disbursal_cursor::*, *};
pub use entity::*;
use error::*;
pub use history::*;
pub use interest_accrual::*;
use jobs::*;
pub use processes::approve_credit_facility::*;
pub use processes::approve_disbursal::*;
use publisher::CreditFacilityPublisher;
pub use repo::credit_facility_cursor::*;
use repo::CreditFacilityRepo;
use tracing::instrument;

#[derive(Clone)]
pub struct CreditFacilities {
    authz: Authorization,
    customers: Customers,
    credit_facility_repo: CreditFacilityRepo,
    disbursal_repo: DisbursalRepo,
    governance: Governance,
    jobs: Jobs,
    ledger: Ledger,
    price: Price,
    config: CreditFacilityConfig,
    approve_disbursal: ApproveDisbursal,
    approve_credit_facility: ApproveCreditFacility,
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
        customers: &Customers,
        ledger: &Ledger,
        price: &Price,
        outbox: &Outbox,
    ) -> Result<Self, CreditFacilityError> {
        let publisher = CreditFacilityPublisher::new(export, outbox);
        let credit_facility_repo = CreditFacilityRepo::new(pool, &publisher);
        let disbursal_repo = DisbursalRepo::new(pool, export);
        let approve_disbursal = ApproveDisbursal::new(
            &disbursal_repo,
            &credit_facility_repo,
            authz.audit(),
            governance,
            ledger,
        );
        let approve_credit_facility = ApproveCreditFacility::new(
            &credit_facility_repo,
            ledger,
            price,
            jobs,
            authz.audit(),
            governance,
        );
        jobs.add_initializer_and_spawn_unique(
            cvl::CreditFacilityProcessingJobInitializer::new(
                credit_facility_repo.clone(),
                price,
                authz.audit(),
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
            authz.audit(),
        ));
        jobs.add_initializer_and_spawn_unique(
            CreditFacilityApprovalJobInitializer::new(outbox, &approve_credit_facility),
            CreditFacilityApprovalJobConfig,
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            DisbursalApprovalJobInitializer::new(outbox, &approve_disbursal),
            DisbursalApprovalJobConfig,
        )
        .await?;
        let _ = governance
            .init_policy(APPROVE_CREDIT_FACILITY_PROCESS)
            .await;
        let _ = governance.init_policy(APPROVE_DISBURSAL_PROCESS).await;

        Ok(Self {
            authz: authz.clone(),
            customers: customers.clone(),
            credit_facility_repo,
            disbursal_repo,
            governance: governance.clone(),
            jobs: jobs.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            config,
            approve_disbursal,
            approve_credit_facility,
        })
    }

    pub async fn subject_can_create(
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

    #[instrument(name = "credit_facility.initiate", skip(self), err)]
    pub async fn initiate(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        facility: UsdCents,
        terms: TermValues,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let customer_id = customer_id.into();

        let audit_info = self
            .subject_can_create(sub, true)
            .await?
            .expect("audit info missing");

        let customer = match self.customers.find_by_id(sub, customer_id).await? {
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

        let mut db = self.credit_facility_repo.begin_op().await?;
        self.governance
            .start_process(&mut db, id, id.to_string(), APPROVE_CREDIT_FACILITY_PROCESS)
            .await?;
        let credit_facility = self
            .credit_facility_repo
            .create_in_op(&mut db, new_credit_facility)
            .await?;
        self.ledger
            .create_accounts_for_credit_facility(credit_facility.id, credit_facility.account_ids)
            .await?;

        db.commit().await?;

        Ok(credit_facility)
    }

    #[instrument(name = "credit_facility.find", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::Read)
            .await?;

        match self.credit_facility_repo.find_by_id(id.into()).await {
            Ok(credit_facility) => Ok(Some(credit_facility)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "credit_facility.balance", skip(self), err)]
    pub async fn balance(
        &self,
        sub: &Subject,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacilityBalance, CreditFacilityError> {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::Read)
            .await?;

        let credit_facility = self.credit_facility_repo.find_by_id(id.into()).await?;

        Ok(credit_facility.balances())
    }

    pub async fn subject_can_initiate_disbursal(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::InitiateDisbursal,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "credit_facility.initiate_disbursal", skip(self), err)]
    #[es_entity::retry_on_concurrent_modification]
    pub async fn initiate_disbursal(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<Disbursal, CreditFacilityError> {
        let audit_info = self
            .subject_can_initiate_disbursal(sub, true)
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
        balances.check_disbursal_amount(amount)?;

        let mut db = self.credit_facility_repo.begin_op().await?;
        let now = crate::time::now();
        let new_disbursal = credit_facility.initiate_disbursal(amount, now, audit_info)?;
        self.governance
            .start_process(
                &mut db,
                new_disbursal.approval_process_id,
                new_disbursal.approval_process_id.to_string(),
                APPROVE_DISBURSAL_PROCESS,
            )
            .await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        let disbursal = self
            .disbursal_repo
            .create_in_op(&mut db, new_disbursal)
            .await?;

        db.commit().await?;
        Ok(disbursal)
    }

    #[instrument(name = "credit_facility.find_disbursal_by_id", skip(self), err)]
    pub async fn find_disbursal_by_id(
        &self,
        sub: &Subject,
        id: impl Into<DisbursalId> + std::fmt::Debug,
    ) -> Result<Option<Disbursal>, CreditFacilityError> {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::Read)
            .await?;

        match self.disbursal_repo.find_by_id(id.into()).await {
            Ok(loan) => Ok(Some(loan)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn ensure_up_to_date_disbursal_status(
        &self,
        disbursal: &Disbursal,
    ) -> Result<Option<Disbursal>, CreditFacilityError> {
        self.approve_disbursal.execute_from_svc(disbursal).await
    }

    pub async fn ensure_up_to_date_status(
        &self,
        credit_facility: &CreditFacility,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        self.approve_credit_facility
            .execute_from_svc(credit_facility)
            .await
    }

    pub async fn subject_can_update_collateral(
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

    #[es_entity::retry_on_concurrent_modification]
    #[instrument(name = "credit_facility.update_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        updated_collateral: Satoshis,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let audit_info = self
            .subject_can_update_collateral(sub, true)
            .await?
            .expect("audit info missing");

        let price = self.price.usd_cents_per_btc().await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let credit_facility_collateral_update =
            credit_facility.initiate_collateral_update(updated_collateral)?;

        let mut db = self.credit_facility_repo.begin_op().await?;
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
            &mut db,
            &self.ledger,
            self.authz.audit(),
            &self.credit_facility_repo,
            &self.jobs,
            price,
        )
        .await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        db.commit().await?;
        Ok(credit_facility)
    }

    pub async fn subject_can_record_payment(
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

    #[instrument(name = "credit_facility.record_payment", skip(self), err)]
    pub async fn record_payment(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut db = self.credit_facility_repo.begin_op().await?;

        let audit_info = self
            .subject_can_record_payment(sub, true)
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
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        db.commit().await?;

        Ok(credit_facility)
    }

    #[instrument(name = "credit_facility.list_for_customer", skip(self), err)]
    pub async fn list_for_customer(
        &self,
        sub: &Subject,
        customer_id: CustomerId,
    ) -> Result<Vec<CreditFacility>, CreditFacilityError> {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;

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

    #[instrument(name = "credit_facility.list_by_created_at", skip(self), err)]
    pub async fn list_by_created_at(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
            .list_by_created_at(query, direction.into())
            .await
    }

    #[instrument(
        name = "credit_facility.list_by_created_at_for_status",
        skip(self),
        err
    )]
    pub async fn list_by_created_at_for_status(
        &self,
        sub: &Subject,
        status: CreditFacilityStatus,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
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
        sub: &Subject,
        collateralization_state: CollateralizationState,
        query: es_entity::PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
            .list_for_collateralization_state_by_created_at(
                collateralization_state,
                query,
                direction.into(),
            )
            .await
    }

    #[instrument(
        name = "credit_facility.list_by_collateralization_ratio",
        skip(self),
        err
    )]
    pub async fn list_by_collateralization_ratio(
        &self,
        sub: &Subject,
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
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
            .list_by_collateralization_ratio(query, direction.into())
            .await
    }

    #[instrument(
        name = "credit_facility.list_by_collateralization_ratio_for_status",
        skip(self),
        err
    )]
    pub async fn list_by_collateralization_ratio_for_status(
        &self,
        sub: &Subject,
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
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
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
        sub: &Subject,
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
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo
            .list_for_collateralization_state_by_collateralization_ratio(
                collateralization_state,
                query,
                direction.into(),
            )
            .await
    }

    pub async fn subject_can_complete(
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

    #[instrument(name = "credit_facility.complete", skip(self), err)]
    pub async fn complete_facility(
        &self,
        sub: &Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
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

        let mut db = self.credit_facility_repo.begin_op().await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        db.commit().await?;

        Ok(credit_facility)
    }

    pub async fn list_disbursals_for_credit_facility(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Vec<Disbursal>, CreditFacilityError> {
        self.authz
            .enforce_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::ListDisbursals,
            )
            .await?;

        let disbursals = self
            .disbursal_repo
            .list_for_credit_facility_id_by_created_at(
                credit_facility_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities;
        Ok(disbursals)
    }

    pub async fn list_disbursals_by_created_at(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<DisbursalsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Disbursal, DisbursalsByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::ListDisbursals,
            )
            .await?;

        let disbursals = self
            .disbursal_repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?;
        Ok(disbursals)
    }

    pub async fn find_all<T: From<CreditFacility>>(
        &self,
        ids: &[CreditFacilityId],
    ) -> Result<HashMap<CreditFacilityId, T>, CreditFacilityError> {
        self.credit_facility_repo.find_all(ids).await
    }

    pub async fn find_all_disbursals<T: From<Disbursal>>(
        &self,
        ids: &[DisbursalId],
    ) -> Result<HashMap<DisbursalId, T>, CreditFacilityError> {
        Ok(self.disbursal_repo.find_all(ids).await?)
    }
}

#[cfg(test)]
mod test {
    use audit::{Audit, AuditEntryId};
    use authz::Authorization;
    use chrono::Utc;
    use core_user::{Role, Users};
    use lava_events::LavaEvent;
    use rbac_types::{CustomerAction, CustomerAllOrOne, LavaAction, LavaObject, LavaRole};
    use rust_decimal_macros::dec;

    use rand::distributions::{Alphanumeric, DistString};

    use crate::{
        customer::CustomerConfig,
        ledger::{customer::CustomerLedgerAccountIds, LedgerConfig},
        terms::{Duration, InterestInterval},
    };

    use super::*;

    #[tokio::test]
    async fn interest_accrual_lifecycle_2() -> anyhow::Result<()> {
        pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
            let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
            let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
            let pool = sqlx::PgPool::connect(&pg_con).await?;
            Ok(pool)
        }

        pub async fn init_users(
            pool: &sqlx::PgPool,
            authz: &Authorization<Audit<Subject, LavaObject, LavaAction>, Role>,
        ) -> anyhow::Result<(
            Users<Audit<Subject, LavaObject, LavaAction>, LavaEvent>,
            Subject,
        )> {
            let superuser_email = "superuser@test.io".to_string();
            let outbox = Outbox::init(pool).await?;
            let users = Users::init(pool, authz, &outbox, Some(superuser_email.clone())).await?;
            let superuser = users
                .find_by_email(None, &superuser_email)
                .await?
                .expect("Superuser not found");
            Ok((users, Subject::from(superuser.id)))
        }

        fn generate_random_username() -> String {
            let random_string: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
            random_string.to_lowercase()
        }

        fn generate_email(username: &String) -> String {
            format!("{}@example.com", username)
        }

        fn default_terms() -> TermValues {
            TermValues::builder()
                .annual_rate(dec!(12))
                .duration(Duration::Months(3))
                .accrual_interval(InterestInterval::EndOfMonth)
                .incurrence_interval(InterestInterval::EndOfDay)
                .liquidation_cvl(dec!(105))
                .margin_call_cvl(dec!(125))
                .initial_cvl(dec!(140))
                .build()
                .expect("should build a valid term")
        }

        fn dummy_audit_info() -> AuditInfo {
            AuditInfo {
                audit_entry_id: AuditEntryId::from(1),
                sub: "sub".to_string(),
            }
        }

        let pool = init_pool().await?;
        let jobs = Jobs::new(&pool, JobExecutorConfig::default());
        let export = Export::new(LedgerConfig::default().cala_url.clone(), &jobs);
        let outbox = Outbox::init(&pool).await?;
        let publisher = CreditFacilityPublisher::new(&export, &outbox);
        let credit_facility_repo = CreditFacilityRepo::new(&pool, &publisher);
        let disbursal_repo = DisbursalRepo::new(&pool, &export);

        let audit = Audit::new(&pool);
        let authz = Authorization::init(&pool, &audit).await?;
        authz
            .add_permission_to_role(
                &LavaRole::SUPERUSER,
                Object::Customer(CustomerAllOrOne::All),
                CustomerAction::Create,
            )
            .await?;

        let governance = Governance::new(&pool, &authz, &outbox);
        let _ = governance
            .init_policy(APPROVE_CREDIT_FACILITY_PROCESS)
            .await;
        let _ = governance.init_policy(APPROVE_DISBURSAL_PROCESS).await;

        let ledger = Ledger::init(LedgerConfig::default(), &authz).await?;
        let customers =
            Customers::new(&pool, &&CustomerConfig::default(), &ledger, &authz, &export);

        let price = Price::init(&jobs, &export).await?;

        let (_, superuser_subject) = init_users(&pool, &authz).await?;

        let username = &generate_random_username();
        let customer = customers
            .create(
                &superuser_subject,
                generate_email(username),
                username.to_string(),
            )
            .await?;

        let id = CreditFacilityId::new();

        let new_credit_facility = NewCreditFacility::builder()
            .id(id)
            .approval_process_id(id)
            .customer_id(customer.id)
            .terms(default_terms())
            .facility(UsdCents::from(1_000_000_00))
            .account_ids(CreditFacilityAccountIds::new())
            .customer_account_ids(CustomerLedgerAccountIds::new())
            .audit_info(dummy_audit_info())
            .build()
            .expect("could not build new credit facility");
        let mut db = credit_facility_repo.begin_op().await?;
        governance
            .start_process(&mut db, id, id.to_string(), APPROVE_CREDIT_FACILITY_PROCESS)
            .await?;
        let mut credit_facility = credit_facility_repo
            .create_in_op(&mut db, new_credit_facility)
            .await?;

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .did_execute();

        let credit_facility_collateral_update =
            credit_facility.initiate_collateral_update(Satoshis::from(100_00_000_000))?;

        let price = price.usd_cents_per_btc().await?;
        credit_facility.confirm_collateral_update(
            credit_facility_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            price,
            CreditFacilityConfig::default().upgrade_buffer_cvl_pct,
        );
        credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        let credit_facility_activation = credit_facility.activation_data(price)?;
        credit_facility
            .activate(credit_facility_activation, Utc::now(), dummy_audit_info())
            .did_execute();
        credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        assert_eq!(credit_facility.status(), CreditFacilityStatus::Active);

        let new_disbursal = credit_facility.initiate_disbursal(
            UsdCents::from(100_000_00),
            Utc::now(),
            dummy_audit_info(),
        )?;
        governance
            .start_process(
                &mut db,
                new_disbursal.approval_process_id,
                new_disbursal.approval_process_id.to_string(),
                APPROVE_DISBURSAL_PROCESS,
            )
            .await?;
        credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        let mut disbursal = disbursal_repo.create_in_op(&mut db, new_disbursal).await?;
        disbursal
            .approval_process_concluded(true, dummy_audit_info())
            .did_execute();
        let disbursal_data = disbursal.disbursal_data()?;
        disbursal.confirm(&disbursal_data, Utc::now(), dummy_audit_info());
        credit_facility.confirm_disbursal(
            &disbursal,
            Some(disbursal_data.tx_id),
            Utc::now(),
            dummy_audit_info(),
        );

        let account_ids = credit_facility.account_ids;
        {
            let outstanding = credit_facility.outstanding();
            let accrual = credit_facility
                .interest_accrual_in_progress()
                .expect("Accrual in progress should exist for scheduled job");
            assert_eq!(accrual.count_incurred(), 0);

            let interest_incurrence = accrual.initiate_incurrence(outstanding, account_ids);
            accrual.confirm_incurrence(interest_incurrence, dummy_audit_info());
        }
        credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        db.commit().await?;

        credit_facility = credit_facility_repo.find_by_id(credit_facility.id).await?;
        let accrual = credit_facility
            .interest_accrual_in_progress()
            .expect("Accrual in progress should exist for scheduled job");
        assert_eq!(accrual.count_incurred(), 1);
        Ok(())
    }
}
