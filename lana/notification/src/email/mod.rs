pub mod config;
pub mod error;
pub mod job;
pub mod templates;

use ::job::{JobId, Jobs};
use core_access::user::Users;
use core_credit::{
    CoreCredit, CreditFacilityId, CustomerId, ObligationId, ObligationType, PriceOfOneBTC,
};
use core_customer::Customers;
use domain_config::ExposedDomainConfigsReadOnly;
use job::{EmailSenderConfig, EmailSenderInit, EmailSenderJobSpawner};
use lana_events::LanaEvent;
use smtp_client::SmtpClient;

use templates::{
    DepositAccountCreatedEmailData, EmailTemplate, EmailType, OverduePaymentEmailData,
    PartialLiquidationInitiatedEmailData, RoleCreatedEmailData, UnderMarginCallEmailData,
};

pub use config::{EmailInfraConfig, NotificationFromEmail, NotificationFromName};
pub use error::EmailError;

#[derive(Clone)]
pub struct EmailNotification<Perms>
where
    Perms: authz::PermissionCheck,
{
    users: Users<Perms::Audit, LanaEvent>,
    credit: CoreCredit<Perms, LanaEvent>,
    customers: Customers<Perms, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
    _authz: std::marker::PhantomData<Perms>,
}

impl<Perms> EmailNotification<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    pub async fn init(
        jobs: &mut Jobs,
        domain_configs: &ExposedDomainConfigsReadOnly,
        infra_config: EmailInfraConfig,
        users: &Users<Perms::Audit, LanaEvent>,
        credit: &CoreCredit<Perms, LanaEvent>,
        customers: &Customers<Perms, LanaEvent>,
    ) -> Result<Self, EmailError> {
        let template = EmailTemplate::new(infra_config.admin_panel_url.clone())?;
        let smtp_client = SmtpClient::init(infra_config.to_smtp_config())?;

        let email_sender_job_spawner = jobs.add_initializer(EmailSenderInit::new(
            smtp_client,
            template,
            domain_configs.clone(),
        ));

        Ok(Self {
            users: users.clone(),
            credit: credit.clone(),
            customers: customers.clone(),
            email_sender_job_spawner,
            _authz: std::marker::PhantomData,
        })
    }

    pub async fn send_obligation_overdue_notification_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        obligation_id: &ObligationId,
        credit_facility_id: &CreditFacilityId,
        amount: &core_money::UsdCents,
    ) -> Result<(), EmailError> {
        let obligation = self
            .credit
            .obligations()
            .find_by_id_without_audit(*obligation_id)
            .await?;

        let credit_facility = self
            .credit
            .facilities()
            .find_by_id_without_audit(*credit_facility_id)
            .await?;

        let customer = self
            .customers
            .find_by_id_without_audit(credit_facility.customer_id)
            .await?;

        let email_data = OverduePaymentEmailData {
            public_id: credit_facility.public_id.to_string(),
            payment_type: match obligation.obligation_type {
                ObligationType::Disbursal => "Principal Repayment".to_string(),
                ObligationType::Interest => "Interest Payment".to_string(),
            },
            original_amount: obligation.initial_amount,
            outstanding_amount: *amount,
            due_date: obligation.due_at(),
            customer_email: customer.email,
        };

        let mut has_next_page = true;
        let mut after = None;
        // currently email notifications are sent to all users in the system
        // TODO: create a role for receiving margin call / overdue payment emails
        while has_next_page {
            let es_entity::PaginatedQueryRet {
                entities: users,
                has_next_page: next_page,
                end_cursor,
            } = self
                .users
                .list_users_without_audit(
                    es_entity::PaginatedQueryArgs { first: 20, after },
                    es_entity::ListDirection::Descending,
                )
                .await?;
            (after, has_next_page) = (end_cursor, next_page);

            for user in users {
                let email_config = EmailSenderConfig {
                    recipient: user.email,
                    email_type: EmailType::OverduePayment(email_data.clone()),
                };
                self.email_sender_job_spawner
                    .spawn_in_op(op, JobId::new(), email_config)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn send_partial_liquidation_initiated_notification_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        credit_facility_id: &CreditFacilityId,
        customer_id: &CustomerId,
        trigger_price: &PriceOfOneBTC,
        initially_estimated_to_liquidate: &core_money::Satoshis,
        initially_expected_to_receive: &core_money::UsdCents,
    ) -> Result<(), EmailError> {
        let customer = self
            .customers
            .find_by_id_without_audit(*customer_id)
            .await?;

        let email_data = PartialLiquidationInitiatedEmailData {
            facility_id: credit_facility_id.to_string(),
            trigger_price: *trigger_price,
            initially_estimated_to_liquidate: *initially_estimated_to_liquidate,
            initially_expected_to_receive: *initially_expected_to_receive,
        };

        let mut has_next_page = true;
        let mut after = None;
        while has_next_page {
            let es_entity::PaginatedQueryRet {
                entities: users,
                has_next_page: next_page,
                end_cursor,
            } = self
                .users
                .list_users_without_audit(
                    es_entity::PaginatedQueryArgs { first: 20, after },
                    es_entity::ListDirection::Descending,
                )
                .await?;
            (after, has_next_page) = (end_cursor, next_page);

            for user in users {
                let email_config = EmailSenderConfig {
                    recipient: user.email,
                    email_type: EmailType::PartialLiquidationInitiated(email_data.clone()),
                };
                self.email_sender_job_spawner
                    .spawn_in_op(op, JobId::new(), email_config)
                    .await?;
            }
        }

        let email_config = EmailSenderConfig {
            recipient: customer.email,
            email_type: EmailType::PartialLiquidationInitiated(email_data),
        };

        self.email_sender_job_spawner
            .spawn_in_op(op, JobId::new(), email_config)
            .await?;
        Ok(())
    }

    pub async fn send_under_margin_call_notification_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        credit_facility_id: &CreditFacilityId,
        customer_id: &CustomerId,
        effective: &chrono::NaiveDate,
        collateral: &core_money::Satoshis,
        outstanding_disbursed: &core_money::UsdCents,
        outstanding_interest: &core_money::UsdCents,
        price: &PriceOfOneBTC,
    ) -> Result<(), EmailError> {
        let customer = self
            .customers
            .find_by_id_without_audit(*customer_id)
            .await?;

        let total_outstanding = *outstanding_disbursed + *outstanding_interest;
        let email_data = UnderMarginCallEmailData {
            facility_id: credit_facility_id.to_string(),
            effective: *effective,
            collateral: *collateral,
            outstanding_disbursed: *outstanding_disbursed,
            outstanding_interest: *outstanding_interest,
            total_outstanding,
            price: *price,
        };

        let email_config = EmailSenderConfig {
            recipient: customer.email,
            email_type: EmailType::UnderMarginCall(email_data),
        };

        self.email_sender_job_spawner
            .spawn_in_op(op, JobId::new(), email_config)
            .await?;
        Ok(())
    }

    pub async fn send_deposit_account_created_notification_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        account_id: &core_deposit::DepositAccountId,
        account_holder_id: &core_deposit::DepositAccountHolderId,
    ) -> Result<(), EmailError> {
        let customer_id: core_customer::CustomerId = (*account_holder_id).into();
        let customer = self.customers.find_by_id_without_audit(customer_id).await?;

        let email_data = DepositAccountCreatedEmailData {
            account_id: account_id.to_string(),
            customer_email: customer.email.clone(),
        };

        let email_config = EmailSenderConfig {
            recipient: customer.email,
            email_type: EmailType::DepositAccountCreated(email_data),
        };

        self.email_sender_job_spawner
            .spawn_in_op(op, JobId::new(), email_config)
            .await?;

        Ok(())
    }

    pub async fn send_role_created_notification_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        role_id: &core_access::RoleId,
        role_name: &str,
    ) -> Result<(), EmailError> {
        let email_data = RoleCreatedEmailData {
            role_id: role_id.to_string(),
            role_name: role_name.to_string(),
        };

        let mut has_next_page = true;
        let mut after = None;
        // Send email to all users in the system
        while has_next_page {
            let es_entity::PaginatedQueryRet {
                entities: users,
                has_next_page: next_page,
                end_cursor,
            } = self
                .users
                .list_users_without_audit(
                    es_entity::PaginatedQueryArgs { first: 20, after },
                    es_entity::ListDirection::Descending,
                )
                .await?;
            (after, has_next_page) = (end_cursor, next_page);

            for user in users {
                let email_config = EmailSenderConfig {
                    recipient: user.email,
                    email_type: EmailType::RoleCreated(email_data.clone()),
                };
                self.email_sender_job_spawner
                    .spawn_in_op(op, JobId::new(), email_config)
                    .await?;
            }
        }
        Ok(())
    }
}
