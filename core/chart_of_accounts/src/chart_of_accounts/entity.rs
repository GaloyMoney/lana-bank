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
    pub(super) events: EntityEvents<ChartOfAccountsEvent>,
}

impl ChartOfAccounts {
    pub fn add_control_account(
        &self,
        category: ChartOfAccountsCategory,
        account_set_id: LedgerAccountSetId,
    ) {
        unimplemented!()
    }

    pub fn add_control_sub_account(&self, control_account_code: ChartOfAccountsCategory) {
        unimplemented!()
    }
}
