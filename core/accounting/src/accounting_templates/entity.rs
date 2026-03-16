use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::accounting_templates::AccountingTemplateError;

use super::{AccountingTemplateId, AccountingTemplateValues};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingTemplateId")]
pub enum AccountingTemplateEvent {
    Initialized {
        id: AccountingTemplateId,
        code: String,
        name: String,
        values: AccountingTemplateValues,
    },
    ValuesUpdated {
        values: AccountingTemplateValues,
    },
    NameUpdated {
        name: String,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingTemplate {
    pub id: AccountingTemplateId,
    pub code: String,
    pub name: String,
    pub values: AccountingTemplateValues,
    events: EntityEvents<AccountingTemplateEvent>,
}

impl AccountingTemplate {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("AccountingTemplate has never been persisted")
    }

    pub fn update_values(&mut self, new_values: AccountingTemplateValues) -> Idempotent<()> {
        if self.values == new_values {
            return Idempotent::AlreadyApplied;
        }

        self.events.push(AccountingTemplateEvent::ValuesUpdated {
            values: new_values.clone(),
        });
        self.values = new_values;
        Idempotent::Executed(())
    }

    pub fn update_name(&mut self, new_name: String) -> Idempotent<()> {
        if self.name == new_name {
            return Idempotent::AlreadyApplied;
        }

        self.events.push(AccountingTemplateEvent::NameUpdated {
            name: new_name.clone(),
        });
        self.name = new_name;
        Idempotent::Executed(())
    }
}

impl AccountingTemplateValues {
    pub fn validate_code(code: String) -> Result<String, AccountingTemplateError> {
        if code.is_empty() {
            return Err(AccountingTemplateError::InvalidCode(
                "Code cannot be empty".into(),
            ));
        }
        if !code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(AccountingTemplateError::InvalidCode(
                "code must be uppercase alphanumeric with underscores".into(),
            ));
        }
        Ok(code)
    }

    pub fn validate_name(name: String) -> Result<String, AccountingTemplateError> {
        if name.is_empty() {
            return Err(AccountingTemplateError::InvalidName(
                "Name cannot be empty".into(),
            ));
        }
        if name.len() > 255 {
            return Err(AccountingTemplateError::InvalidName("Name too long".into()));
        }
        Ok(name)
    }

    pub fn validate(&self) -> Result<(), AccountingTemplateError> {
        if self.entries.is_empty() {
            return Err(AccountingTemplateError::InvalidTemplate(
                "Template must have at least one entry".into(),
            ));
        }

        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.account_id_or_code.is_empty() {
                return Err(AccountingTemplateError::InvalidEntry(
                    idx,
                    "Account reference cannot be empty".into(),
                ));
            }
        }

        Ok(())
    }
}

impl TryFromEvents<AccountingTemplateEvent> for AccountingTemplate {
    fn try_from_events(
        events: EntityEvents<AccountingTemplateEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = AccountingTemplateBuilder::default();

        for event in events.iter_all() {
            match event {
                AccountingTemplateEvent::Initialized {
                    id,
                    code,
                    name,
                    values,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .code(code.clone())
                        .name(name.clone())
                        .values(values.clone());
                }
                AccountingTemplateEvent::ValuesUpdated { values, .. } => {
                    builder = builder.values(values.clone());
                }
                AccountingTemplateEvent::NameUpdated { name, .. } => {
                    builder = builder.name(name.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewAccountingTemplate {
    #[builder(setter(into))]
    pub id: AccountingTemplateId,
    #[builder(setter(into))]
    pub code: String,
    #[builder(setter(into))]
    pub name: String,
    pub values: AccountingTemplateValues,
}

impl NewAccountingTemplate {
    pub fn builder() -> NewAccountingTemplateBuilder {
        NewAccountingTemplateBuilder::default()
    }

    pub fn build(self) -> Result<AccountingTemplateEvent, AccountingTemplateError> {
        let code = AccountingTemplateValues::validate_code(self.code)?;

        let name = AccountingTemplateValues::validate_name(self.name)?;

        self.values.validate()?;

        Ok(AccountingTemplateEvent::Initialized {
            id: self.id,
            code,
            name,
            values: self.values,
        })
    }
}

impl IntoEvents<AccountingTemplateEvent> for NewAccountingTemplate {
    fn into_events(self) -> EntityEvents<AccountingTemplateEvent> {
        EntityEvents::init(
            self.id,
            [AccountingTemplateEvent::Initialized {
                id: self.id,
                code: self.code,
                name: self.name,
                values: self.values,
            }],
        )
    }
}
