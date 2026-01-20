mod event;
mod role;
mod user;

pub use event::*;
pub use role::*;
pub use user::*;

use audit::AuditInfo;

fn extract_sub(context: &Option<es_entity::ContextData>) -> String {
    AuditInfo::from_context(context)
        .map(|info| info.sub)
        .unwrap_or_else(|| "unknown".to_string())
}
