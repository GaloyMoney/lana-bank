use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use crate::chart_of_accounts::error::ChartOfAccountsError;
use cala_ledger::account::NewAccount;
//fix errors
#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartNodeId")]
pub enum ChartNodeEvent {
    Initialized {
        id: ChartNodeId,
        chart_id: ChartId,
        spec: AccountSpec,
        ledger_account_set_id: CalaAccountSetId,
    },
    ManualTransactionAccountAssigned {
        ledger_account_id: LedgerAccountId,
    },
    ChildNodeAdded {
        child_node_id: ChartNodeId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ChartNode {
    pub id: ChartNodeId,
    pub chart_id: ChartId,
    pub spec: AccountSpec,
    pub account_set_id: CalaAccountSetId,
    #[builder(setter(strip_option), default)]
    pub manual_transaction_account_id: Option<LedgerAccountId>,

    children: Vec<ChartNodeId>,

    events: EntityEvents<ChartNodeEvent>,
}

impl ChartNode {
    pub fn assign_manual_transaction_account(&mut self) -> Idempotent<NewAccount> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ChartNodeEvent::ManualTransactionAccountAssigned { .. }
        );

        let ledger_account_id = LedgerAccountId::new();

        self.events
            .push(ChartNodeEvent::ManualTransactionAccountAssigned { ledger_account_id });

        self.manual_transaction_account_id = Some(ledger_account_id);

        let new_account = NewAccount::builder()
            .name(format!("{} Manual", self.spec.code))
            .id(ledger_account_id)
            .code(self.spec.code.manual_account_external_id(self.chart_id))
            .external_id(self.spec.code.manual_account_external_id(self.chart_id))
            .build()
            .expect("Could not build new account");

        Idempotent::Executed(new_account)
    }

    pub fn add_child_node(&mut self, child_node_id: ChartNodeId) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ChartNodeEvent::ChildNodeAdded { child_node_id: id, .. } if id == &child_node_id
        );

        self.children.push(child_node_id);
        self.events
            .push(ChartNodeEvent::ChildNodeAdded { child_node_id });
        Idempotent::Executed(())
    }

    pub fn get_children(&self) -> Vec<ChartNodeId> {
        self.children.clone()
    }

    pub fn check_can_have_manual_transactions(&self) -> Result<(), ChartOfAccountsError> {
        match self.children.is_empty() {
            true => Ok(()),
            false => Err(ChartOfAccountsError::NonLeafAccount(
                self.spec.code.to_string(),
            )),
        }
    }

    pub fn is_trial_balance_account(&self) -> bool {
        self.spec.code.len_sections() == 2
    }
}

impl TryFromEvents<ChartNodeEvent> for ChartNode {
    fn try_from_events(events: EntityEvents<ChartNodeEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartNodeBuilder::default();
        let mut children = Vec::new();

        for event in events.iter_all() {
            match event {
                ChartNodeEvent::Initialized {
                    id,
                    chart_id,
                    spec,
                    ledger_account_set_id,
                } => {
                    builder = builder
                        .id(*id)
                        .chart_id(*chart_id)
                        .spec(spec.clone())
                        .account_set_id(*ledger_account_set_id);
                }
                ChartNodeEvent::ManualTransactionAccountAssigned { ledger_account_id } => {
                    builder = builder.manual_transaction_account_id(*ledger_account_id);
                }
                ChartNodeEvent::ChildNodeAdded { child_node_id } => {
                    children.push(*child_node_id);
                }
            }
        }

        builder = builder.children(children);
        builder.events(events).build()
    }
}
#[derive(Debug, Clone, Builder)]
pub struct NewChartNode {
    pub id: ChartNodeId,
    pub chart_id: ChartId,
    pub spec: AccountSpec,
    pub ledger_account_set_id: CalaAccountSetId,
    #[builder(setter(strip_option), default)]
    pub children_node_ids: Option<Vec<ChartNodeId>>,
}

impl IntoEvents<ChartNodeEvent> for NewChartNode {
    fn into_events(self) -> EntityEvents<ChartNodeEvent> {
        let mut events = vec![ChartNodeEvent::Initialized {
            id: self.id,
            chart_id: self.chart_id,
            spec: self.spec,
            ledger_account_set_id: self.ledger_account_set_id,
        }];

        if let Some(children_node_ids) = self.children_node_ids {
            for child_node_id in children_node_ids {
                events.push(ChartNodeEvent::ChildNodeAdded {
                    child_node_id: child_node_id,
                });
            }
        }

        EntityEvents::init(self.id, events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(s: &str) -> AccountCodeSection {
        s.parse::<AccountCodeSection>().unwrap()
    }

    fn default_spec() -> AccountSpec {
        AccountSpec::try_new(
            None,
            vec![section("1")],
            "Assets".parse::<AccountName>().unwrap(),
            DebitOrCredit::Debit,
        )
        .unwrap()
    }

    fn new_chart_node() -> NewChartNode {
        NewChartNode {
            id: ChartNodeId::new(),
            chart_id: ChartId::new(),
            spec: default_spec(),
            ledger_account_set_id: CalaAccountSetId::new(),
            children_node_ids: None,
        }
    }

    #[test]
    fn assign_manual_transaction_account_is_idempotent() {
        let new_node = new_chart_node();
        let events = new_node.into_events();
        let mut node = ChartNode::try_from_events(events).unwrap();

        let _ = node.assign_manual_transaction_account();
        assert!(node.manual_transaction_account_id.is_some());

        let result = node.assign_manual_transaction_account();
        matches!(result, Idempotent::Ignored);
    }
}
