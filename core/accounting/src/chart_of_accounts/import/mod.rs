pub mod csv;

use super::entity::{Chart, NewChartAccountDetails};
use crate::{
    chart_node::*,
    primitives::{AccountSpec, CalaAccountSetId, CalaJournalId, ChartNodeId},
};

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
        let mut new_chart_nodes = Vec::new();

        for spec in account_specs {
            let chart_node = NewChartNode::builder()
                .id(ChartNodeId::new())
                .chart_id(self.chart.id)
                .spec(spec)
                .ledger_account_set_id(CalaAccountSetId::new())
                .build()
                .expect("could not build NewChartNode");
            new_account_sets.push(chart_node.new_account_set(self.journal_id));
            new_account_set_ids.push(chart_node.ledger_account_set_id);
            new_chart_nodes.push(chart_node);
        }

        self.chart.add_all_new_nodes(new_chart_nodes); // check idempotent?

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
    fn can_import_one_account() {
        let mut chart = chart_from(initial_events());
        let journal_id = CalaJournalId::new();
        let import = BulkAccountImport::new(&mut chart, journal_id);
        let specs = vec![AccountSpec {
            name: "Assets".parse().unwrap(),
            parent: None,
            code: "1".parse().unwrap(),
            normal_balance_type: DebitOrCredit::Credit,
        }];
        let result = import.import(specs);
        assert_eq!(chart.n_unpersisted_nodes(), 1);
        assert_eq!(result.new_account_sets.len(), 1);
        assert_eq!(result.new_account_set_ids.len(), 1);
        assert!(result.new_connections.is_empty());
    }

    #[test]
    fn can_import_two_account() {
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
        assert_eq!(chart.n_unpersisted_nodes(), 2);
        assert_eq!(result.new_account_sets.len(), 2);
        assert_eq!(result.new_account_set_ids.len(), 2);
        assert!(result.new_connections.is_empty());
    }

    #[test]
    fn can_import_account_with_child() {
        assert!(true);
    }

    #[test]
    fn can_import_account_with_many_children() {
        assert!(true);
    }
}
