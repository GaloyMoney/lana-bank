use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use cala_ledger::{account::NewAccount, account_set::NewAccountSet};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use std::collections::HashMap;

use super::{error::*, tree};

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
        manual_transaction_account_id: Option<LedgerAccountId>,
    },
    ManualTransactionAccountAdded {
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
    pub manual_transaction_account_id: Option<LedgerAccountId>,

    events: EntityEvents<ChartNodeEvent>,
}

impl ChartNode {
    pub fn add_manual_transaction_account(&mut self) -> Idempotent<(CalaAccountSetId, NewAccount)> {
        if self.manual_transaction_account_id.is_some() {
            return Idempotent::Ignored;
        }

        let ledger_account_id = LedgerAccountId::new();

        self.events
            .push(ChartNodeEvent::ManualTransactionAccountAdded { ledger_account_id });

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
                    manual_transaction_account_id,
                } => {
                    builder = builder
                        .id(*id)
                        .chart_id(*chart_id)
                        .spec(spec.clone())
                        .account_set_id(*ledger_account_set_id)
                        .manual_transaction_account_id(*manual_transaction_account_id);
                }
                ChartNodeEvent::ManualTransactionAccountAdded { ledger_account_id } => {
                    builder = builder.manual_transaction_account_id(Some(*ledger_account_id));
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
                manual_transaction_account_id: self.manual_transaction_account_id,
            }],
        )
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartEvent {
    Initialized {
        id: ChartId,
        name: String,
        reference: String,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
    pub name: String,

    all_accounts: HashMap<AccountCode, ChartNodeId>,
    manual_transaction_accounts: HashMap<LedgerAccountId, ChartNodeId>,

    events: EntityEvents<ChartEvent>,

    #[es_entity(nested)]
    #[builder(default)]
    chart_nodes: Nested<ChartNode>,
}

impl Chart {
    pub(super) fn create_node_without_verifying_parent(
        &mut self,
        spec: &AccountSpec,
        journal_id: CalaJournalId,
    ) -> Idempotent<NewChartAccountDetails> {
        if self.all_accounts.contains_key(&spec.code) {
            return Idempotent::Ignored;
        }

        let node_id = ChartNodeId::new();
        let ledger_account_set_id = CalaAccountSetId::new();

        let new_chart_node = NewChartNode {
            id: node_id,
            chart_id: self.id,
            spec: spec.clone(),
            ledger_account_set_id,
            manual_transaction_account_id: None,
        };

        self.chart_nodes.add_new(new_chart_node);

        self.all_accounts.insert(spec.code.clone(), node_id);

        let parent_account_set_id = if let Some(parent) = spec.parent.as_ref() {
            self.get_node_by_code(parent)
                .map(|node| node.account_set_id)
        } else {
            None
        };

        let new_account_set = NewAccountSet::builder()
            .id(ledger_account_set_id)
            .journal_id(journal_id)
            .name(spec.name.to_string())
            .description(spec.name.to_string())
            .external_id(spec.code.account_set_external_id(self.id))
            .normal_balance_type(spec.normal_balance_type)
            .build()
            .expect("Could not build new account set");

        Idempotent::Executed(NewChartAccountDetails {
            parent_account_set_id,
            new_account_set,
        })
    }

    pub(super) fn create_child_node(
        &mut self,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
        journal_id: CalaJournalId,
    ) -> Result<Idempotent<NewChartAccountDetails>, ChartOfAccountsError> {
        let parent_normal_balance_type = self
            .get_node_by_code(&parent_code)
            .map(|node| node.spec.normal_balance_type)
            .ok_or(ChartOfAccountsError::ParentAccountNotFound(
                parent_code.to_string(),
            ))?;

        let spec = AccountSpec::try_new(
            Some(parent_code),
            code.into(),
            name,
            parent_normal_balance_type,
        )?;

        Ok(self.create_node_without_verifying_parent(&spec, journal_id))
    }

    pub(super) fn trial_balance_account_ids_from_new_accounts<'a>(
        &'a mut self,
        new_account_set_ids: &'a [CalaAccountSetId],
    ) -> impl Iterator<Item = CalaAccountSetId> + '_ {
        self.chart_nodes
            .iter_persisted_mut()
            .filter(move |node| {
                node.spec.code.len_sections() == 2
                    && new_account_set_ids.contains(&node.account_set_id)
            })
            .map(move |node| node.account_set_id)
    }

    pub(super) fn trial_balance_account_id_from_new_account(
        &mut self,
        new_account_set_id: CalaAccountSetId,
    ) -> Option<CalaAccountSetId> {
        self.chart_nodes.iter_persisted_mut().find_map(|node| {
            if node.spec.code.len_sections() == 2 && new_account_set_id == node.account_set_id {
                Some(node.account_set_id)
            } else {
                None
            }
        })
    }

    /// Returns ancestors, in this chart of accounts, of an account with `code` (not included).
    pub fn ancestors<T: From<CalaAccountSetId>>(&self, code: &AccountCode) -> Vec<T> {
        let mut result = Vec::new();
        let mut current_code = code;

        if let Some(node) = self.get_node_by_code(current_code) {
            current_code = match &node.spec.parent {
                Some(parent_code) => parent_code,
                None => return result,
            };
        } else {
            return result;
        }

        while let Some(node) = self.get_node_by_code(current_code) {
            result.push(T::from(node.account_set_id));
            match &node.spec.parent {
                Some(parent_code) => current_code = parent_code,
                None => break,
            }
        }

        result
    }

    /// Returns direct children, in this chart of accounts, of an account with `code` (not included).
    pub fn children(
        &self,
        code: &AccountCode,
    ) -> impl Iterator<Item = (&AccountCode, CalaAccountSetId)> {
        self.chart_nodes
            .iter_persisted_mut()
            .filter_map(move |node| {
                if node.spec.parent.as_ref() == Some(code) {
                    Some((&node.spec.code, node.account_set_id))
                } else {
                    None
                }
            })
    }

    fn get_node_by_code(&self, code: &AccountCode) -> Option<&ChartNode> {
        self.all_accounts
            .get(code)
            .and_then(|node_id| self.chart_nodes.get_persisted(node_id))
    }

    fn get_node_by_code_mut(&mut self, code: &AccountCode) -> Option<&mut ChartNode> {
        let node_id = self.all_accounts.get(code)?;
        self.chart_nodes.get_persisted_mut(node_id)
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<CalaAccountSetId, ChartOfAccountsError> {
        self.get_node_by_code(code)
            .map(|node| node.account_set_id)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))
    }

    pub fn check_can_have_manual_transactions(
        &self,
        code: &AccountCode,
    ) -> Result<(), ChartOfAccountsError> {
        match self.children(code).next() {
            None => Ok(()),
            _ => Err(ChartOfAccountsError::NonLeafAccount(code.to_string())),
        }
    }

    pub fn manual_transaction_account(
        &mut self,
        account_id_or_code: AccountIdOrCode,
    ) -> Result<ManualAccountFromChart, ChartOfAccountsError> {
        match account_id_or_code {
            AccountIdOrCode::Id(id) => Ok(match self.manual_transaction_accounts.get(&id) {
                Some(node_id) => {
                    let node = self
                        .chart_nodes
                        .get_persisted(node_id)
                        .expect("Node ID in index should exist");
                    self.check_can_have_manual_transactions(&node.spec.code)?;
                    ManualAccountFromChart::IdInChart(id)
                }
                None => ManualAccountFromChart::NonChartId(id),
            }),
            AccountIdOrCode::Code(code) => {
                self.check_can_have_manual_transactions(&code)?;

                let node = self
                    .get_node_by_code(&code)
                    .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))?;

                if let Some(existing_id) = node.manual_transaction_account_id {
                    return Ok(ManualAccountFromChart::IdInChart(existing_id));
                }

                let node_id = *self.all_accounts.get(&code).unwrap();
                let node_mut = self.chart_nodes.get_persisted_mut(&node_id).unwrap();

                match node_mut.add_manual_transaction_account() {
                    Idempotent::Executed((account_set_id, new_account)) => {
                        self.manual_transaction_accounts
                            .insert(new_account.id.into(), node_id);

                        Ok(ManualAccountFromChart::NewAccount((
                            account_set_id,
                            new_account,
                        )))
                    }
                    Idempotent::Ignored => {
                        let node = self.chart_nodes.get_persisted(&node_id).unwrap();
                        Ok(ManualAccountFromChart::IdInChart(
                            node.manual_transaction_account_id.unwrap(),
                        ))
                    }
                }
            }
        }
    }

    pub fn chart(&self) -> tree::ChartTree {
        tree::project_from_nodes(self.id, &self.name, self.chart_nodes.iter_persisted_mut())
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();

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
                        .name(name.to_string());
                }
            }
        }

        builder
            .all_accounts(HashMap::new())
            .manual_transaction_accounts(HashMap::new())
            .events(events)
            .build()
    }
}

// impl es_entity::Parent<ChartNode> for Chart {
//     fn new_children_mut(&mut self) -> &mut Vec<<ChartNode as EsEntity>::New> {
//         self.chart_nodes.new_entities_mut()
//     }

//     fn iter_persisted_children_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut ChartNode>
//     where
//         ChartNode: 'a,
//     {
//         self.chart_nodes.iter_persisted_mut()
//     }

//     fn inject_children(&mut self, children: impl IntoIterator<Item = ChartNode>) {
//
//         let children_vec = children.into_iter().collect();
//         for child_node in &children_vec {
//             if let Some(manual_id) = child_node.manual_transaction_account_id {
//                 self.manual_transaction_accounts.insert(manual_id, child_node.id);
//             }
//             self.all_accounts.insert(child_node.spec.code.clone(), child_node.id);

//         }
//         self.chart_nodes.load(children_vec);
//     }
// }

#[derive(Debug, Builder)]
pub struct NewChart {
    #[builder(setter(into))]
    pub(super) id: ChartId,
    pub(super) name: String,
    pub(super) reference: String,
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
            }],
        )
    }
}

#[derive(Debug)]
pub enum ManualAccountFromChart {
    IdInChart(LedgerAccountId),
    NonChartId(LedgerAccountId),
    NewAccount((CalaAccountSetId, NewAccount)),
}

pub struct NewChartAccountDetails {
    pub new_account_set: NewAccountSet,
    pub parent_account_set_id: Option<CalaAccountSetId>,
}
