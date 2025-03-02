use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{primitives::LedgerAccountSetId, ChartId, EncodedPath};

use super::{ChartEvent, Segmentation};

#[derive(Debug)]
pub struct ChartTreeAlt {
    pub id: ChartId,
    pub name: String,
    pub children: Vec<ChartEntry>,
}

#[derive(Debug, Clone)]
pub enum ChartEntry {
    Node {
        id: LedgerAccountSetId,
        reference: String,
        encoded_path: EncodedPath,
        parent: Option<EncodedPath>,
        children: Vec<ChartEntry>,
    },
    Leaf {
        id: LedgerAccountSetId,
        reference: String,
        encoded_path: EncodedPath,
        parent: Option<EncodedPath>,
    },
}

#[derive(Debug, Clone)]
pub struct NodeWithRef {
    pub id: LedgerAccountSetId,
    pub reference: String,
    pub encoded_path: EncodedPath,
    pub parent: Option<EncodedPath>,
    pub children: Vec<Rc<RefCell<NodeWithRef>>>,
}

impl NodeWithRef {
    fn into_chart_entry(self) -> ChartEntry {
        if self.children.is_empty() {
            ChartEntry::Leaf {
                id: self.id,
                reference: self.reference,
                encoded_path: self.encoded_path,
                parent: self.parent,
            }
        } else {
            ChartEntry::Node {
                id: self.id,
                reference: self.reference,
                encoded_path: self.encoded_path,
                parent: self.parent,
                children: self
                    .children
                    .into_iter()
                    .map(|child_rc| {
                        let child = Rc::try_unwrap(child_rc)
                            .expect("Child has multiple owners")
                            .into_inner();
                        child.into_chart_entry()
                    })
                    .collect(),
            }
        }
    }
}

#[derive(Debug, Clone)]
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

    let mut chart_children: Vec<Rc<RefCell<NodeWithRef>>> = vec![];
    let mut entries_by_path: HashMap<EncodedPath, Weak<RefCell<NodeWithRef>>> = HashMap::new();
    for entry in entries_as_leaves {
        let node_rc = Rc::new(RefCell::new(NodeWithRef {
            id: entry.id,
            reference: entry.reference.to_string(),
            encoded_path: entry.encoded_path.clone(),
            parent: None,
            children: vec![],
        }));

        if let Some(parent_path) = segmentation
            .parent(entry.encoded_path.clone())
            .expect("Path length was not validated")
        {
            node_rc.borrow_mut().parent = Some(parent_path.clone());

            // FIXME: add parent validation in create_node
            let expect_get_mut =
                &format!("Parent missing in entries_by_path for path {}", parent_path);
            let expect_upgrade = &format!("Parent node for path {} was dropped", parent_path);
            let parent = entries_by_path
                .get_mut(&parent_path)
                .expect(expect_get_mut)
                .upgrade()
                .expect(expect_upgrade);
            parent.borrow_mut().children.push(Rc::clone(&node_rc));
        } else {
            chart_children.push(Rc::clone(&node_rc));
        }

        entries_by_path
            .entry(entry.encoded_path.clone())
            .or_insert_with(|| Rc::downgrade(&node_rc));
    }

    ChartTreeAlt {
        id: id.expect(""),
        name: name.expect(""),
        children: chart_children
            .into_iter()
            .map(|child_rc| {
                let child_refcell = Rc::try_unwrap(child_rc)
                    .expect("Child has multiple owners")
                    .into_inner();
                child_refcell.into_chart_entry()
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use es_entity::*;

    use crate::{Chart, LedgerAccountSetId, NewChart};

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
            .segmentation(Segmentation::new(vec![1, 1, 1, 1]))
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
                    LedgerAccountSetId::new(),
                    "assets".to_string(),
                    "1".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "loans-receivable".to_string(),
                    "10".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "fixed-loans-receivable".to_string(),
                    "100".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "fixed-loans-receivable-01".to_string(),
                    "1000".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "fixed-loans-receivable-02".to_string(),
                    "1001".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        let tree = chart.chart_alt();
        let level_0 = &tree.children[0];
        let level_1 = match level_0 {
            ChartEntry::Node {
                encoded_path,
                children,
                ..
            } => {
                assert_eq!(*encoded_path, "1".parse().unwrap());
                &children[0]
            }
            ChartEntry::Leaf { .. } => panic!(""),
        };
        let level_2 = match level_1 {
            ChartEntry::Node {
                encoded_path,
                children,
                ..
            } => {
                assert_eq!(*encoded_path, "10".parse().unwrap());
                &children[0]
            }
            ChartEntry::Leaf { .. } => panic!(""),
        };
        let (level_3a, level_3b) = match level_2 {
            ChartEntry::Node {
                encoded_path,
                children,
                ..
            } => {
                assert_eq!(*encoded_path, "100".parse().unwrap());
                (&children[0], &children[1])
            }
            ChartEntry::Leaf { .. } => panic!(""),
        };
        match level_3a {
            ChartEntry::Leaf { encoded_path, .. } => {
                assert_eq!(*encoded_path, "1000".parse().unwrap());
            }
            ChartEntry::Node { .. } => panic!(""),
        };
        match level_3b {
            ChartEntry::Leaf { encoded_path, .. } => {
                assert_eq!(*encoded_path, "1001".parse().unwrap());
            }
            ChartEntry::Node { .. } => panic!(""),
        };

        {
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "liabilities".to_string(),
                    "2".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "user-checking".to_string(),
                    "20".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();

            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "sub-user-checking".to_string(),
                    "200".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_node(
                    LedgerAccountSetId::new(),
                    "sub-user-checking-01".to_string(),
                    "2000".parse().unwrap(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        let tree = chart.chart_alt();
        let level_0 = &tree.children[1];
        let level_1 = match level_0 {
            ChartEntry::Node {
                encoded_path,
                children,
                ..
            } => {
                assert_eq!(*encoded_path, "2".parse().unwrap());
                &children[0]
            }
            ChartEntry::Leaf { .. } => panic!(""),
        };
        let level_2 = match level_1 {
            ChartEntry::Node {
                encoded_path,
                children,
                ..
            } => {
                assert_eq!(*encoded_path, "20".parse().unwrap());
                &children[0]
            }
            ChartEntry::Leaf { .. } => panic!(""),
        };
        let level_3 = match level_2 {
            ChartEntry::Node {
                encoded_path,
                children,
                ..
            } => {
                assert_eq!(*encoded_path, "200".parse().unwrap());
                &children[0]
            }
            ChartEntry::Leaf { .. } => panic!(""),
        };
        match level_3 {
            ChartEntry::Leaf { encoded_path, .. } => {
                assert_eq!(*encoded_path, "2000".parse().unwrap());
            }
            ChartEntry::Node { .. } => panic!(""),
        };
        dbg!(tree);
    }
}
