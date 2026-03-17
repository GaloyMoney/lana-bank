use std::sync::LazyLock;

use chrono::NaiveDate;
use job::JobId;
use uuid::Uuid;

/// Precomputed namespace: uuid_v5(DNS_NAMESPACE, "lana-bank.eod")
static EOD_NAMESPACE: LazyLock<Uuid> =
    LazyLock::new(|| Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"lana-bank.eod"));

pub fn eod_manager_id(date: &NaiveDate) -> JobId {
    let name = format!("eod-{date}");
    JobId::from(Uuid::new_v5(&EOD_NAMESPACE, name.as_bytes()))
}

pub fn eod_child_id(date: &NaiveDate, process_name: &str) -> JobId {
    let name = format!("eod-{date}-{process_name}");
    JobId::from(Uuid::new_v5(&EOD_NAMESPACE, name.as_bytes()))
}

pub fn eod_entity_id(date: &NaiveDate, process_name: &str, entity_id: &Uuid) -> JobId {
    let name = format!("eod-{date}-{process_name}-{entity_id}");
    JobId::from(Uuid::new_v5(&EOD_NAMESPACE, name.as_bytes()))
}
