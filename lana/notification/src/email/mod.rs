pub mod config;
pub mod error;
pub mod job;
pub mod templates;

use ::job::Jobs;
use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use domain_config::ExposedDomainConfigsReadOnly;
use job::{EmailEventListenerHandler, EmailSenderInit, EmailSenderJobSpawner};
use lana_events::LanaEvent;
use smtp_client::SmtpClient;

use job::deposit_account_created_email::DepositAccountCreatedEmailInitializer;
use job::obligation_overdue_email::ObligationOverdueEmailInitializer;
use job::partial_liquidation_email::PartialLiquidationEmailInitializer;
use job::role_created_email::RoleCreatedEmailInitializer;
use job::under_margin_call_email::UnderMarginCallEmailInitializer;

pub use config::{EmailInfraConfig, NotificationFromEmail, NotificationFromName};
pub use error::EmailError;

pub async fn init<Perms>(
    jobs: &mut Jobs,
    domain_configs: &ExposedDomainConfigsReadOnly,
    infra_config: EmailInfraConfig,
    users: &Users<<Perms as authz::PermissionCheck>::Audit, LanaEvent>,
    credit: &CoreCredit<Perms, LanaEvent>,
    customers: &Customers<Perms, LanaEvent>,
) -> Result<EmailEventListenerHandler, EmailError>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    let template = templates::EmailTemplate::try_new(infra_config.admin_panel_url.clone())?;
    let smtp_client = SmtpClient::try_new(infra_config.to_smtp_config())?;

    let email_sender_job_spawner: EmailSenderJobSpawner = jobs.add_initializer(
        EmailSenderInit::new(smtp_client, template, domain_configs.clone()),
    );

    let obligation_overdue = jobs.add_initializer(ObligationOverdueEmailInitializer::<Perms>::new(
        credit,
        customers,
        users,
        email_sender_job_spawner.clone(),
    ));

    let partial_liquidation =
        jobs.add_initializer(PartialLiquidationEmailInitializer::<Perms>::new(
            customers,
            users,
            email_sender_job_spawner.clone(),
        ));

    let under_margin_call = jobs.add_initializer(UnderMarginCallEmailInitializer::<Perms>::new(
        customers,
        email_sender_job_spawner.clone(),
    ));

    let deposit_account_created =
        jobs.add_initializer(DepositAccountCreatedEmailInitializer::<Perms>::new(
            customers,
            email_sender_job_spawner.clone(),
        ));

    let role_created = jobs.add_initializer(RoleCreatedEmailInitializer::<Perms>::new(
        users,
        email_sender_job_spawner,
    ));

    Ok(EmailEventListenerHandler::new(
        obligation_overdue,
        partial_liquidation,
        under_margin_call,
        deposit_account_created,
        role_created,
    ))
}
