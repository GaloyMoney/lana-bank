use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::{
    path::*,
    primitives::{ChartAccountDetails, ChartCreationDetails, ChartId, LedgerAccountId},
};

pub use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartEvent {
    Initialized {
        id: ChartId,
        reference: String,
        audit_info: AuditInfo,
    },
    ControlAccountAdded {
        encoded_path: String,
        path: ControlAccountPath,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
    ControlSubAccountAdded {
        encoded_path: String,
        path: ControlSubAccountPath,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
    TransactionAccountAdded {
        id: LedgerAccountId,
        encoded_path: String,
        path: TransactionAccountPath,
        name: String,
        description: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
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

    pub fn find_control_account_by_reference(
        &self,
        reference_to_check: String,
    ) -> Option<ControlAccountPath> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartEvent::ControlAccountAdded {
                path, reference, ..
            } if reference_to_check == *reference => Some(*path),
            _ => None,
        })
    }

    pub fn create_control_account(
        &mut self,
        category: ChartCategory,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    ) -> Result<ControlAccountPath, ChartError> {
        if self
            .find_control_account_by_reference(reference.to_string())
            .is_some()
        {
            return Err(ChartError::ControlAccountAlreadyRegistered(reference));
        };

        let path = self.next_control_account(category)?;
        self.events.push(ChartEvent::ControlAccountAdded {
            encoded_path: path.path_encode(self.id),
            path,
            name,
            reference,
            audit_info,
        });

        Ok(path)
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
    ) -> Option<ControlSubAccountPath> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartEvent::ControlSubAccountAdded {
                path, reference, ..
            } if reference_to_check == *reference => Some(*path),
            _ => None,
        })
    }

    pub fn create_control_sub_account(
        &mut self,
        control_account: ControlAccountPath,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    ) -> Result<ControlSubAccountPath, ChartError> {
        if self
            .find_control_sub_account_by_reference(reference.to_string())
            .is_some()
        {
            return Err(ChartError::ControlSubAccountAlreadyRegistered(reference));
        };

        let path = self.next_control_sub_account(control_account)?;
        self.events.push(ChartEvent::ControlSubAccountAdded {
            encoded_path: path.path_encode(self.id),
            path,
            name,
            reference,
            audit_info,
        });

        Ok(path)
    }

    fn next_transaction_account(
        &self,
        control_sub_account: ControlSubAccountPath,
    ) -> Result<TransactionAccountPath, ChartError> {
        Ok(self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartEvent::TransactionAccountAdded { path, .. }
                    if path.category == control_sub_account.category
                        && path.control_account() == control_sub_account.control_account()
                        && path.control_sub_account() == control_sub_account =>
                {
                    Some(path.next())
                }
                _ => None,
            })
            .unwrap_or(Ok(control_sub_account.first_transaction_account()))?)
    }

    pub fn add_transaction_account(
        &mut self,
        creation_details: ChartCreationDetails,
        audit_info: AuditInfo,
    ) -> Result<ChartAccountDetails, ChartError> {
        let path = self.next_transaction_account(creation_details.control_sub_account)?;
        self.events.push(ChartEvent::TransactionAccountAdded {
            id: creation_details.account_id,
            encoded_path: path.path_encode(self.id),
            path,
            name: creation_details.name.clone(),
            description: creation_details.description.clone(),
            audit_info,
        });

        Ok(ChartAccountDetails {
            account_id: creation_details.account_id,
            encoded_path: path.path_encode(self.id),
            path,
            name: creation_details.name,
            description: creation_details.description,
        })
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();
        for event in events.iter_all() {
            match event {
                ChartEvent::Initialized { id, reference, .. } => {
                    builder = builder.id(*id).reference(reference.to_string())
                }
                ChartEvent::ControlAccountAdded { .. } => (),
                ChartEvent::ControlSubAccountAdded { .. } => (),
                ChartEvent::TransactionAccountAdded { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewChart {
    #[builder(setter(into))]
    pub(super) id: ChartId,
    pub(super) reference: String,
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
                reference: self.reference,
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
            .reference("ref-01".to_string())
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
            .reference("ref-01".to_string())
            .audit_info(audit_info.clone())
            .build()
            .unwrap();

        let events = new_chart.into_events();
        let chart = Chart::try_from_events(events).unwrap();

        assert_eq!(chart.id, id);
    }

    #[test]
    fn test_create_control_account() {
        let mut chart = init_chart_of_events();
        let ControlAccountPath { category, index } = chart
            .create_control_account(
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
                ChartCategory::Assets,
                "Assets #1".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_control_account(
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
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ControlSubAccountPath {
            category,
            control_index,
            index,
        } = chart
            .create_control_sub_account(
                control_account,
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
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        chart
            .create_control_sub_account(
                control_account,
                "Current Assets #1".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_control_sub_account(
            control_account,
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
    fn test_create_transaction_account() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        let control_sub_account = chart
            .create_control_sub_account(
                control_account,
                "Current Assets".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ChartAccountDetails {
            path:
                TransactionAccountPath {
                    category,
                    control_index,
                    control_sub_index,
                    index,
                },
            ..
        } = chart
            .add_transaction_account(
                ChartCreationDetails {
                    account_id: LedgerAccountId::new(),
                    control_sub_account,
                    name: "Cash".to_string(),
                    description: "Cash account".to_string(),
                },
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(control_index, AccountIdx::FIRST);
        assert_eq!(control_sub_index, AccountIdx::FIRST);
        assert_eq!(index, AccountIdx::FIRST);
    }

    #[test]
    fn test_create_sequential_control_accounts() {
        let mut chart = init_chart_of_events();

        chart
            .create_control_account(
                ChartCategory::Assets,
                "First".to_string(),
                "assets-01".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ControlAccountPath { category, index } = chart
            .create_control_account(
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
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        chart
            .create_control_sub_account(
                control_account,
                "First".to_string(),
                "first-asset".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        let ControlSubAccountPath {
            category,
            control_index,
            index,
        } = chart
            .create_control_sub_account(
                control_account,
                "Second".to_string(),
                "second-asset".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(control_index, AccountIdx::FIRST);
        assert_eq!(index, AccountIdx::FIRST.next());
    }

    #[test]
    fn test_create_sequential_transaction_accounts() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartCategory::Assets,
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        let control_sub_account = chart
            .create_control_sub_account(
                control_account,
                "Current Assets".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        chart
            .add_transaction_account(
                ChartCreationDetails {
                    account_id: LedgerAccountId::new(),
                    control_sub_account,
                    name: "First".to_string(),
                    description: "First transaction account".to_string(),
                },
                dummy_audit_info(),
            )
            .unwrap();

        let ChartAccountDetails {
            path:
                TransactionAccountPath {
                    category,
                    control_index,
                    control_sub_index,
                    index,
                },
            ..
        } = chart
            .add_transaction_account(
                ChartCreationDetails {
                    account_id: LedgerAccountId::new(),
                    control_sub_account,
                    name: "Second".to_string(),
                    description: "Second transaction account".to_string(),
                },
                dummy_audit_info(),
            )
            .unwrap();
        assert_eq!(category, ChartCategory::Assets);
        assert_eq!(control_index, AccountIdx::FIRST);
        assert_eq!(control_sub_index, AccountIdx::FIRST);
        assert_eq!(index, AccountIdx::FIRST.next());
    }
}
