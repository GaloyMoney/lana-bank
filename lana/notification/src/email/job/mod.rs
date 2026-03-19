mod event_listener;
pub(crate) mod send_deposit_account_created_email;
pub(crate) mod send_obligation_overdue_email;
pub(crate) mod send_partial_liquidation_email;
pub(crate) mod send_role_created_email;
pub(crate) mod send_under_margin_call_email;

pub(crate) use event_listener::*;

use audit::AuditSvc;
use core_access::user::Users;
use domain_config::ExposedDomainConfigsReadOnly;
use obix::out::OutboxEventMarker;
use smtp_client::SmtpClient;

use crate::email::config::{NotificationFromEmail, NotificationFromName};
use crate::email::templates::{EmailTemplate, EmailType};

pub(crate) async fn send_rendered_email(
    smtp_client: &SmtpClient,
    template: &EmailTemplate,
    domain_configs: &ExposedDomainConfigsReadOnly,
    recipient: &str,
    email_type: &EmailType,
) -> Result<(), Box<dyn std::error::Error>> {
    let from_email_config = domain_configs
        .get_without_audit::<NotificationFromEmail>()
        .await?;
    let from_email = match from_email_config.maybe_value() {
        Some(from_email) => from_email,
        None => {
            tracing::warn!("no configured notification from email; skipping email");
            return Ok(());
        }
    };

    let from_name_config = domain_configs
        .get_without_audit::<NotificationFromName>()
        .await?;
    let from_name = match from_name_config.maybe_value() {
        Some(from_name) => from_name,
        None => {
            tracing::warn!("no configured notification from name; skipping email");
            return Ok(());
        }
    };

    let (subject, body) = template.render_email(email_type)?;
    smtp_client
        .send_email(&from_email, Some(&from_name), recipient, &subject, body)
        .await?;
    Ok(())
}

pub(crate) async fn send_email_to_all_users<Audit, E>(
    smtp_client: &SmtpClient,
    template: &EmailTemplate,
    domain_configs: &ExposedDomainConfigsReadOnly,
    users: &Users<Audit, E>,
    email_type: &EmailType,
) -> Result<(), Box<dyn std::error::Error>>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Action: From<core_access::CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<core_access::CoreAccessObject>,
    <Audit as AuditSvc>::Subject: From<core_access::UserId>,
    E: OutboxEventMarker<lana_events::CoreAccessEvent>,
{
    let mut has_next_page = true;
    let mut after = None;
    while has_next_page {
        let es_entity::PaginatedQueryRet {
            entities,
            has_next_page: next_page,
            end_cursor,
        } = users
            .list_users_without_audit(
                es_entity::PaginatedQueryArgs { first: 20, after },
                es_entity::ListDirection::Descending,
            )
            .await?;
        (after, has_next_page) = (end_cursor, next_page);

        for user in entities {
            if let Err(e) = send_rendered_email(
                smtp_client,
                template,
                domain_configs,
                &user.email,
                email_type,
            )
            .await
            {
                tracing::warn!(
                    recipient = %user.email,
                    error = %e,
                    "failed to send email to user; continuing with remaining recipients"
                );
            }
        }
    }
    Ok(())
}
