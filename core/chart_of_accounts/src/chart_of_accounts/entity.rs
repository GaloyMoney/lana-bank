use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::primitives::ChartOfAccountId;

pub use super::{code::*, error::*};

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
        code: ChartOfAccountCode,
        name: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ChartOfAccount {
    pub id: ChartOfAccountId,
    pub(super) events: EntityEvents<ChartOfAccountEvent>,
}

pub struct ChartOfAccountAccountDetails {
    code: ChartOfAccountCode,
    name: String,
}

impl ChartOfAccount {
    fn next_control_account(
        &self,
        category: ChartOfAccountCode,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        self.events
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
            .unwrap_or_else(|| ChartOfAccountCode::first_control_account(category))
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
        self.events
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
            .unwrap_or_else(|| ChartOfAccountCode::first_control_sub_account(&control_account))
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
        self.events
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
            .unwrap_or_else(|| ChartOfAccountCode::first_transaction_account(&control_sub_account))
    }

    pub fn create_transaction_account(
        &mut self,
        control_sub_account: ChartOfAccountCode,
        name: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountCode, ChartOfAccountError> {
        let code = self.next_transaction_account(control_sub_account)?;
        self.events
            .push(ChartOfAccountEvent::TransactionAccountAdded {
                code,
                name: name.to_string(),
                audit_info,
            });

        Ok(code)
    }

    pub fn find_account(
        &self,
        account_code: ChartOfAccountCode,
    ) -> Option<ChartOfAccountAccountDetails> {
        self.events.iter_all().rev().find_map(|event| match event {
            ChartOfAccountEvent::TransactionAccountAdded { code, name, .. }
                if *code == account_code =>
            {
                Some(ChartOfAccountAccountDetails {
                    code: *code,
                    name: name.to_string(),
                })
            }
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
