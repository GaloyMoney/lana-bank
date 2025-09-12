use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::chart_node::ChartNode;
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
        if let Some(parent) = node.spec.parent
            && let Some(parent_weak) = tree_nodes_by_code.get_mut(&parent)
            && let Some(parent_rc) = parent_weak.upgrade()
        {
            parent_rc.borrow_mut().children.push(Rc::clone(&node_rc));
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
