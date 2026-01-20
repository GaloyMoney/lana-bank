mod event;
mod role;
mod user;

pub use event::*;
pub use role::*;
pub use user::*;

use audit::AuditInfo;

fn extract_created_by(context: &Option<es_entity::ContextData>) -> String {
    context
        .as_ref()
        .and_then(|ctx| ctx.lookup::<AuditInfo>("audit_info").ok().flatten())
        .map(|info| info.sub)
        .unwrap_or_else(|| "unknown".to_string())
}
