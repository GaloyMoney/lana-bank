use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartOfAccountsId")]
pub enum ChartOfAccountsEvent {
    Initialized {
        id: ChartOfAccountsId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ChartOfAccounts {
    assets_account_set_id: LedgerAccountSetId,
    pub(super) events: EntityEvents<ChartOfAccountsEvent>,
}

impl ChartOfAccounts {
    pub fn add_assets_category(&self) {
        unimplemented!()
    }
}
