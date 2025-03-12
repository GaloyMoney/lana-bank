use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{primitives::LedgerAccountSetId, ChartId};

use crate::{AccountCode, AccountName, AccountSpec, ChartEvent};

#[derive(Debug)]
pub struct ChartTree {
    pub id: ChartId,
    pub name: String,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: LedgerAccountSetId,
    pub code: AccountCode,
    pub name: AccountName,
    pub parent: Option<AccountCode>,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct TreeNodeWithRef {
    id: LedgerAccountSetId,
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
    pub id: LedgerAccountSetId,
    pub spec: AccountSpec,
}

pub(super) fn project<'a>(events: impl DoubleEndedIterator<Item = &'a ChartEvent>) -> ChartTree {
    let mut id: Option<ChartId> = None;
    let mut name: Option<String> = None;
    let mut entity_nodes: Vec<EntityNode> = vec![];

    for event in events {
        match event {
            ChartEvent::Initialized {
                id: chart_id,
                name: chart_name,
                ..
            } => {
                id = Some(*chart_id);
                name = Some(chart_name.to_string());
            }
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

    let mut chart_children: Vec<Rc<RefCell<TreeNodeWithRef>>> = vec![];
    let mut tree_nodes_by_code: HashMap<AccountCode, Weak<RefCell<TreeNodeWithRef>>> =
        HashMap::new();

    entity_nodes.sort_by_key(|l| l.spec.code.clone());
    for node in entity_nodes {
        let node_rc = Rc::new(RefCell::new(TreeNodeWithRef {
            id: node.id,
            code: node.spec.code.clone(),
            name: node.spec.name.clone(),
            parent: node.spec.parent.clone(),
            children: vec![],
        }));
        if let Some(parent) = node.spec.parent {
            tree_nodes_by_code
                .get_mut(&parent)
                .expect("Parent missing in tree_nodes_by_code for code")
                .upgrade()
                .expect("Parent node for code was dropped")
                .borrow_mut()
                .children
                .push(Rc::clone(&node_rc));
        } else {
            chart_children.push(Rc::clone(&node_rc));
        }

        tree_nodes_by_code
            .entry(node.spec.code)
            .or_insert_with(|| Rc::downgrade(&node_rc));
    }

    ChartTree {
        id: id.expect("chart id is missing"),
        name: name.expect("chart name is missing"),
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
    use es_entity::*;

    use crate::chart_of_accounts::{Chart, NewChart};

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
    fn test_project_chart_structure() {
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
        let tree = chart.chart();
        let assets = &tree.children[0];
        assert_eq!(assets.code, AccountCode::new(vec!["1".parse().unwrap()]));
        let assets_2 = &assets.children[0];
        assert_eq!(assets_2.code, AccountCode::new(vec!["11".parse().unwrap()]));
        let cash = &assets_2.children[0];
        assert_eq!(
            cash.code,
            AccountCode::new(["11", "01"].iter().map(|c| c.parse().unwrap()).collect(),)
        );
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
}
