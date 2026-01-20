pub mod csv;

#[cfg(test)]
mod tests {
    use crate::{
        chart_of_accounts::entity::{Chart, ChartEvent},
        primitives::{CalaAccountSetId, CalaJournalId, ChartId, DebitOrCredit},
    };
    use es_entity::{EntityEvents, TryFromEvents};

    use crate::primitives::AccountSpec;

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
        let result = chart.import_accounts(specs, journal_id);
        assert_eq!(result.new_account_sets.len(), 2);
        assert_eq!(result.new_account_set_ids.len(), 2);
        assert_eq!(result.new_connections.len(), 2);
    }

    #[test]
    fn can_import_multiple_accounts_with_children() {
        let mut chart = chart_from(initial_events());
        let journal_id = CalaJournalId::new();

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
        let result = chart.import_accounts(specs, journal_id);
        assert_eq!(result.new_account_sets.len(), 5);
        assert_eq!(result.new_account_set_ids.len(), 5);
        assert_eq!(result.new_connections.len(), 5);
    }
}
