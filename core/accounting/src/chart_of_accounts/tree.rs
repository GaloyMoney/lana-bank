use std::collections::HashMap;

use super::chart_node::ChartNode;
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

impl TreeNode {
    // returns the ids of all the descendants of the node
    pub fn descendants(&self) -> Vec<CalaAccountSetId> {
        let mut result = Vec::new();
        let mut stack: Vec<&TreeNode> = self.children.iter().rev().collect();

        while let Some(node) = stack.pop() {
            result.push(node.id);
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
        result
    }
}

struct TempNode {
    id: CalaAccountSetId,
    code: AccountCode,
    name: AccountName,
    parent: Option<AccountCode>,
    children_indices: Vec<usize>,
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
    let mut temp_nodes: Vec<TempNode> = Vec::new();
    let mut code_to_index: HashMap<AccountCode, usize> = HashMap::new();
    let mut root_indices: Vec<usize> = Vec::new();

    let mut entity_nodes: Vec<EntityNode> = nodes
        .map(|node| EntityNode {
            id: node.account_set_id,
            spec: node.spec.clone(),
        })
        .collect();

    entity_nodes.sort_by_key(|node| node.spec.code.clone());

    for node in entity_nodes {
        let index = temp_nodes.len();
        let temp_node = TempNode {
            id: node.id,
            code: node.spec.code.clone(),
            name: node.spec.name.clone(),
            parent: node.spec.parent.clone(),
            children_indices: vec![],
        };

        if let Some(parent_code) = &node.spec.parent {
            let parent_idx = code_to_index
                .get(parent_code)
                .expect("Parent missing in code_to_index");
            temp_nodes[*parent_idx].children_indices.push(index);
        } else {
            root_indices.push(index);
        }

        code_to_index.insert(node.spec.code.clone(), index);
        temp_nodes.push(temp_node);
    }

    fn build_tree_node(nodes: &[TempNode], index: usize) -> TreeNode {
        let node = &nodes[index];
        TreeNode {
            id: node.id,
            code: node.code.clone(),
            name: node.name.clone(),
            parent: node.parent.clone(),
            children: node
                .children_indices
                .iter()
                .map(|&child_idx| build_tree_node(nodes, child_idx))
                .collect(),
        }
    }

    ChartTree {
        id: chart_id,
        name: chart_name.to_string(),
        children: root_indices
            .iter()
            .map(|&idx| build_tree_node(&temp_nodes, idx))
            .collect(),
    }
}
