use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use audit::AuditInfo;

use es_entity::*;

use crate::primitives::*;

use super::{error::*, tree};

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
    NodeAdded {
        spec: AccountSpec,
        ledger_account_set_id: CalaAccountSetId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
    pub name: String,
    all_accounts: HashMap<AccountCode, (AccountSpec, CalaAccountSetId)>,

    events: EntityEvents<ChartEvent>,
}

impl Chart {
    pub fn create_node(
        &mut self,
        spec: &AccountSpec,
        audit_info: AuditInfo,
    ) -> Idempotent<(Option<CalaAccountSetId>, CalaAccountSetId)> {
        if self.all_accounts.contains_key(&spec.code) {
            return Idempotent::Ignored;
        }
        let ledger_account_set_id = CalaAccountSetId::new();
        self.events.push(ChartEvent::NodeAdded {
            spec: spec.clone(),
            ledger_account_set_id,
            audit_info,
        });
        let parent = if let Some(parent) = spec.parent.as_ref() {
            self.all_accounts.get(parent).map(|(_, id)| *id)
        } else {
            None
        };
        self.all_accounts
            .insert(spec.code.clone(), (spec.clone(), ledger_account_set_id));
        Idempotent::Executed((parent, ledger_account_set_id))
    }

    pub fn trial_balance_account_ids_from_new_accounts(
        &self,
        new_account_set_ids: &[CalaAccountSetId],
    ) -> impl Iterator<Item = CalaAccountSetId> {
        self.all_accounts
            .values()
            .filter(move |(spec, id)| {
                spec.code.len_sections() == 2 && new_account_set_ids.contains(id)
            })
            .map(|(_, id)| *id)
    }

    pub fn account_spec(&self, code: &AccountCode) -> Option<&(AccountSpec, CalaAccountSetId)> {
        self.all_accounts.get(code)
    }

    /// Returns ancestors of this chart of accounts, starting with `code` (not included).
    /// The lower in hierarchy the parent is, the lower index it will have in the resulting vector;
    /// the root of the chart of accounts will be last.
    pub fn ancestors<T: From<CalaAccountSetId>>(&self, code: &AccountCode) -> Vec<T> {
        let mut result = Vec::new();
        let mut current_code = code;

        if let Some((spec, _)) = self.all_accounts.get(current_code) {
            current_code = match &spec.parent {
                Some(parent_code) => parent_code,
                None => return result,
            };
        } else {
            return result;
        }

        while let Some((spec, account_set_id)) = self.all_accounts.get(current_code) {
            result.push(T::from(*account_set_id));
            match &spec.parent {
                Some(parent_code) => current_code = parent_code,
                None => break,
            }
        }

        result
    }

    pub fn children<T: From<CalaAccountSetId>>(&self, code: &AccountCode) -> Vec<T> {
        self.all_accounts
            .values()
            .filter(|(spec, _)| spec.parent.as_ref() == Some(code))
            .map(|(_, account_set_id)| T::from(*account_set_id))
            .collect()
    }

    pub fn account_spec_from_code_str(
        &self,
        code: String,
    ) -> Option<&(AccountSpec, CalaAccountSetId)> {
        if let Ok(code) = code.parse() {
            if let Some(spec) = self.account_spec(&code) {
                return Some(spec);
            }
            if code.len_sections() > 1 {
                return None;
            }
        }

        self.all_accounts
            .iter()
            .find(|(k, _)| k.is_equivalent_to_str(&code))
            .map(|(_, v)| v)
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<CalaAccountSetId, ChartOfAccountsError> {
        self.account_spec(code)
            .map(|(_, id)| *id)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))
    }

    pub fn chart(&self) -> tree::ChartTree {
        tree::project(self.events.iter_all())
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();
        let mut all_accounts = HashMap::new();
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
                ChartEvent::NodeAdded {
                    spec,
                    ledger_account_set_id,
                    ..
                } => {
                    all_accounts.insert(spec.code.clone(), (spec.clone(), *ledger_account_set_id));
                }
            }
        }
        builder.all_accounts(all_accounts).events(events).build()
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
mod test {
    use audit::{AuditEntryId, AuditInfo};

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn chart_from(events: Vec<ChartEvent>) -> Chart {
        Chart::try_from_events(EntityEvents::init(ChartId::new(), events)).unwrap()
    }

    fn initial_events() -> Vec<ChartEvent> {
        vec![ChartEvent::Initialized {
            id: ChartId::new(),
            name: "Test Chart".to_string(),
            reference: "test-chart".to_string(),
            audit_info: dummy_audit_info(),
        }]
    }

    fn default_chart() -> (
        Chart,
        (CalaAccountSetId, CalaAccountSetId, CalaAccountSetId),
    ) {
        let mut chart = chart_from(initial_events());
        let (_, level_1_id) = chart
            .create_node(
                &AccountSpec::new(
                    None,
                    vec![section("1")],
                    "Assets".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                ),
                dummy_audit_info(),
            )
            .expect("Already executed");
        let (_, level_2_id) = chart
            .create_node(
                &AccountSpec::new(
                    Some(code("1")),
                    vec![section("1"), section("1")],
                    "Current Assets".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                ),
                dummy_audit_info(),
            )
            .expect("Already executed");
        let (_, level_3_id) = chart
            .create_node(
                &AccountSpec::new(
                    Some(code("1.1")),
                    vec![section("1"), section("1"), section("1")],
                    "Cash".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                ),
                dummy_audit_info(),
            )
            .expect("Already executed");

        (chart, (level_1_id, level_2_id, level_3_id))
    }

    fn section(s: &str) -> AccountCodeSection {
        s.parse::<AccountCodeSection>().unwrap()
    }

    fn code(s: &str) -> AccountCode {
        s.parse::<AccountCode>().unwrap()
    }

    #[test]
    fn adds_from_all_new_trial_balance_accounts() {
        let (chart, (level_1_id, level_2_id, level_3_id)) = default_chart();

        let new_ids = chart
            .trial_balance_account_ids_from_new_accounts(&[level_1_id, level_2_id, level_3_id])
            .collect::<Vec<_>>();
        assert_eq!(new_ids.len(), 1);
        assert!(new_ids.contains(&level_2_id));
    }

    #[test]
    fn adds_from_some_new_trial_balance_accounts() {
        let (mut chart, _) = default_chart();

        let (_, new_account_set_id) = chart
            .create_node(
                &AccountSpec::new(
                    Some(code("1")),
                    vec![section("1"), section("2")],
                    "Long-term Assets".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                ),
                dummy_audit_info(),
            )
            .expect("Already executed");

        let new_ids = chart
            .trial_balance_account_ids_from_new_accounts(&[new_account_set_id])
            .collect::<Vec<_>>();
        assert!(new_ids.contains(&new_account_set_id));
        assert_eq!(new_ids.len(), 1);
    }
}
