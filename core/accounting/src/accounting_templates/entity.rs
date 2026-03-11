use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

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
