use cala_ledger::account::NewAccount;
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use audit::AuditInfo;

use es_entity::*;

use crate::primitives::*;

use super::{error::*, tree};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
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
    ManualTransactionAccountAdded {
        code: AccountCode,
        ledger_account_id: LedgerAccountId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
    pub name: String,
    all_accounts: HashMap<AccountCode, AccountDetails>,
    manual_transaction_accounts: HashMap<LedgerAccountId, AccountCode>,

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
            self.all_accounts.get(parent).map(
                |AccountDetails {
                     account_set_id: id, ..
                 }| *id,
            )
        } else {
            None
        };
        self.all_accounts.insert(
            spec.code.clone(),
            AccountDetails {
                spec: spec.clone(),
                account_set_id: ledger_account_set_id,
                manual_transaction_account_id: None,
            },
        );
        Idempotent::Executed((parent, ledger_account_set_id))
    }

    pub fn trial_balance_account_ids_from_new_accounts(
        &self,
        new_account_set_ids: &[CalaAccountSetId],
    ) -> impl Iterator<Item = CalaAccountSetId> {
        self.all_accounts
            .values()
            .filter(
                move |AccountDetails {
                          spec,
                          account_set_id: id,
                          ..
                      }| {
                    spec.code.len_sections() == 2 && new_account_set_ids.contains(id)
                },
            )
            .map(
                |AccountDetails {
                     account_set_id: id, ..
                 }| *id,
            )
    }

    fn account_spec(&self, code: &AccountCode) -> Option<&AccountDetails> {
        self.all_accounts.get(code)
    }

    /// Returns ancestors of this chart of accounts, starting with `code` (not included).
    /// The lower in hierarchy the parent is, the lower index it will have in the resulting vector;
    /// the root of the chart of accounts will be last.
    pub fn ancestors<T: From<CalaAccountSetId>>(&self, code: &AccountCode) -> Vec<T> {
        let mut result = Vec::new();
        let mut current_code = code;

        if let Some(AccountDetails { spec, .. }) = self.all_accounts.get(current_code) {
            current_code = match &spec.parent {
                Some(parent_code) => parent_code,
                None => return result,
            };
        } else {
            return result;
        }

        while let Some(AccountDetails {
            spec,
            account_set_id,
            ..
        }) = self.all_accounts.get(current_code)
        {
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
            .filter(|AccountDetails { spec, .. }| spec.parent.as_ref() == Some(code))
            .map(|AccountDetails { account_set_id, .. }| T::from(*account_set_id))
            .collect()
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<CalaAccountSetId, ChartOfAccountsError> {
        self.account_spec(code)
            .map(
                |AccountDetails {
                     account_set_id: id, ..
                 }| *id,
            )
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))
    }

    pub fn check_can_have_manual_transactions(
        &self,
        code: &AccountCode,
    ) -> Result<(), ChartOfAccountsError> {
        if !self.children::<CalaAccountSetId>(code).is_empty() {
            return Err(ChartOfAccountsError::NonLeafAccount(code.to_string()));
        };

        Ok(())
    }

    fn manual_transaction_account_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<(CalaAccountSetId, Option<LedgerAccountId>), ChartOfAccountsError> {
        let account_details = self
            .account_spec(code)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))?;

        self.check_can_have_manual_transactions(code)?;

        let AccountDetails {
            account_set_id,
            manual_transaction_account_id,
            ..
        } = account_details;

        Ok((*account_set_id, *manual_transaction_account_id))
    }

    pub fn manual_transaction_account(
        &mut self,
        account_id_or_code: AccountIdOrCode,
        audit_info: AuditInfo,
    ) -> Result<ManualAccountFromChart, ChartOfAccountsError> {
        match account_id_or_code {
            AccountIdOrCode::Id(id) => Ok(match self.manual_transaction_accounts.get(&id) {
                Some(code) => {
                    self.check_can_have_manual_transactions(code)?;

                    ManualAccountFromChart::IdInChart(id)
                }
                None => ManualAccountFromChart::NonChartId(id),
            }),
            AccountIdOrCode::Code(code) => {
                let manual_account = match self.manual_transaction_account_id_from_code(&code)? {
                    (_, Some(id)) => ManualAccountFromChart::IdInChart(id),
                    (account_set_id, None) => {
                        let id = LedgerAccountId::new();
                        self.events.push(ChartEvent::ManualTransactionAccountAdded {
                            code: code.clone(),
                            ledger_account_id: id,
                            audit_info,
                        });

                        if let Some(AccountDetails {
                            manual_transaction_account_id,
                            ..
                        }) = self.all_accounts.get_mut(&code)
                        {
                            *manual_transaction_account_id = Some(id);
                        }
                        self.manual_transaction_accounts.insert(id, code.clone());

                        ManualAccountFromChart::NewAccount((
                            account_set_id,
                            NewAccount::builder()
                                .name(format!("{code} Manual"))
                                .id(id)
                                .code(code.manual_account_external_id(self.id))
                                .external_id(code.manual_account_external_id(self.id))
                                .build()
                                .unwrap(),
                        ))
                    }
                };

                Ok(manual_account)
            }
        }
    }

    pub fn chart(&self) -> tree::ChartTree {
        tree::project(self.events.iter_all())
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();
        let mut all_accounts = HashMap::new();
        let mut manual_transaction_accounts = HashMap::new();
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
                    all_accounts.insert(
                        spec.code.clone(),
                        AccountDetails {
                            spec: spec.clone(),
                            account_set_id: *ledger_account_set_id,
                            manual_transaction_account_id: None,
                        },
                    );
                }
                ChartEvent::ManualTransactionAccountAdded {
                    code,
                    ledger_account_id,
                    ..
                } => {
                    manual_transaction_accounts.insert(*ledger_account_id, code.clone());

                    if let Some(AccountDetails {
                        manual_transaction_account_id,
                        ..
                    }) = all_accounts.get_mut(code)
                    {
                        *manual_transaction_account_id = Some(*ledger_account_id);
                    }
                }
            }
        }
        builder
            .all_accounts(all_accounts)
            .manual_transaction_accounts(manual_transaction_accounts)
            .events(events)
            .build()
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

#[derive(Debug)]
pub enum ManualAccountFromChart {
    IdInChart(LedgerAccountId),
    NonChartId(LedgerAccountId),
    NewAccount((CalaAccountSetId, NewAccount)),
}

struct AccountDetails {
    spec: AccountSpec,
    account_set_id: CalaAccountSetId,
    manual_transaction_account_id: Option<LedgerAccountId>,
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

    #[test]
    fn manual_transaction_account_by_id_non_chart_id() {
        let mut chart = chart_from(initial_events());
        let random_id = LedgerAccountId::new();

        let id = match chart
            .manual_transaction_account(AccountIdOrCode::Id(random_id), dummy_audit_info())
            .unwrap()
        {
            ManualAccountFromChart::NonChartId(id) => id,
            _ => panic!("expected NonChartId"),
        };
        assert_eq!(id, random_id);
    }

    #[test]
    fn manual_transaction_account_by_code_new_account() {
        let (mut chart, (_l1, _l2, level_3_set_id)) = default_chart();
        let acct_code = code("1.1.1");
        let before_count = chart.events.iter_all().count();

        let (account_set_id, new_account) = match chart
            .manual_transaction_account(
                AccountIdOrCode::Code(acct_code.clone()),
                dummy_audit_info(),
            )
            .unwrap()
        {
            ManualAccountFromChart::NewAccount((account_set_id, new_account)) => {
                (account_set_id, new_account)
            }
            _ => panic!("expected NewAccount"),
        };

        assert_eq!(account_set_id, level_3_set_id);
        assert!(
            chart
                .manual_transaction_accounts
                .contains_key(&new_account.id.into())
        );

        let events: Vec<_> = chart.events.iter_all().collect();
        assert_eq!(events.len(), before_count + 1);

        let (code, ledger_account_id) = match events.last().unwrap() {
            ChartEvent::ManualTransactionAccountAdded {
                code,
                ledger_account_id,
                ..
            } => (code.clone(), *ledger_account_id),
            _ => panic!("expected ManualTransactionAccountAdded"),
        };
        assert_eq!(code, acct_code);
        assert_eq!(ledger_account_id, new_account.id.into());
    }

    #[test]
    fn manual_transaction_account_by_code_existing_account() {
        let (mut chart, _) = default_chart();
        let acct_code = code("1.1.1");

        let first = chart
            .manual_transaction_account(
                AccountIdOrCode::Code(acct_code.clone()),
                dummy_audit_info(),
            )
            .unwrap();
        let ledger_id = match first {
            ManualAccountFromChart::NewAccount((_, new_account)) => new_account.id,
            _ => panic!("expected NewAccount"),
        };

        let second = chart
            .manual_transaction_account(
                AccountIdOrCode::Code(acct_code.clone()),
                dummy_audit_info(),
            )
            .unwrap();
        match second {
            ManualAccountFromChart::IdInChart(id) => assert_eq!(id, ledger_id.into()),
            other => panic!("expected IdInChart, got {:?}", other),
        }
    }

    #[test]
    fn manual_transaction_account_by_id_in_chart() {
        let (mut chart, _) = default_chart();
        let acct_code = code("1.1.1");

        let ManualAccountFromChart::NewAccount((_, new_account)) = chart
            .manual_transaction_account(
                AccountIdOrCode::Code(acct_code.clone()),
                dummy_audit_info(),
            )
            .unwrap()
        else {
            panic!("expected NewAccount");
        };

        let ledger_id = LedgerAccountId::from(new_account.id);
        let id = match chart
            .manual_transaction_account(AccountIdOrCode::Id(ledger_id), dummy_audit_info())
            .unwrap()
        {
            ManualAccountFromChart::IdInChart(id) => id,
            _ => panic!("expected IdInChart"),
        };
        assert_eq!(id, ledger_id)
    }

    #[test]
    fn manual_transaction_account_code_not_found() {
        let mut chart = chart_from(initial_events());
        let bad_code = code("9.9.9");

        let err = chart
            .manual_transaction_account(AccountIdOrCode::Code(bad_code.clone()), dummy_audit_info())
            .unwrap_err();

        match err {
            ChartOfAccountsError::CodeNotFoundInChart(c) => assert_eq!(c, bad_code),
            other => panic!("expected CodeNotFoundInChart, got {:?}", other),
        }
    }

    #[test]
    fn manual_transaction_non_leaf_code() {
        let (mut chart, _) = default_chart();
        let acct_code = code("1.1");

        let res = chart.manual_transaction_account(
            AccountIdOrCode::Code(acct_code.clone()),
            dummy_audit_info(),
        );
        assert!(matches!(res, Err(ChartOfAccountsError::NonLeafAccount(_))));
    }

    #[test]
    fn manual_transaction_non_leaf_account_id_in_chart() {
        let (mut chart, _) = default_chart();
        let random_id = LedgerAccountId::new();
        let acct_code = code("1.1");
        chart
            .manual_transaction_accounts
            .insert(random_id, acct_code);

        let res =
            chart.manual_transaction_account(AccountIdOrCode::Id(random_id), dummy_audit_info());
        assert!(matches!(res, Err(ChartOfAccountsError::NonLeafAccount(_))));
    }
}
