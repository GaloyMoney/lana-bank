use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{primitives::LedgerAccountSetId, ChartId, EncodedPath};

use super::{ChartEvent, Segmentation};

#[derive(Debug)]
pub struct ChartTreeAlt {
    pub id: ChartId,
    pub name: String,
    pub children: Vec<Rc<RefCell<Node>>>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: LedgerAccountSetId,
    pub reference: String,
    pub encoded_path: EncodedPath,
    pub parent: Option<EncodedPath>,
    pub children: Vec<Rc<RefCell<Node>>>,
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

    let mut chart_children: Vec<Rc<RefCell<Node>>> = vec![];
    let mut entries_by_path: HashMap<EncodedPath, Rc<RefCell<Node>>> = HashMap::new();
    for entry in entries_as_leaves {
        let node_rc = Rc::new(RefCell::new(Node {
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
            if let Some(parent) = entries_by_path.get_mut(&parent_path) {
                dbg!(&parent.borrow().encoded_path);
                parent.borrow_mut().children.push(Rc::clone(&node_rc))
            }
        } else {
            chart_children.push(Rc::clone(&node_rc));
        }

        entries_by_path
            .entry(entry.encoded_path.clone())
            .or_insert_with(|| Rc::clone(&node_rc));
    }

    ChartTreeAlt {
        id: id.expect(""),
        name: name.expect(""),
        children: chart_children,
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
        }

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
        dbg!(chart.chart_alt());
        // {
        //     let control_account = chart
        //         .create_control_account(
        //             LedgerAccountSetId::new(),
        //             ChartCategory::Equity,
        //             "Shareholder Equity".to_string(),
        //             "shareholder-equity".to_string(),
        //             dummy_audit_info(),
        //         )
        //         .unwrap();
        //     chart
        //         .create_control_sub_account(
        //             LedgerAccountSetId::new(),
        //             control_account.path,
        //             "Shareholder Equity".to_string(),
        //             "sub-shareholder-equity".to_string(),
        //             dummy_audit_info(),
        //         )
        //         .unwrap();
        // }
        // assert_eq!(
        //     chart.chart().equity.children[0].children[0].encoded_path,
        //     "30101"
        // );

        // {
        //     chart
        //         .create_control_account(
        //             LedgerAccountSetId::new(),
        //             ChartCategory::Revenues,
        //             "Interest Revenue".to_string(),
        //             "interest-revenue".to_string(),
        //             dummy_audit_info(),
        //         )
        //         .unwrap();
        // }
        // assert_eq!(chart.chart().revenues.children[0].encoded_path, "40100");
        // assert_eq!(chart.chart().revenues.children[0].children.len(), 0);

        // assert_eq!(chart.chart().expenses.children.len(), 0);
    }
}
