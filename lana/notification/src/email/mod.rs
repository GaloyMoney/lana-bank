pub mod config;
pub mod error;
pub mod job;
pub mod templates;

use ::job::Jobs;
use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use domain_config::ExposedDomainConfigsReadOnly;
use job::EmailEventListenerHandler;
use lana_events::LanaEvent;
use smtp_client::SmtpClient;

use job::send_deposit_account_created_email::SendDepositAccountCreatedEmailInitializer;
use job::send_obligation_overdue_email::SendObligationOverdueEmailInitializer;
use job::send_partial_liquidation_email::SendPartialLiquidationEmailInitializer;
use job::send_role_created_email::SendRoleCreatedEmailInitializer;
use job::send_under_margin_call_email::SendUnderMarginCallEmailInitializer;

pub use config::{EmailInfraConfig, NotificationFromEmail, NotificationFromName};
pub use error::EmailError;

pub(crate) async fn init<Perms>(
    jobs: &mut Jobs,
    domain_configs: &ExposedDomainConfigsReadOnly,
    infra_config: EmailInfraConfig,
    users: &Users<<Perms as authz::PermissionCheck>::Audit, LanaEvent>,
    credit: &CoreCredit<Perms, LanaEvent>,
    customers: &Customers<Perms, LanaEvent>,
) -> Result<EmailEventListenerHandler<Perms>, EmailError>
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

    let send_obligation_overdue_email =
        jobs.add_initializer(SendObligationOverdueEmailInitializer::<Perms>::new(
            credit,
            customers,
            smtp_client.clone(),
            template.clone(),
            domain_configs.clone(),
        ));

    let send_partial_liquidation_email =
        jobs.add_initializer(SendPartialLiquidationEmailInitializer::new(
            smtp_client.clone(),
            template.clone(),
            domain_configs.clone(),
        ));

    let send_under_margin_call_email =
        jobs.add_initializer(SendUnderMarginCallEmailInitializer::<Perms>::new(
            customers,
            smtp_client.clone(),
            template.clone(),
            domain_configs.clone(),
        ));

    let send_deposit_account_created_email =
        jobs.add_initializer(SendDepositAccountCreatedEmailInitializer::<Perms>::new(
            customers,
            smtp_client.clone(),
            template.clone(),
            domain_configs.clone(),
        ));

    let send_role_created_email = jobs.add_initializer(SendRoleCreatedEmailInitializer::new(
        smtp_client,
        template,
        domain_configs.clone(),
    ));

    Ok(EmailEventListenerHandler::new(
        users.clone(),
        customers.clone(),
        send_obligation_overdue_email,
        send_partial_liquidation_email,
        send_under_margin_call_email,
        send_deposit_account_created_email,
        send_role_created_email,
    ))
}
