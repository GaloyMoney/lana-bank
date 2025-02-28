use std::collections::HashMap;

use crate::{primitives::LedgerAccountSetId, ChartId, EncodedPath};

use super::{ChartEvent, Segmentation};

#[derive(Debug)]
pub struct ChartTreeAlt {
    pub id: ChartId,
    pub name: String,
    pub children: Vec<Node>, // TODO: use ChartEntry instead?
}

#[derive(Debug)]
pub enum ChartEntry {
    Node(Node),
    Leaf(Leaf),
}

#[derive(Debug)]
pub struct Node {
    pub id: LedgerAccountSetId,
    pub reference: String,
    pub encoded_path: EncodedPath,
    pub children: Vec<ChartEntry>,
}

#[derive(Debug)]
pub struct Leaf {
    pub id: LedgerAccountSetId,
    pub reference: String,
    pub encoded_path: EncodedPath,
}

pub(super) fn project<'a>(events: impl DoubleEndedIterator<Item = &'a ChartEvent>) -> ChartTreeAlt {
    let mut id: Option<ChartId> = None;
    let mut name: Option<String> = None;
    let mut segmentation: Option<Segmentation> = None;
    let mut entries_as_leaves: Vec<Leaf> = vec![];

    for event in events {
        match event {
            ChartEvent::Initialized {
                id: chart_id,
                name: chart_name,
                segmentation: chart_segmentation,
                ..
            } => {
                id = Some(*chart_id);
                name = Some(chart_name.to_string());
                segmentation = Some(chart_segmentation.clone());
            }
            ChartEvent::NodeAdded {
                id,
                reference,
                encoded_path,
                ..
            } => entries_as_leaves.push(Leaf {
                id: *id,
                reference: reference.to_string(),
                encoded_path: encoded_path.clone(),
            }),
            ChartEvent::ControlAccountAdded { .. } => (),
            ChartEvent::ControlSubAccountAdded { .. } => (),
            ChartEvent::Updated { .. } => (),
        }
    }

    let segmentation = segmentation.expect("Expected segmentation not found");
    entries_as_leaves.sort_by_key(|l| l.encoded_path.clone());

    let mut entries_by_path: HashMap<EncodedPath, Node> = HashMap::new();
    for entry in entries_as_leaves {
        let entry_as_node = Node {
            id: entry.id,
            reference: entry.reference,
            encoded_path: entry.encoded_path.clone(),
            children: vec![],
        };
        match segmentation
            .parent(entry.encoded_path.clone())
            .expect("Path length was not validated")
        {
            Some(parent_path) => {
                if let Some(parent) = entries_by_path.get_mut(&parent_path) {
                    parent.children.push(ChartEntry::Node(entry_as_node))
                }
            }
            None => {
                entries_by_path
                    .entry(entry.encoded_path)
                    .or_insert_with(|| entry_as_node);
                ()
            }
        }
    }

    ChartTreeAlt {
        id: id.expect(""),
        name: name.expect(""),
        children: entries_by_path
            .into_iter()
            .map(|(_key, node)| node)
            .collect(),
    }
}
