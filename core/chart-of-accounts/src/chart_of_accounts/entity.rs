use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::{
    code::*,
    primitives::{ChartId, ChartOfAccountAccountDetails, LedgerAccountId},
};

pub use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartOfAccountEvent {
    Initialized {
        id: ChartId,
        reference: String,
        audit_info: AuditInfo,
    },
    ControlAccountAdded {
        code: ChartOfAccountCode,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
    ControlSubAccountAdded {
        code: ChartOfAccountCode,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
    TransactionAccountAdded {
        id: LedgerAccountId,
        code: ChartOfAccountCode,
        name: String,
        description: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ChartOfAccount {
    pub id: ChartId,
    pub reference: String,
    pub(super) events: EntityEvents<ChartOfAccountEvent>,
}

impl ChartOfAccount {
    fn next_control_account(
        &self,
        category: ChartOfAccountCode,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        Ok(self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartOfAccountEvent::ControlAccountAdded { code, .. }
                    if code.category() == category.category() =>
                {
                    Some(code.next())
                }
                _ => None,
            })
            .unwrap_or_else(|| ChartOfAccountCode::first_control_account(category))?)
    }

    pub fn find_control_account_by_reference(
        &self,
        reference_to_check: String,
    ) -> Option<ChartOfAccountCode> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartOfAccountEvent::ControlAccountAdded {
                code, reference, ..
            } if reference_to_check == *reference => Some(*code),
            _ => None,
        })
    }

    pub fn create_control_account(
        &mut self,
        category: ChartOfAccountCode,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        if self
            .find_control_account_by_reference(reference.to_string())
            .is_some()
        {
            return Err(ChartOfAccountError::ControlAccountAlreadyRegistered(
                reference,
            ));
        };

        let code = self.next_control_account(category)?;
        self.events.push(ChartOfAccountEvent::ControlAccountAdded {
            code,
            name,
            reference,
            audit_info,
        });

        Ok(code)
    }

    fn next_control_sub_account(
        &self,
        control_account: ChartOfAccountCode,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        Ok(self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartOfAccountEvent::ControlSubAccountAdded { code, .. }
                    if code.category() == control_account.category()
                        && code.control_account() == control_account.control_account() =>
                {
                    Some(code.next())
                }
                _ => None,
            })
            .unwrap_or_else(|| ChartOfAccountCode::first_control_sub_account(&control_account))?)
    }

    pub fn find_control_sub_account_by_reference(
        &self,
        reference_to_check: String,
    ) -> Option<ChartOfAccountCode> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartOfAccountEvent::ControlSubAccountAdded {
                code, reference, ..
            } if reference_to_check == *reference => Some(*code),
            _ => None,
        })
    }

    pub fn create_control_sub_account(
        &mut self,
        control_account: ChartOfAccountCode,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        if self
            .find_control_sub_account_by_reference(reference.to_string())
            .is_some()
        {
            return Err(ChartOfAccountError::ControlSubAccountAlreadyRegistered(
                reference,
            ));
        };

        let code = self.next_control_sub_account(control_account)?;
        self.events
            .push(ChartOfAccountEvent::ControlSubAccountAdded {
                code,
                name,
                reference,
                audit_info,
            });

        Ok(code)
    }

    fn next_transaction_account(
        &self,
        control_sub_account: ChartOfAccountCode,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        Ok(self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                ChartOfAccountEvent::TransactionAccountAdded { code, .. }
                    if code.category() == control_sub_account.category()
                        && code.control_account() == control_sub_account.control_account()
                        && code.control_sub_account()
                            == control_sub_account.control_sub_account() =>
                {
                    Some(code.next())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                ChartOfAccountCode::first_transaction_account(&control_sub_account)
            })?)
    }

    pub fn add_transaction_account(
        &mut self,
        account_id: impl Into<LedgerAccountId>,
        control_sub_account: ChartOfAccountCode,
        name: &str,
        description: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountAccountDetails, ChartOfAccountError> {
        let account_id = account_id.into();
        let path = self.next_transaction_account(control_sub_account)?;
        self.events
            .push(ChartOfAccountEvent::TransactionAccountAdded {
                id: account_id,
                code: path,
                name: name.to_string(),
                description: description.to_string(),
                audit_info,
            });

        Ok(ChartOfAccountAccountDetails {
            account_id,
            code: path.to_code(self.id),
            path,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    pub fn find_account(
        &self,
        account_path: ChartOfAccountCode,
    ) -> Option<ChartOfAccountAccountDetails> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartOfAccountEvent::TransactionAccountAdded {
                id,
                code: path,
                name,
                description,
                ..
            } if *path == account_path => Some(ChartOfAccountAccountDetails {
                account_id: *id,
                path: *path,
                code: path.to_code(self.id),
                name: name.to_string(),
                description: description.to_string(),
            }),
            _ => None,
        })
    }
}

impl TryFromEvents<ChartOfAccountEvent> for ChartOfAccount {
    fn try_from_events(events: EntityEvents<ChartOfAccountEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartOfAccountBuilder::default();
        for event in events.iter_all() {
            match event {
                ChartOfAccountEvent::Initialized { id, reference, .. } => {
                    builder = builder.id(*id).reference(reference.to_string())
                }
                ChartOfAccountEvent::ControlAccountAdded { .. } => (),
                ChartOfAccountEvent::ControlSubAccountAdded { .. } => (),
                ChartOfAccountEvent::TransactionAccountAdded { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewChartOfAccount {
    #[builder(setter(into))]
    pub(super) id: ChartId,
    pub(super) reference: String,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewChartOfAccount {
    pub fn builder() -> NewChartOfAccountBuilder {
        NewChartOfAccountBuilder::default()
    }
}

impl IntoEvents<ChartOfAccountEvent> for NewChartOfAccount {
    fn into_events(self) -> EntityEvents<ChartOfAccountEvent> {
        EntityEvents::init(
            self.id,
            [ChartOfAccountEvent::Initialized {
                id: self.id,
                reference: self.reference,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::code::{AccountIdx, ChartOfAccountCategoryCode};

    use super::*;

    use audit::{AuditEntryId, AuditInfo};

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn init_chart_of_events() -> ChartOfAccount {
        let id = ChartId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChartOfAccount::builder()
            .id(id)
            .reference("ref-01".to_string())
            .audit_info(audit_info)
            .build()
            .unwrap();

        let events = new_chart.into_events();
        ChartOfAccount::try_from_events(events).unwrap()
    }

    #[test]
    fn test_create_new_chart_of_account() {
        let id = ChartId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChartOfAccount::builder()
            .id(id)
            .reference("ref-01".to_string())
            .audit_info(audit_info.clone())
            .build()
            .unwrap();

        let events = new_chart.into_events();
        let chart = ChartOfAccount::try_from_events(events).unwrap();

        assert_eq!(chart.id, id);
    }

    #[test]
    fn test_create_control_account() {
        let mut chart = init_chart_of_events();
        match chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountCode::ControlAccount { category, index } => {
                assert_eq!(category, ChartOfAccountCategoryCode::Assets);
                assert_eq!(index, AccountIdx::FIRST);
            }
            other => panic!("Expected FIRST control account, got {:?}", other),
        }
    }

    #[test]
    fn test_control_account_duplicate_reference() {
        let mut chart = init_chart_of_events();
        chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Assets #1".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart.create_control_account(
            ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
            "Assets #2".to_string(),
            "assets".to_string(),
            dummy_audit_info(),
        ) {
            Err(e) => {
                assert!(matches!(
                    e,
                    ChartOfAccountError::ControlAccountAlreadyRegistered(_)
                ));
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
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart
            .create_control_sub_account(
                control_account,
                "Current Assets".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountCode::ControlSubAccount {
                category,
                control_index,
                index,
            } => {
                assert_eq!(category, ChartOfAccountCategoryCode::Assets);
                assert_eq!(control_index, AccountIdx::FIRST);
                assert_eq!(index, AccountIdx::FIRST);
            }
            other => panic!("Expected FIRST control sub account, got {:?}", other),
        }
    }

    #[test]
    fn test_control_sub_account_duplicate_reference() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
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
                    ChartOfAccountError::ControlSubAccountAlreadyRegistered(_)
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
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
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

        match chart
            .add_transaction_account(
                LedgerAccountId::new(),
                control_sub_account,
                "Cash",
                "Cash account",
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountAccountDetails {
                path:
                    ChartOfAccountCode::TransactionAccount {
                        category,
                        control_index,
                        control_sub_index,
                        index,
                    },
                ..
            } => {
                assert_eq!(category, ChartOfAccountCategoryCode::Assets);
                assert_eq!(control_index, AccountIdx::FIRST);
                assert_eq!(control_sub_index, AccountIdx::FIRST);
                assert_eq!(index, AccountIdx::FIRST);
            }
            other => panic!("Expected FIRST transaction account, got {:?}", other),
        }
    }

    #[test]
    fn test_create_sequential_control_accounts() {
        let mut chart = init_chart_of_events();

        chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "First".to_string(),
                "assets-01".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        match chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Second".to_string(),
                "assets-02".to_string(),
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountCode::ControlAccount { category, index } => {
                assert_eq!(category, ChartOfAccountCategoryCode::Assets);
                assert_eq!(index, AccountIdx::FIRST.next());
            }
            other => panic!("Expected SECOND control account, got {:?}", other),
        }
    }

    #[test]
    fn test_create_sequential_control_sub_accounts() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
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

        match chart
            .create_control_sub_account(
                control_account,
                "Second".to_string(),
                "second-asset".to_string(),
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountCode::ControlSubAccount {
                category,
                control_index,
                index,
            } => {
                assert_eq!(category, ChartOfAccountCategoryCode::Assets);
                assert_eq!(control_index, AccountIdx::FIRST);
                assert_eq!(index, AccountIdx::FIRST.next());
            }
            other => panic!("Expected SECOND control sub account, got {:?}", other),
        }
    }

    #[test]
    fn test_create_sequential_transaction_accounts() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Assets".to_string(),
                "assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();
        let sub_account = chart
            .create_control_sub_account(
                control_account,
                "Current Assets".to_string(),
                "current-assets".to_string(),
                dummy_audit_info(),
            )
            .unwrap();

        chart
            .add_transaction_account(
                LedgerAccountId::new(),
                sub_account,
                "First",
                "First transaction account",
                dummy_audit_info(),
            )
            .unwrap();

        match chart
            .add_transaction_account(
                LedgerAccountId::new(),
                sub_account,
                "Second",
                "Second transaction account",
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountAccountDetails {
                path:
                    ChartOfAccountCode::TransactionAccount {
                        category,
                        control_index,
                        control_sub_index,
                        index,
                    },
                ..
            } => {
                assert_eq!(category, ChartOfAccountCategoryCode::Assets);
                assert_eq!(control_index, AccountIdx::FIRST);
                assert_eq!(control_sub_index, AccountIdx::FIRST);
                assert_eq!(index, AccountIdx::FIRST.next());
            }
            other => panic!("Expected SECOND transaction account, got {:?}", other),
        }
    }

    #[test]
    fn test_find_account() {
        let mut chart = init_chart_of_events();
        let audit_info = dummy_audit_info();

        let category = ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets);
        let control_account = chart
            .create_control_account(
                category,
                "Assets".to_string(),
                "assets".to_string(),
                audit_info.clone(),
            )
            .unwrap();
        let sub_account = chart
            .create_control_sub_account(
                control_account,
                "Current Assets".to_string(),
                "current-assets".to_string(),
                audit_info.clone(),
            )
            .unwrap();
        let transaction_account = chart
            .add_transaction_account(
                LedgerAccountId::new(),
                sub_account,
                "Cash",
                "Cash account",
                audit_info,
            )
            .unwrap();

        let found = chart.find_account(transaction_account.path).unwrap();
        assert_eq!(found.code, transaction_account.code);
        assert_eq!(found.name, "Cash");

        assert!(chart.find_account("20101001".parse().unwrap()).is_none());
    }
}
