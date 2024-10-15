mod config;
mod cursor;
mod disbursement;
mod entity;
pub mod error;
mod history;
mod interest_accrual;
mod jobs;
mod repo;

use crate::{
    audit::Audit,
    authorization::{Authorization, CreditFacilityAction, CustomerAllOrOne, Object},
    customer::Customers,
    data_export::Export,
    entity::EntityError,
    job::Jobs,
    ledger::{credit_facility::*, Ledger},
    price::Price,
    primitives::{
        AuditInfo, CreditFacilityId, CustomerId, DisbursementIdx, Satoshis, Subject, UsdCents,
        UserId,
    },
    terms::TermValues,
    user::{UserRepo, Users},
};

pub use config::*;
pub use cursor::*;
pub use disbursement::*;
pub use entity::*;
use error::*;
pub use history::*;
pub use interest_accrual::*;
use jobs::*;
use repo::*;
use tracing::instrument;

#[derive(Clone)]
pub struct CreditFacilities {
    pool: sqlx::PgPool,
    authz: Authorization,
    customers: Customers,
    credit_facility_repo: CreditFacilityRepo,
    disbursement_repo: DisbursementRepo,
    interest_accrual_repo: InterestAccrualRepo,
    user_repo: UserRepo,
    ledger: Ledger,
    price: Price,
    jobs: Jobs,
    config: CreditFacilityConfig,
}

impl CreditFacilities {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: &sqlx::PgPool,
        config: CreditFacilityConfig,
        jobs: &Jobs,
        export: &Export,
        authz: &Authorization,
        audit: &Audit,
        customers: &Customers,
        users: &Users,
        ledger: &Ledger,
        price: &Price,
    ) -> Self {
        let credit_facility_repo = CreditFacilityRepo::new(pool, export);
        let disbursement_repo = DisbursementRepo::new(pool, export);
        let interest_accrual_repo = InterestAccrualRepo::new(pool, export);
        jobs.add_initializer(interest::FacilityProcessingJobInitializer::new(
            ledger,
            credit_facility_repo.clone(),
            interest_accrual_repo.clone(),
            audit,
        ));

        Self {
            pool: pool.clone(),
            authz: authz.clone(),
            customers: customers.clone(),
            credit_facility_repo,
            disbursement_repo,
            interest_accrual_repo,
            user_repo: users.repo().clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            jobs: jobs.clone(),
            config,
        }
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

    #[instrument(name = "lava.credit_facility.create", skip(self), err)]
    pub async fn create(
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

        let new_credit_facility = NewCreditFacility::builder()
            .id(CreditFacilityId::new())
            .customer_id(customer_id)
            .terms(terms)
            .facility(facility)
            .account_ids(CreditFacilityAccountIds::new())
            .customer_account_ids(customer.account_ids)
            .audit_info(audit_info)
            .build()
            .expect("could not build new credit facility");

        let mut db_tx = self.pool.begin().await?;
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

    pub async fn user_can_approve(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::Approve,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.credit_facility.add_approval", skip(self), err)]
    pub async fn add_approval(
        &self,
        sub: &Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let credit_facility_id = credit_facility_id.into();

        let audit_info = self
            .user_can_approve(sub, true)
            .await?
            .expect("audit info missing");

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let subject_id = uuid::Uuid::from(sub);
        let user = self.user_repo.find_by_id(UserId::from(subject_id)).await?;

        let mut db_tx = self.pool.begin().await?;
        let price = self.price.usd_cents_per_btc().await?;

        if let Some(credit_facility_approval) =
            credit_facility.add_approval(user.id, user.current_roles(), audit_info, price)?
        {
            let executed_at = self
                .ledger
                .approve_credit_facility(credit_facility_approval.clone())
                .await?;
            credit_facility.confirm_approval(credit_facility_approval, executed_at, audit_info);
        }

        self.credit_facility_repo
            .persist_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        db_tx.commit().await?;

        Ok(credit_facility)
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
        let new_disbursement = credit_facility.initiate_disbursement(audit_info, amount)?;
        self.credit_facility_repo
            .persist_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        let disbursement = self
            .disbursement_repo
            .create_in_tx(&mut db_tx, new_disbursement)
            .await?;

        db_tx.commit().await?;
        Ok(disbursement)
    }

    pub async fn user_can_approve_disbursement(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CreditFacilityError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::CreditFacility,
                CreditFacilityAction::ApproveDisbursement,
                enforce,
            )
            .await?)
    }

    #[instrument(
        name = "lava.credit_facility.add_disbursement_approval",
        skip(self),
        err
    )]
    pub async fn add_disbursement_approval(
        &self,
        sub: &Subject,
        credit_facility_id: CreditFacilityId,
        disbursement_idx: DisbursementIdx,
    ) -> Result<Disbursement, CreditFacilityError> {
        let audit_info = self
            .user_can_approve_disbursement(sub, true)
            .await?
            .expect("audit info missing");

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let subject_id = uuid::Uuid::from(sub);
        let user = self.user_repo.find_by_id(UserId::from(subject_id)).await?;

        let mut disbursement = self
            .disbursement_repo
            .find_by_idx_for_credit_facility(credit_facility_id, disbursement_idx)
            .await?;

        let mut db_tx = self.pool.begin().await?;

        if let Some(disbursement_data) =
            disbursement.add_approval(user.id, user.current_roles(), audit_info)?
        {
            let executed_at = self
                .ledger
                .record_disbursement(disbursement_data.clone())
                .await?;
            disbursement.confirm_approval(&disbursement_data, executed_at, audit_info);

            credit_facility.confirm_disbursement(
                &disbursement,
                disbursement_data.tx_id,
                executed_at,
                audit_info,
            );

            if disbursement.idx == DisbursementIdx::FIRST {
                let new_accrual = credit_facility
                    .start_interest_accrual(audit_info)?
                    .expect("Accrual start date is before facility expiry date");
                let accrual = self
                    .interest_accrual_repo
                    .create_in_tx(&mut db_tx, new_accrual)
                    .await?;
                self.jobs
                    .create_and_spawn_at_in_tx::<interest::FacilityProcessingJobInitializer, _>(
                        &mut db_tx,
                        format!("credit-facility-interest-processing-{}", credit_facility.id),
                        interest::CreditFacilityJobConfig {
                            credit_facility_id: credit_facility.id,
                        },
                        accrual
                            .next_incurrence_period()
                            .expect("New accrual has first incurrence period")
                            .end,
                    )
                    .await?;
            }
        }

        self.disbursement_repo
            .persist_in_tx(&mut db_tx, &mut disbursement)
            .await?;
        self.credit_facility_repo
            .persist_in_tx(&mut db_tx, &mut credit_facility)
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
        self.credit_facility_repo
            .persist_in_tx(&mut db_tx, &mut credit_facility)
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
            .persist_in_tx(&mut db_tx, &mut credit_facility)
            .await?;

        self.credit_facility_repo
            .persist_in_tx(&mut db_tx, &mut credit_facility)
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

        self.credit_facility_repo
            .find_for_customer(customer_id)
            .await
    }

    #[instrument(name = "lava.credit_facility.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &Subject,
        query: crate::query::PaginatedQueryArgs<CreditFacilityByCreatedAtCursor>,
    ) -> Result<
        crate::query::PaginatedQueryRet<CreditFacility, CreditFacilityByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .enforce_permission(sub, Object::CreditFacility, CreditFacilityAction::List)
            .await?;
        self.credit_facility_repo.list(query).await
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
            .persist_in_tx(&mut db_tx, &mut credit_facility)
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

        let disbursements = self.disbursement_repo.list(credit_facility_id).await?;
        Ok(disbursements)
    }
}
