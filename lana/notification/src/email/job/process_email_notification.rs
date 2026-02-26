use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

use core_access::RoleId;
use core_credit::{CreditFacilityId, CustomerId, PriceOfOneBTC};
use core_credit_collection::ObligationId;
use core_deposit::{DepositAccountHolderId, DepositAccountId};
use money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EmailNotificationConfig {
    ObligationOverdue {
        obligation_id: ObligationId,
        credit_facility_id: CreditFacilityId,
        outstanding_amount: UsdCents,
    },
    PartialLiquidationInitiated {
        credit_facility_id: CreditFacilityId,
        customer_id: CustomerId,
        trigger_price: PriceOfOneBTC,
        initially_estimated_to_liquidate: Satoshis,
        initially_expected_to_receive: UsdCents,
    },
    UnderMarginCallThreshold {
        credit_facility_id: CreditFacilityId,
        customer_id: CustomerId,
        effective: NaiveDate,
        collateral: Satoshis,
        outstanding_disbursed: UsdCents,
        outstanding_interest: UsdCents,
        price: PriceOfOneBTC,
    },
    DepositAccountCreated {
        account_id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
    },
    RoleCreated {
        role_id: RoleId,
        role_name: String,
    },
}

pub const PROCESS_EMAIL_NOTIFICATION_COMMAND: JobType =
    JobType::new("command.notification.process-email-notification");

pub struct ProcessEmailNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck,
{
    email_notification: EmailNotification<Perms>,
}

impl<Perms> ProcessEmailNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck,
{
    pub fn new(email_notification: EmailNotification<Perms>) -> Self {
        Self { email_notification }
    }
}

impl<Perms> JobInitializer for ProcessEmailNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    type Config = EmailNotificationConfig;

    fn job_type(&self) -> JobType {
        PROCESS_EMAIL_NOTIFICATION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessEmailNotificationJobRunner {
            config: job.config()?,
            email_notification: self.email_notification.clone(),
        }))
    }
}

pub struct ProcessEmailNotificationJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
{
    config: EmailNotificationConfig,
    email_notification: EmailNotification<Perms>,
}

#[async_trait]
impl<Perms> JobRunner for ProcessEmailNotificationJobRunner<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "notification.process_email_notification_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        match &self.config {
            EmailNotificationConfig::ObligationOverdue {
                obligation_id,
                credit_facility_id,
                outstanding_amount,
            } => {
                self.email_notification
                    .send_obligation_overdue_notification_in_op(
                        &mut op,
                        obligation_id,
                        credit_facility_id,
                        outstanding_amount,
                    )
                    .await?;
            }
            EmailNotificationConfig::PartialLiquidationInitiated {
                credit_facility_id,
                customer_id,
                trigger_price,
                initially_estimated_to_liquidate,
                initially_expected_to_receive,
            } => {
                self.email_notification
                    .send_partial_liquidation_initiated_notification_in_op(
                        &mut op,
                        credit_facility_id,
                        customer_id,
                        trigger_price,
                        initially_estimated_to_liquidate,
                        initially_expected_to_receive,
                    )
                    .await?;
            }
            EmailNotificationConfig::UnderMarginCallThreshold {
                credit_facility_id,
                customer_id,
                effective,
                collateral,
                outstanding_disbursed,
                outstanding_interest,
                price,
            } => {
                self.email_notification
                    .send_under_margin_call_notification_in_op(
                        &mut op,
                        credit_facility_id,
                        customer_id,
                        effective,
                        collateral,
                        outstanding_disbursed,
                        outstanding_interest,
                        price,
                    )
                    .await?;
            }
            EmailNotificationConfig::DepositAccountCreated {
                account_id,
                account_holder_id,
            } => {
                self.email_notification
                    .send_deposit_account_created_notification_in_op(
                        &mut op,
                        account_id,
                        account_holder_id,
                    )
                    .await?;
            }
            EmailNotificationConfig::RoleCreated { role_id, role_name } => {
                self.email_notification
                    .send_role_created_notification_in_op(&mut op, role_id, role_name)
                    .await?;
            }
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
