use crate::primitives::LedgerAccountSetId;

use super::{AccountCode, AccountName, AccountSpec, ChartEvent};

#[derive(Debug, Clone)]
pub struct Node {
    pub id: LedgerAccountSetId,
    pub code: AccountCode,
    pub name: AccountName,
    pub parent: Option<AccountCode>,
}

impl From<EntityNode> for Node {
    fn from(entity_node: EntityNode) -> Self {
        Self {
            id: entity_node.id,
            code: entity_node.spec.code,
            name: entity_node.spec.name,
            parent: entity_node.spec.parent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub id: LedgerAccountSetId,
    pub spec: AccountSpec,
}

pub(super) fn project<'a>(events: impl DoubleEndedIterator<Item = &'a ChartEvent>) -> Vec<Node> {
    let mut entity_nodes: Vec<EntityNode> = vec![];
    for event in events {
        match event {
            ChartEvent::Initialized { .. } => (),
            ChartEvent::NodeAdded {
                ledger_account_set_id: id,
                spec,
                ..
            } => entity_nodes.push(EntityNode {
                id: *id,
                spec: spec.clone(),
            }),
        }
    }
    entity_nodes.into_iter().map(Node::from).collect()
}

#[cfg(test)]
mod tests {
    use es_entity::*;

    use crate::new::{Chart, ChartId, NewChart};

    use super::*;

    use audit::{AuditEntryId, AuditInfo};

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn init_chart_of_events() -> Chart {
        let id = ChartId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChart::builder()
            .id(id)
            .name("Test Chart".to_string())
            .reference("ref-01".to_string())
            .audit_info(audit_info)
            .build()
            .unwrap();

        let events = new_chart.into_events();
        Chart::try_from_events(events).unwrap()
    }

    #[test]
    fn test_project_nodes() {
        let mut chart = init_chart_of_events();

        {
            chart
                .create_node(
                    &AccountSpec {
                        parent: None,
                        code: AccountCode::new(vec!["1".parse().unwrap()]),
                        name: "Assets".parse().unwrap(),
                    },
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    &AccountSpec {
                        parent: Some(AccountCode::new(vec!["1".parse().unwrap()])),
                        code: AccountCode::new(vec!["11".parse().unwrap()]),
                        name: "Assets".parse().unwrap(),
                    },
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    &AccountSpec {
                        parent: Some(AccountCode::new(vec!["11".parse().unwrap()])),
                        code: AccountCode::new(
                            ["11", "01"].iter().map(|c| c.parse().unwrap()).collect(),
                        ),
                        name: "Cash".parse().unwrap(),
                    },
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    &AccountSpec {
                        parent: Some(AccountCode::new(
                            ["11", "01"].iter().map(|c| c.parse().unwrap()).collect(),
                        )),
                        code: AccountCode::new(
                            ["11", "01", "0101"]
                                .iter()
                                .map(|c| c.parse().unwrap())
                                .collect(),
                        ),
                        name: "Central Office".parse().unwrap(),
                    },
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(chart.nodes().len(), 4);
    }
}
