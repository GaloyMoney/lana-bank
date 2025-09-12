use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use cala_ledger::account::NewAccount;

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

    events: EntityEvents<ChartNodeEvent>,
}

impl ChartNode {
    pub fn assign_manual_transaction_account(
        &mut self,
    ) -> Idempotent<(CalaAccountSetId, NewAccount)> {
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

        Idempotent::Executed((self.account_set_id, new_account))
    }
}

impl TryFromEvents<ChartNodeEvent> for ChartNode {
    fn try_from_events(events: EntityEvents<ChartNodeEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartNodeBuilder::default();

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
                        .account_set_id(*ledger_account_set_id)
                }
                ChartNodeEvent::ManualTransactionAccountAssigned { ledger_account_id } => {
                    builder = builder.manual_transaction_account_id(*ledger_account_id);
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Clone, Builder)]
pub struct NewChartNode {
    pub id: ChartNodeId,
    pub chart_id: ChartId,
    pub spec: AccountSpec,
    pub ledger_account_set_id: CalaAccountSetId,
    pub manual_transaction_account_id: Option<LedgerAccountId>,
}

impl IntoEvents<ChartNodeEvent> for NewChartNode {
    fn into_events(self) -> EntityEvents<ChartNodeEvent> {
        EntityEvents::init(
            self.id,
            vec![ChartNodeEvent::Initialized {
                id: self.id,
                chart_id: self.chart_id,
                spec: self.spec,
                ledger_account_set_id: self.ledger_account_set_id,
            }],
        )
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
            manual_transaction_account_id: None,
        }
    }

    #[test]
    fn new_chart_node_produces_initialized_event() {
        let new_node = new_chart_node();
        let events = new_node.clone().into_events();

        let first = events.iter_all().next().unwrap();
        match first {
            ChartNodeEvent::Initialized {
                id,
                chart_id,
                spec,
                ledger_account_set_id,
            } => {
                assert_eq!(id, &new_node.id);
                assert_eq!(chart_id, &new_node.chart_id);
                assert_eq!(spec.code, new_node.spec.code);
                assert_eq!(ledger_account_set_id, &new_node.ledger_account_set_id);
            }
            _ => panic!("Expected Initialized event"),
        }
    }

    #[test]
    fn try_from_events_rebuilds_state() {
        let new_node = new_chart_node();
        let events = new_node.clone().into_events();
        let node = ChartNode::try_from_events(events.clone()).unwrap();

        assert_eq!(node.id, new_node.id);
        assert_eq!(node.chart_id, new_node.chart_id);
        assert_eq!(node.spec.code, new_node.spec.code);
        assert_eq!(node.account_set_id, new_node.ledger_account_set_id);
        assert!(node.manual_transaction_account_id.is_none());
    }

    #[test]
    fn assign_manual_transaction_account_adds_event_and_sets_field() {
        let new_node = new_chart_node();
        let events = new_node.into_events();
        let mut node = ChartNode::try_from_events(events).unwrap();

        let result = node.assign_manual_transaction_account();

        match result {
            Idempotent::Executed((set_id, new_account)) => {
                assert_eq!(set_id, node.account_set_id);
                assert_eq!(
                    new_account.id,
                    node.manual_transaction_account_id.unwrap().into()
                );
            }
            _ => panic!("Expected Executed on first assignment"),
        }

        let last_event = node.events.iter_all().last().unwrap();
        matches!(
            last_event,
            ChartNodeEvent::ManualTransactionAccountAssigned { .. }
        );
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
