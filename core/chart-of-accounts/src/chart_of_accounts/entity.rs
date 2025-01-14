use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::{
    path::*,
    primitives::{ChartAccountDetails, ChartCreationDetails, ChartId, LedgerAccountSetId},
    ControlSubAccountDetails,
};

use super::chart;
pub use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartEvent {
    Initialized {
        id: ChartId,
        name: String,
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

    pub fn chart(&self) -> chart::ChartOfAccountsProjection {
        chart::project(self.events.iter_all())
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

    pub fn add_transaction_account(
        &mut self,
        creation_details: ChartCreationDetails,
        _audit_info: AuditInfo,
    ) -> Result<ChartAccountDetails, ChartError> {
        // TODO: implement by potentially adding account to ControlSubAccount nested entity

        Ok(ChartAccountDetails {
            account_id: creation_details.account_id,
            encoded_path: format!(
                "{}.{}",
                creation_details.control_sub_account, creation_details.account_id
            ),
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
                LedgerAccountSetId::new(),
                control_account,
                "Current Assets #1".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_control_sub_account(
            LedgerAccountSetId::new(),
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
                LedgerAccountSetId::new(),
                control_account,
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
}
