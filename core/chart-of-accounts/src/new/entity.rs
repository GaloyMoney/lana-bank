use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use audit::AuditInfo;

use es_entity::*;

use super::{error::*, primitives::*, tree};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartEvent {
    Initialized {
        id: ChartId,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
    NodeAdded {
        spec: AccountSpec,
        ledger_account_set_id: LedgerAccountSetId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
    pub name: String,
    all_accounts: HashMap<AccountCode, (AccountSpec, LedgerAccountSetId)>,
    pub(super) events: EntityEvents<ChartEvent>,
}

impl Chart {
    pub fn create_node(
        &mut self,
        spec: &AccountSpec,
        audit_info: AuditInfo,
    ) -> Idempotent<(Option<LedgerAccountSetId>, LedgerAccountSetId)> {
        if self.all_accounts.contains_key(&spec.code) {
            return Idempotent::AlreadyApplied;
        }
        let ledger_account_set_id = LedgerAccountSetId::new();
        self.events.push(ChartEvent::NodeAdded {
            spec: spec.clone(),
            ledger_account_set_id,
            audit_info,
        });
        let parent = if let Some(parent) = spec.parent.as_ref() {
            self.all_accounts.get(parent).map(|(_, id)| *id)
        } else {
            None
        };
        self.all_accounts
            .insert(spec.code.clone(), (spec.clone(), ledger_account_set_id));
        Idempotent::Executed((parent, ledger_account_set_id))
    }

    pub fn all_non_top_level_accounts(
        &self,
    ) -> impl Iterator<Item = &(AccountSpec, LedgerAccountSetId)> {
        self.all_accounts
            .values()
            .filter(|(spec, _)| spec.code.len_sections() != 1)
    }

    pub fn account_spec(&self, code: &AccountCode) -> Option<&(AccountSpec, LedgerAccountSetId)> {
        self.all_accounts.get(code)
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<LedgerAccountSetId, AltChartError> {
        self.account_spec(code)
            .map(|(_, id)| *id)
            .ok_or_else(|| AltChartError::CodeNotFoundInChart(code.clone()))
    }

    pub fn chart(&self) -> tree::ChartTree {
        tree::project(self.events.iter_all())
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();
        let mut all_accounts = HashMap::new();
        for event in events.iter_all() {
            match event {
                ChartEvent::Initialized {
                    id,
                    reference,
                    name,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .reference(reference.to_string())
                        .name(name.to_string())
                }
                ChartEvent::NodeAdded {
                    spec,
                    ledger_account_set_id,
                    ..
                } => {
                    all_accounts.insert(spec.code.clone(), (spec.clone(), *ledger_account_set_id));
                }
            }
        }
        builder.all_accounts(all_accounts).events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewChart {
    #[builder(setter(into))]
    pub(super) id: ChartId,
    pub(super) name: String,
    pub(super) reference: String,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewChart {
    pub fn builder() -> NewChartBuilder {
        NewChartBuilder::default()
    }
}

impl IntoEvents<ChartEvent> for NewChart {
    fn into_events(self) -> EntityEvents<ChartEvent> {
        EntityEvents::init(
            self.id,
            [ChartEvent::Initialized {
                id: self.id,
                name: self.name,
                reference: self.reference,
                audit_info: self.audit_info,
            }],
        )
    }
}
