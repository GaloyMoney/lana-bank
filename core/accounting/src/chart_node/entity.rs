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
    pub fn assign_manual_transaction_account(&mut self) -> Idempotent<(CalaAccountSetId, NewAccount)> {
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
