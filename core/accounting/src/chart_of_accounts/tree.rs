use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{entity::ChartEvent, entity::ChartNode};
use crate::primitives::{AccountCode, AccountName, AccountSpec, CalaAccountSetId, ChartId};

#[derive(Debug)]
pub struct ChartTree {
    pub id: ChartId,
    pub name: String,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: CalaAccountSetId,
    pub code: AccountCode,
    pub name: AccountName,
    pub parent: Option<AccountCode>,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct TreeNodeWithRef {
    id: CalaAccountSetId,
    code: AccountCode,
    name: AccountName,
    parent: Option<AccountCode>,
    children: Vec<Rc<RefCell<TreeNodeWithRef>>>,
}

impl TreeNodeWithRef {
    fn into_node(self) -> TreeNode {
        TreeNode {
            id: self.id,
            code: self.code,
            name: self.name,
            parent: self.parent,
            children: self
                .children
                .into_iter()
                .map(|child_rc| {
                    let child = Rc::try_unwrap(child_rc)
                        .expect("Child has multiple owners")
                        .into_inner();
                    child.into_node()
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub id: CalaAccountSetId,
    pub spec: AccountSpec,
}

pub(super) fn project_from_nodes<'a>(
    chart_id: ChartId,
    chart_name: &str,
    nodes: impl Iterator<Item = &'a ChartNode>,
) -> ChartTree {
    let mut chart_children: Vec<Rc<RefCell<TreeNodeWithRef>>> = vec![];
    let mut tree_nodes_by_code: HashMap<AccountCode, Weak<RefCell<TreeNodeWithRef>>> =
        HashMap::new();

    let mut entity_nodes: Vec<EntityNode> = nodes
        .map(|node| EntityNode {
            id: node.account_set_id,
            spec: node.spec.clone(),
        })
        .collect();

    entity_nodes.sort_by_key(|node| node.spec.code.clone());

    for node in entity_nodes {
        let node_rc = Rc::new(RefCell::new(TreeNodeWithRef {
            id: node.id,
            code: node.spec.code.clone(),
            name: node.spec.name.clone(),
            parent: node.spec.parent.clone(),
            children: vec![],
        }));

        if let Some(parent) = node.spec.parent {
            if let Some(parent_weak) = tree_nodes_by_code.get_mut(&parent) {
                if let Some(parent_rc) = parent_weak.upgrade() {
                    parent_rc.borrow_mut().children.push(Rc::clone(&node_rc));
                }
            }
        } else {
            chart_children.push(Rc::clone(&node_rc));
        }

        tree_nodes_by_code
            .entry(node.spec.code)
            .or_insert_with(|| Rc::downgrade(&node_rc));
    }

    ChartTree {
        id: chart_id,
        name: chart_name.to_string(),
        children: chart_children
            .into_iter()
            .map(|child_rc| {
                let child_refcell = Rc::try_unwrap(child_rc)
                    .expect("Child has multiple owners")
                    .into_inner();
                child_refcell.into_node()
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use cala_ledger::DebitOrCredit;
    use es_entity::*;

    use crate::{
        chart_of_accounts::{
            Chart, NewChart,
            chart_node::{ChartNode, NewChartNode},
        },
        primitives::CalaJournalId,
    };

    use super::*;

    fn init_chart_of_events() -> Chart {
        let id = ChartId::new();

        let new_chart = NewChart::builder()
            .id(id)
            .name("Test Chart".to_string())
            .reference("ref-01".to_string())
            .build()
            .unwrap();

        let events = new_chart.into_events();
        Chart::try_from_events(events).unwrap()
    }

    #[test]
    fn test_project_chart_structure() {
        let mut chart = init_chart_of_events();

        chart
            .create_node_without_verifying_parent(
                &AccountSpec {
                    parent: None,
                    code: AccountCode::new(vec!["1".parse().unwrap()]),
                    name: "Assets".parse().unwrap(),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                CalaJournalId::new(),
            )
            .unwrap();

        chart
            .create_node_without_verifying_parent(
                &AccountSpec {
                    parent: Some(AccountCode::new(vec!["1".parse().unwrap()])),
                    code: AccountCode::new(vec!["11".parse().unwrap()]),
                    name: "Current Assets".parse().unwrap(),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                CalaJournalId::new(),
            )
            .unwrap();

        chart
            .create_node_without_verifying_parent(
                &AccountSpec {
                    parent: Some(AccountCode::new(vec!["11".parse().unwrap()])),
                    code: AccountCode::new(
                        ["11", "01"].iter().map(|c| c.parse().unwrap()).collect(),
                    ),
                    name: "Cash".parse().unwrap(),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                CalaJournalId::new(),
            )
            .unwrap();

        chart
            .create_node_without_verifying_parent(
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
                    normal_balance_type: DebitOrCredit::Credit,
                },
                CalaJournalId::new(),
            )
            .unwrap();

        let tree = chart.chart();

        assert_eq!(tree.id, chart.id);
        assert_eq!(tree.name, chart.name);
        assert_eq!(tree.children.len(), 1);

        let assets = &tree.children[0];
        assert_eq!(assets.code, AccountCode::new(vec!["1".parse().unwrap()]));
        assert_eq!(assets.children.len(), 1);

        let current_assets = &assets.children[0];
        assert_eq!(
            current_assets.code,
            AccountCode::new(vec!["11".parse().unwrap()])
        );
        assert_eq!(current_assets.children.len(), 1);

        let cash = &current_assets.children[0];
        assert_eq!(
            cash.code,
            AccountCode::new(["11", "01"].iter().map(|c| c.parse().unwrap()).collect())
        );
        assert_eq!(cash.children.len(), 1);

        let central_office = &cash.children[0];
        assert_eq!(
            central_office.code,
            AccountCode::new(
                ["11", "01", "0101"]
                    .iter()
                    .map(|c| c.parse().unwrap())
                    .collect(),
            )
        );
        assert!(central_office.children.is_empty());
    }

    #[test]
    fn test_project_from_nodes_directly() {
        let chart_id = ChartId::new();

        let nodes = vec![];

        let tree = project_from_nodes(chart_id, "Test Chart", nodes.iter());

        assert_eq!(tree.id, chart_id);
        assert_eq!(tree.name, "Test Chart");
    }
}
