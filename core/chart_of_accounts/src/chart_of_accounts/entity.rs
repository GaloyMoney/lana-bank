use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::{
    code::*,
    primitives::{ChartOfAccountAccountDetails, ChartOfAccountId, LedgerAccountId},
};

pub use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartOfAccountId")]
pub enum ChartOfAccountEvent {
    Initialized {
        id: ChartOfAccountId,
        audit_info: AuditInfo,
    },
    ControlAccountAdded {
        code: ChartOfAccountCode,
        name: String,
        audit_info: AuditInfo,
    },
    ControlSubAccountAdded {
        code: ChartOfAccountCode,
        name: String,
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
    pub id: ChartOfAccountId,
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

    pub fn create_control_account(
        &mut self,
        category: ChartOfAccountCode,
        name: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        let code = self.next_control_account(category)?;
        self.events.push(ChartOfAccountEvent::ControlAccountAdded {
            code,
            name: name.to_string(),
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

    pub fn create_control_sub_account(
        &mut self,
        control_account: ChartOfAccountCode,
        name: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        let code = self.next_control_sub_account(control_account)?;
        self.events
            .push(ChartOfAccountEvent::ControlSubAccountAdded {
                code,
                name: name.to_string(),
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

    pub fn create_transaction_account(
        &mut self,
        control_sub_account: ChartOfAccountCode,
        name: &str,
        description: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountAccountDetails, ChartOfAccountError> {
        let code = self.next_transaction_account(control_sub_account)?;
        let account_id = LedgerAccountId::new();
        self.events
            .push(ChartOfAccountEvent::TransactionAccountAdded {
                id: account_id,
                code,
                name: name.to_string(),
                description: description.to_string(),
                audit_info,
            });

        Ok(ChartOfAccountAccountDetails {
            account_id,
            code,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    pub fn find_account(
        &self,
        account_code: ChartOfAccountCode,
    ) -> Option<ChartOfAccountAccountDetails> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartOfAccountEvent::TransactionAccountAdded {
                id,
                code,
                name,
                description,
                ..
            } if *code == account_code => Some(ChartOfAccountAccountDetails {
                account_id: *id,
                code: *code,
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
                ChartOfAccountEvent::Initialized { id, .. } => builder = builder.id(*id),
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
    pub(super) id: ChartOfAccountId,
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
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{AccountIdx, ChartOfAccountCategoryCode};

    use super::*;

    use audit::{AuditEntryId, AuditInfo};

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn init_chart_of_events() -> ChartOfAccount {
        let id = ChartOfAccountId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChartOfAccount::builder()
            .id(id)
            .audit_info(audit_info)
            .build()
            .unwrap();

        let events = new_chart.into_events();
        ChartOfAccount::try_from_events(events).unwrap()
    }

    #[test]
    fn test_create_new_chart_of_account() {
        let id = ChartOfAccountId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChartOfAccount::builder()
            .id(id)
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
                "Assets",
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
    fn test_create_control_sub_account() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Assets",
                dummy_audit_info(),
            )
            .unwrap();

        match chart
            .create_control_sub_account(control_account, "Current Assets", dummy_audit_info())
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
    fn test_create_transaction_account() {
        let mut chart = init_chart_of_events();
        let control_account = chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Assets",
                dummy_audit_info(),
            )
            .unwrap();
        let control_sub_account = chart
            .create_control_sub_account(control_account, "Current Assets", dummy_audit_info())
            .unwrap();

        match chart
            .create_transaction_account(
                control_sub_account,
                "Cash",
                "Cash account",
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountAccountDetails {
                code:
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
                "First",
                dummy_audit_info(),
            )
            .unwrap();

        match chart
            .create_control_account(
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets),
                "Second",
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
                "Assets",
                dummy_audit_info(),
            )
            .unwrap();

        chart
            .create_control_sub_account(control_account, "First", dummy_audit_info())
            .unwrap();

        match chart
            .create_control_sub_account(control_account, "Second", dummy_audit_info())
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
                "Assets",
                dummy_audit_info(),
            )
            .unwrap();
        let sub_account = chart
            .create_control_sub_account(control_account, "Current Assets", dummy_audit_info())
            .unwrap();

        chart
            .create_transaction_account(
                sub_account,
                "First",
                "First transaction account",
                dummy_audit_info(),
            )
            .unwrap();

        match chart
            .create_transaction_account(
                sub_account,
                "Second",
                "Second transaction account",
                dummy_audit_info(),
            )
            .unwrap()
        {
            ChartOfAccountAccountDetails {
                code:
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
            .create_control_account(category, "Assets", audit_info.clone())
            .unwrap();
        let sub_account = chart
            .create_control_sub_account(control_account, "Current Assets", audit_info.clone())
            .unwrap();
        let transaction_account = chart
            .create_transaction_account(sub_account, "Cash", "Cash account", audit_info)
            .unwrap();

        let found = chart.find_account(transaction_account.code).unwrap();
        assert_eq!(found.code, transaction_account.code);
        assert_eq!(found.name, "Cash");

        assert!(chart.find_account("20101001".parse().unwrap()).is_none());
    }
}
