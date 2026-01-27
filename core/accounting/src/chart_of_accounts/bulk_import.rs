use std::collections::HashMap;
use tracing::instrument;

use tracing_macros::record_error_severity;

use super::entity::{Chart, NewAccountSetWithNodeId};
use crate::primitives::{AccountCode, AccountSpec, CalaAccountSetId, CalaJournalId, ChartNodeId};

pub(super) struct BulkAccountImport<'a> {
    chart: &'a mut Chart,
    journal_id: CalaJournalId,
}

impl<'a> BulkAccountImport<'a> {
    pub(super) fn new(chart: &'a mut Chart, journal_id: CalaJournalId) -> Self {
        Self { chart, journal_id }
    }

    #[record_error_severity]
    #[instrument(name = "chart_import.import", skip(self, account_specs), fields(specs_count = account_specs.len()))]
    pub(super) fn import(self, account_specs: Vec<AccountSpec>) -> BulkImportResult {
        let mut new_account_sets = Vec::new();
        let mut new_account_set_ids = Vec::new();
        let mut new_connections = Vec::new();

        let mut parent_code_to_children_ids: HashMap<AccountCode, Vec<ChartNodeId>> =
            HashMap::new();
        let mut node_id_to_account_set_id: HashMap<ChartNodeId, CalaAccountSetId> = HashMap::new();

        let mut sorted_specs = account_specs;
        sorted_specs.sort_by_key(|spec| spec.code.clone());

        // Sort in reverse so that children appear before parents.
        // This relies on the hierarchical structure being encoded in the account codes:
        // e.g., "1", "1.1", "1.1.1". In this scheme "1.1" is a child of "1",
        // and reverse lexical ordering ensures all parents come after their descendants.
        sorted_specs.reverse();

        for spec in sorted_specs {
            let children_node_ids = parent_code_to_children_ids
                .remove(&spec.code)
                .unwrap_or_default();

            if let es_entity::Idempotent::Executed(NewAccountSetWithNodeId {
                new_account_set,
                node_id,
            }) = self.chart.create_node_with_existing_children(
                &spec,
                self.journal_id,
                children_node_ids.clone(),
            ) {
                for child_node_id in children_node_ids {
                    new_connections.push((
                        new_account_set.id,
                        *node_id_to_account_set_id
                            .get(&child_node_id)
                            .expect("Child node should exist"),
                    ));
                }

                if let Some(parent_code) = &spec.parent {
                    if let Some(parent_node) = self
                        .chart
                        .chart_nodes
                        .find_persisted_mut(|n| n.spec.code == *parent_code)
                    {
                        let _ = parent_node.add_child_node(node_id);
                        new_connections.push((parent_node.account_set_id, new_account_set.id));
                    } else {
                        parent_code_to_children_ids
                            .entry(parent_code.clone())
                            .or_default()
                            .push(node_id);
                    }
                } else {
                    new_connections.push((self.chart.account_set_id, new_account_set.id));
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
            account_set_id: CalaAccountSetId::new(),
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
        assert_eq!(result.new_connections.len(), 2);
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
        assert_eq!(result.new_connections.len(), 5);
    }
}
