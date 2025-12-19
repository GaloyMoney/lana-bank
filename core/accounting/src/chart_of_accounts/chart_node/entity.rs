use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{chart_of_accounts::error::ChartOfAccountsError, primitives::*};

use cala_ledger::{account::NewAccount, account_set::NewAccountSet};

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
    pub fn assign_manual_transaction_account(
        &mut self,
    ) -> Result<Idempotent<NewAccount>, ChartOfAccountsError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ChartNodeEvent::ManualTransactionAccountAssigned { .. }
        );
        if !self.can_have_manual_transactions() {
            return Err(ChartOfAccountsError::NonLeafAccount(
                self.spec.code.to_string(),
            ));
        }

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

        Ok(Idempotent::Executed(new_account))
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

    pub fn children(&self) -> impl Iterator<Item = &ChartNodeId> {
        self.children.iter()
    }

    pub fn can_have_manual_transactions(&self) -> bool {
        self.children.is_empty()
    }

    pub fn is_trial_balance_account(&self) -> bool {
        !self.spec.has_parent()
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
    #[builder(default)]
    pub children_node_ids: Vec<ChartNodeId>,
}

impl NewChartNode {
    pub fn builder() -> NewChartNodeBuilder {
        NewChartNodeBuilder::default()
    }

    pub fn new_account_set(&self, journal_id: CalaJournalId) -> NewAccountSet {
        NewAccountSet::builder()
            .id(self.ledger_account_set_id)
            .journal_id(journal_id)
            .name(self.spec.name.to_string())
            .description(self.spec.name.to_string())
            .external_id(self.spec.code.account_set_external_id(self.chart_id))
            .normal_balance_type(self.spec.normal_balance_type)
            .build()
            .expect("Could not build NewAccountSet")
    }
}

impl IntoEvents<ChartNodeEvent> for NewChartNode {
    fn into_events(self) -> EntityEvents<ChartNodeEvent> {
        let mut events = vec![ChartNodeEvent::Initialized {
            id: self.id,
            chart_id: self.chart_id,
            spec: self.spec,
            ledger_account_set_id: self.ledger_account_set_id,
        }];

        for child_node_id in self.children_node_ids {
            events.push(ChartNodeEvent::ChildNodeAdded { child_node_id });
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

    fn new_chart_node(spec: AccountSpec) -> NewChartNode {
        NewChartNode {
            id: ChartNodeId::new(),
            chart_id: ChartId::new(),
            spec,
            ledger_account_set_id: CalaAccountSetId::new(),
            children_node_ids: Vec::new(),
        }
    }

    #[test]
    fn assign_manual_transaction_account_is_idempotent() {
        let new_node = new_chart_node(
            AccountSpec::try_new(
                None,
                vec![section("1")],
                "Assets".parse::<AccountName>().unwrap(),
                DebitOrCredit::Debit,
            )
            .unwrap(),
        );
        let events = new_node.into_events();
        let mut node = ChartNode::try_from_events(events).unwrap();

        let _ = node.assign_manual_transaction_account();
        assert!(node.manual_transaction_account_id.is_some());

        let result = node.assign_manual_transaction_account();
        matches!(result, Ok(Idempotent::AlreadyApplied));
    }

    #[test]
    fn add_child_node_is_idempotent() {
        let new_node = new_chart_node(
            AccountSpec::try_new(
                None,
                vec![section("1")],
                "Assets".parse::<AccountName>().unwrap(),
                DebitOrCredit::Debit,
            )
            .unwrap(),
        );
        let events = new_node.into_events();
        let mut node = ChartNode::try_from_events(events).unwrap();
        let child_node_id = ChartNodeId::new();
        let _ = node.add_child_node(child_node_id);

        let result = node.add_child_node(child_node_id);
        matches!(result, Idempotent::AlreadyApplied);
    }
}
