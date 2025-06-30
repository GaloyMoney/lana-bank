//! Test utilities for audit functionality

use crate::{AuditEntryId, AuditInfo};

/// Creates a dummy AuditInfo for testing purposes
pub fn dummy_audit_info() -> AuditInfo {
    AuditInfo {
        audit_entry_id: AuditEntryId::from(1),
        sub: "test_subject".to_string(),
    }
}