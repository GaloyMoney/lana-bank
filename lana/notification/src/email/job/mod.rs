pub mod deposit_account_created_email;
mod event_listener;
pub mod obligation_overdue_email;
pub mod partial_liquidation_email;
pub mod role_created_email;
pub mod sender;
pub mod under_margin_call_email;

pub use event_listener::*;
pub use sender::*;

use audit::AuditSvc;
use core_access::user::Users;
use obix::out::OutboxEventMarker;

use crate::email::templates::EmailType;

pub(crate) async fn spawn_email_to_all_users<Audit, E>(
    users: &Users<Audit, E>,
    email_sender_job_spawner: &sender::EmailSenderJobSpawner,
    op: &mut impl es_entity::AtomicOperation,
    email_type: EmailType,
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
            let email_config = sender::EmailSenderConfig {
                recipient: user.email,
                email_type: email_type.clone(),
            };
            email_sender_job_spawner
                .spawn_in_op(op, ::job::JobId::new(), email_config)
                .await?;
        }
    }
    Ok(())
}
