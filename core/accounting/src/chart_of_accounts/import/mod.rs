pub mod csv;

use super::entity::{Chart, NewAccountSetWithNodeId};
use crate::primitives::{AccountCode, AccountSpec, CalaAccountSetId, CalaJournalId, ChartNodeId};
use std::collections::HashMap;

pub(super) struct BulkAccountImport<'a> {
    chart: &'a mut Chart,
    journal_id: CalaJournalId,
}

impl<'a> BulkAccountImport<'a> {
    pub fn new(chart: &'a mut Chart, journal_id: CalaJournalId) -> Self {
        Self { chart, journal_id }
    }

    pub(super) fn import(self, account_specs: Vec<AccountSpec>) -> BulkImportResult {
        let mut new_account_sets = Vec::new();
        let mut new_account_set_ids = Vec::new();
        let mut new_connections = Vec::new();

        let mut parent_code_to_children_ids: HashMap<AccountCode, Vec<ChartNodeId>> =
            HashMap::new();
        let mut node_id_to_account_set_id: HashMap<ChartNodeId, CalaAccountSetId> = HashMap::new();

        let mut sorted_specs = account_specs.clone();
        sorted_specs.sort_by_key(|spec| spec.code.clone());

        // Sort specs in reverse order to ensure all children are created before parents
        sorted_specs.reverse();

        for spec in sorted_specs {
            let children_node_ids = parent_code_to_children_ids.get(&spec.code);

            if let es_entity::Idempotent::Executed(NewAccountSetWithNodeId {
                new_account_set,
                node_id,
            }) = self.chart.create_node_with_existing_children(
                &spec,
                self.journal_id,
                children_node_ids.cloned(),
            ) {
                if let Some(children_node_ids) = children_node_ids {
                    for child_node_id in children_node_ids {
                        new_connections.push((
                            new_account_set.id,
                            *node_id_to_account_set_id
                                .get(child_node_id)
                                .expect("Child node should exist"),
                        ));
                    }
                }

                if let Some(parent_code) = spec.parent {
                    parent_code_to_children_ids
                        .entry(parent_code)
                        .or_insert_with(Vec::new)
                        .push(node_id);
                }
                node_id_to_account_set_id.insert(node_id, new_account_set.id);

                new_account_set_ids.push(new_account_set.id);
                new_account_sets.push(new_account_set);
            }
        }

        BulkImportResult {
            new_account_sets,
            new_account_set_ids,
            new_connections,
        }
    }
}

pub(super) struct BulkImportResult {
    pub new_account_sets: Vec<cala_ledger::account_set::NewAccountSet>,
    pub new_account_set_ids: Vec<CalaAccountSetId>,
    pub new_connections: Vec<(CalaAccountSetId, CalaAccountSetId)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chart_of_accounts::entity::ChartEvent,
        primitives::{ChartId, DebitOrCredit},
    };
    use es_entity::{EntityEvents, TryFromEvents};

    fn chart_from(events: Vec<ChartEvent>) -> Chart {
        Chart::try_from_events(EntityEvents::init(ChartId::new(), events)).unwrap()
    }

    fn initial_events() -> Vec<ChartEvent> {
        vec![ChartEvent::Initialized {
            id: ChartId::new(),
            name: "Test Chart".to_string(),
            reference: "test-chart".to_string(),
        }]
    }

    #[test]
    fn can_import_multiple_accounts() {
        let mut chart = chart_from(initial_events());
        let journal_id = CalaJournalId::new();
        let import = BulkAccountImport::new(&mut chart, journal_id);
        let specs = vec![
            AccountSpec {
                name: "Assets".parse().unwrap(),
                parent: None,
                code: "1".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
            AccountSpec {
                name: "Liabilities".parse().unwrap(),
                parent: None,
                code: "2".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
        ];
        let result = import.import(specs);
        assert_eq!(result.new_account_sets.len(), 2);
        assert_eq!(result.new_account_set_ids.len(), 2);
        assert!(result.new_connections.is_empty());
    }

    #[test]
    fn can_import_multiple_accounts_with_children() {
        let mut chart = chart_from(initial_events());
        let journal_id = CalaJournalId::new();
        let import = BulkAccountImport::new(&mut chart, journal_id);

        let specs = vec![
            AccountSpec {
                name: "Assets".parse().unwrap(),
                parent: None,
                code: "1".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
            AccountSpec {
                name: "Current Assets".parse().unwrap(),
                parent: Some("1".parse().unwrap()),
                code: "1.1".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
            AccountSpec {
                name: "Cash".parse().unwrap(),
                parent: Some("1.1".parse().unwrap()),
                code: "1.1.1".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
            AccountSpec {
                name: "Liabilities".parse().unwrap(),
                parent: None,
                code: "2".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
            AccountSpec {
                name: "Current Liabilities".parse().unwrap(),
                parent: Some("2".parse().unwrap()),
                code: "2.1".parse().unwrap(),
                normal_balance_type: DebitOrCredit::Credit,
            },
        ];
        let result = import.import(specs);
        assert_eq!(result.new_account_sets.len(), 5);
        assert_eq!(result.new_account_set_ids.len(), 5);
        assert_eq!(result.new_connections.len(), 3);
    }
}
