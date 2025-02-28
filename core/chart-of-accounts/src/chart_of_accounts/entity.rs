use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::{
    path::*,
    primitives::{ChartId, LedgerAccountSetId},
    ControlAccountDetails, ControlSubAccountDetails, EncodedPath, NodeDetails,
};

pub use super::error::*;
use super::tree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segmentation {
    schema: Vec<u8>,
}

impl Segmentation {
    pub fn new(schema: Vec<u8>) -> Self {
        Segmentation { schema }
    }

    fn schema_by_length_from_start_to_segment_end(&self) -> Vec<usize> {
        let mut lengths_from_start_to_segment_end = vec![];
        let mut cumulative = 0;
        for &segment_length in &self.schema {
            cumulative += segment_length as usize;
            lengths_from_start_to_segment_end.push(cumulative);
        }

        lengths_from_start_to_segment_end
    }

    pub fn check_path(&self, path: EncodedPath) -> Result<(), ChartError> {
        let valid_lengths = self.schema_by_length_from_start_to_segment_end();
        if !valid_lengths.contains(&path.len()) {
            return Err(ChartError::InvalidCodeLength(path.len(), valid_lengths));
        }

        Ok(())
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartEvent {
    Initialized {
        id: ChartId,
        name: String,
        reference: String,
        segmentation: Segmentation,
        audit_info: AuditInfo,
    },
    Updated {
        name: String,
        reference: String,
        segmentation: Segmentation,
        audit_info: AuditInfo,
    },
    NodeAdded {
        id: LedgerAccountSetId,
        encoded_path: EncodedPath,
        reference: String,
        audit_info: AuditInfo,
    },
    ControlAccountAdded {
        id: LedgerAccountSetId,
        encoded_path: String,
        path: ControlAccountPath,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
    ControlSubAccountAdded {
        id: LedgerAccountSetId,
        encoded_path: String,
        path: ControlSubAccountPath,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
    pub name: String,
    pub(super) events: EntityEvents<ChartEvent>,
}

impl Chart {
    fn next_control_account(
        &self,
        category: ChartCategory,
    ) -> Result<ControlAccountPath, ChartError> {
        Ok(self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartEvent::ControlAccountAdded { path, .. } if path.category == category => {
                    Some(path.next())
                }
                _ => None,
            })
            .unwrap_or_else(|| Ok(category.first_control_account()))?)
    }

    pub fn chart(&self) -> tree::ChartTree {
        tree::project(self.events.iter_all())
    }

    pub fn find_control_account_by_reference(
        &self,
        reference_to_check: String,
    ) -> Option<ControlAccountDetails> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartEvent::ControlAccountAdded {
                path,
                reference,
                id,
                name,
                ..
            } if reference_to_check == *reference => Some({
                ControlAccountDetails {
                    path: *path,
                    account_set_id: *id,
                    name: name.to_string(),
                    reference: reference.to_string(),
                }
            }),
            _ => None,
        })
    }

    pub fn create_control_account(
        &mut self,
        id: LedgerAccountSetId,
        category: ChartCategory,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    ) -> Result<ControlAccountDetails, ChartError> {
        if self
            .find_control_account_by_reference(reference.to_string())
            .is_some()
        {
            return Err(ChartError::ControlAccountAlreadyRegistered(reference));
        };

        let path = self.next_control_account(category)?;
        self.events.push(ChartEvent::ControlAccountAdded {
            id,
            encoded_path: path.path_encode(self.id),
            path,
            name: name.to_string(),
            reference: reference.to_string(),
            audit_info,
        });

        Ok(ControlAccountDetails {
            path,
            account_set_id: id,
            name,
            reference,
        })
    }

    fn next_control_sub_account(
        &self,
        control_account: ControlAccountPath,
    ) -> Result<ControlSubAccountPath, ChartError> {
        Ok(self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartEvent::ControlSubAccountAdded { path, .. }
                    if path.category == control_account.category
                        && path.control_account() == control_account =>
                {
                    Some(path.next())
                }
                _ => None,
            })
            .unwrap_or(Ok(control_account.first_control_sub_account()))?)
    }

    pub fn find_control_sub_account_by_reference(
        &self,
        reference_to_check: String,
    ) -> Option<ControlSubAccountDetails> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartEvent::ControlSubAccountAdded {
                path,
                id: account_set_id,
                name,
                reference,
                ..
            } if reference_to_check == *reference => Some(ControlSubAccountDetails {
                path: *path,
                account_set_id: *account_set_id,
                name: name.to_string(),
                reference: reference.to_string(),
            }),
            _ => None,
        })
    }

    pub fn create_control_sub_account(
        &mut self,
        id: LedgerAccountSetId,
        control_account: ControlAccountPath,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    ) -> Result<ControlSubAccountDetails, ChartError> {
        if self
            .find_control_sub_account_by_reference(reference.to_string())
            .is_some()
        {
            return Err(ChartError::ControlSubAccountAlreadyRegistered(reference));
        };

        let path = self.next_control_sub_account(control_account)?;
        self.events.push(ChartEvent::ControlSubAccountAdded {
            id,
            encoded_path: path.path_encode(self.id),
            path,
            name: name.to_string(),
            reference: reference.to_string(),
            audit_info,
        });

        Ok(ControlSubAccountDetails {
            path,
            account_set_id: id,
            name,
            reference,
        })
    }
}

impl Chart {
    fn segmentation(&self) -> Segmentation {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartEvent::Updated { segmentation, .. } => Some(segmentation.clone()),
                ChartEvent::Initialized { segmentation, .. } => Some(segmentation.clone()),
                _ => None,
            })
            .expect("'segmentation' not found")
    }

    pub fn find_node_by_reference(&self, reference_to_check: String) -> Option<NodeDetails> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartEvent::NodeAdded {
                reference,
                encoded_path,
                id,
                ..
            } if reference_to_check == *reference => Some({
                NodeDetails {
                    account_set_id: *id,
                    reference: reference.to_string(),
                    encoded_path: encoded_path.clone(),
                }
            }),
            _ => None,
        })
    }

    fn check_node_exists(&self, node: NodeDetails) -> Result<(), ChartError> {
        if self
            .find_node_by_reference(node.reference.to_string())
            .is_some()
        {
            return Err(ChartError::NodeAlreadyRegisteredByReference(node.reference));
        }

        if self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartEvent::NodeAdded { encoded_path, .. }
                    if node.encoded_path == *encoded_path =>
                {
                    Some(true)
                }
                _ => None,
            })
            .unwrap_or(false)
        {
            return Err(ChartError::NodeAlreadyRegisteredByPath(
                node.encoded_path.to_string(),
            ));
        }

        Ok(())
    }

    fn validate_node(&self, node: NodeDetails) -> Result<(), ChartError> {
        self.check_node_exists(node.clone())?;

        self.segmentation().check_path(node.encoded_path)?;

        Ok(())
    }

    pub fn create_node(
        &mut self,
        id: LedgerAccountSetId,
        reference: String,
        raw_code: String,
        audit_info: AuditInfo,
    ) -> Result<NodeDetails, ChartError> {
        let node_details = NodeDetails {
            account_set_id: id,
            reference,
            encoded_path: raw_code.parse()?,
        };
        self.validate_node(node_details.clone())?;

        self.events.push(ChartEvent::NodeAdded {
            id,
            encoded_path: node_details.encoded_path.clone(),
            reference: node_details.reference.to_string(),
            audit_info,
        });

        Ok(node_details)
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();
        for event in events.iter_all() {
            match event {
                ChartEvent::Initialized {
                    id,
                    reference,
                    name,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .reference(reference.to_string())
                        .name(name.to_string())
                }
                ChartEvent::Updated {
                    reference, name, ..
                } => {
                    builder = builder
                        .reference(reference.to_string())
                        .name(name.to_string())
                }
                ChartEvent::NodeAdded { .. } => (),
                ChartEvent::ControlAccountAdded { .. } => (),
                ChartEvent::ControlSubAccountAdded { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewChart {
    #[builder(setter(into))]
    pub(super) id: ChartId,
    pub(super) name: String,
    pub(super) reference: String,
    pub(super) segmentation: Segmentation,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewChart {
    pub fn builder() -> NewChartBuilder {
        NewChartBuilder::default()
    }
}

impl IntoEvents<ChartEvent> for NewChart {
    fn into_events(self) -> EntityEvents<ChartEvent> {
        EntityEvents::init(
            self.id,
            [ChartEvent::Initialized {
                id: self.id,
                name: self.name,
                reference: self.reference,
                segmentation: self.segmentation,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::path::{AccountIdx, ChartCategory};

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
            .segmentation(Segmentation {
                schema: vec![1, 1, 1, 1, 3],
            })
            .audit_info(audit_info)
            .build()
            .unwrap();

        let events = new_chart.into_events();
        Chart::try_from_events(events).unwrap()
    }

    #[test]
    fn test_create_new_chart_of_account() {
        let id = ChartId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChart::builder()
            .id(id)
            .name("Test Chart".to_string())
            .reference("ref-01".to_string())
            .segmentation(Segmentation::new(vec![]))
            .audit_info(audit_info.clone())
            .build()
            .unwrap();

        let events = new_chart.into_events();
        let chart = Chart::try_from_events(events).unwrap();

        assert_eq!(chart.id, id);
    }

    #[test]
    fn test_create_node() {
        let mut chart = init_chart_of_events();
        let res = chart.create_node(
            LedgerAccountSetId::new(),
            "assets".to_string(),
            "110".parse().unwrap(),
            dummy_audit_info(),
        );
        assert!(res.is_ok())
    }

    #[test]
    fn test_node_duplicate_reference() {
        let mut chart = init_chart_of_events();
        chart
            .create_node(
                LedgerAccountSetId::new(),
                "assets".to_string(),
                "110".parse().unwrap(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_node(
            LedgerAccountSetId::new(),
            "assets".to_string(),
            "111".parse().unwrap(),
            dummy_audit_info(),
        ) {
            Err(e) => {
                assert!(matches!(e, ChartError::NodeAlreadyRegisteredByReference(_)));
            }
            _ => {
                panic!("Expected duplicate reference to error")
            }
        }
    }

    #[test]
    fn test_node_duplicate_code() {
        let mut chart = init_chart_of_events();
        chart
            .create_node(
                LedgerAccountSetId::new(),
                "assets_01".to_string(),
                "110".parse().unwrap(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_node(
            LedgerAccountSetId::new(),
            "assets_02".to_string(),
            "110".parse().unwrap(),
            dummy_audit_info(),
        ) {
            Err(e) => {
                assert!(matches!(e, ChartError::NodeAlreadyRegisteredByPath(_)));
            }
            _ => {
                panic!("Expected duplicate reference to error")
            }
        }
    }

    #[test]
    fn test_create_control_account() {
        let mut chart = init_chart_of_events();
        let ControlAccountDetails {
            path: ControlAccountPath { category, index },
            ..
        } = chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(index, AccountIdx::FIRST);
    }

    #[test]
    fn test_control_account_duplicate_reference() {
        let mut chart = init_chart_of_events();
        chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "Assets #1".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_control_account(
            LedgerAccountSetId::new(),
            ChartCategory::Assets,
            "Assets #2".to_string(),
            "assets".to_string(),
            dummy_audit_info(),
        ) {
            Err(e) => {
                assert!(matches!(e, ChartError::ControlAccountAlreadyRegistered(_)));
            }
            _ => {
                panic!("Expected duplicate reference to error")
            }
        }
    }

    #[test]
    fn test_create_control_sub_account() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ControlSubAccountDetails {
            path:
                ControlSubAccountPath {
                    category,
                    control_index,
                    index,
                },
            ..
        } = chart
            .create_control_sub_account(
                LedgerAccountSetId::new(),
                control_account.path,
                "Current Assets".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(control_index, AccountIdx::FIRST);
        assert_eq!(index, AccountIdx::FIRST);
    }

    #[test]
    fn test_control_sub_account_duplicate_reference() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        chart
            .create_control_sub_account(
                LedgerAccountSetId::new(),
                control_account.path,
                "Current Assets #1".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_control_sub_account(
            LedgerAccountSetId::new(),
            control_account.path,
            "Current Assets #2".to_string(),
            "current-assets".to_string(),
            dummy_audit_info(),
        ) {
            Err(e) => {
                assert!(matches!(
                    e,
                    ChartError::ControlSubAccountAlreadyRegistered(_)
                ));
            }
            _ => {
                panic!("Expected duplicate reference to error")
            }
        }
    }

    #[test]
    fn test_create_sequential_control_accounts() {
        let mut chart = init_chart_of_events();

        chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "First".to_string(),
                "assets-01".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ControlAccountDetails {
            path: ControlAccountPath { category, index },
            ..
        } = chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "Second".to_string(),
                "assets-02".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(index, AccountIdx::FIRST.next());
    }

    #[test]
    fn test_create_sequential_control_sub_accounts() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                LedgerAccountSetId::new(),
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        chart
            .create_control_sub_account(
                LedgerAccountSetId::new(),
                control_account.path,
                "First".to_string(),
                "first-asset".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ControlSubAccountDetails {
            path:
                ControlSubAccountPath {
                    category,
                    control_index,
                    index,
                },
            ..
        } = chart
            .create_control_sub_account(
                LedgerAccountSetId::new(),
                control_account.path,
                "Second".to_string(),
                "second-asset".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(control_index, AccountIdx::FIRST);
        assert_eq!(index, AccountIdx::FIRST.next());
    }

    mod segmentation {
        use super::*;

        #[test]
        fn test_schema_by_length_from_start_to_segment_end() {
            let segmentation = Segmentation {
                schema: vec![2, 3, 4],
            };
            let expected = vec![2, 5, 9];
            assert_eq!(
                segmentation.schema_by_length_from_start_to_segment_end(),
                expected
            );
        }

        #[test]
        fn test_check_path_valid() {
            let segmentation = Segmentation {
                schema: vec![2, 3, 4],
            };
            let valid_path: EncodedPath = "12345".parse().unwrap();
            let result = segmentation.check_path(valid_path);
            assert!(
                result.is_ok(),
                "Expected valid code length to pass check_path"
            );
        }

        #[test]
        fn test_check_path_invalid() {
            let schema = vec![2, 3, 4];
            let segmentation = Segmentation { schema };
            let invalid_path: EncodedPath = "1234".parse().unwrap();
            let e = segmentation.check_path(invalid_path);
            assert!(matches!(e, Err(ChartError::InvalidCodeLength(4, _))));
        }
    }
}
