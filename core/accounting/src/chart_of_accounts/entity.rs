use cala_ledger::{account::NewAccount, account_set::NewAccountSet};
use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::chart_node::entity::*;
use crate::primitives::*;

use super::{error::*, tree};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ChartId")]
pub enum ChartEvent {
    Initialized {
        id: ChartId,
        account_set_id: CalaAccountSetId,
        name: String,
        reference: String,
        first_period_opened_as_of: chrono::NaiveDate,
        first_period_opened_at: DateTime<Utc>,
    },
    AccountingPeriodClosed {
        effective: chrono::NaiveDate,
        recorded_at: DateTime<Utc>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub account_set_id: CalaAccountSetId,
    pub reference: String,
    pub name: String,
    pub closing: AccountingClosing,

    events: EntityEvents<ChartEvent>,

    #[es_entity(nested)]
    #[builder(default)]
    chart_nodes: Nested<ChartNode>,
}

impl Chart {
    pub(super) fn create_node(
        &mut self,
        spec: &AccountSpec,
        journal_id: CalaJournalId,
    ) -> Idempotent<NewChartAccountDetails> {
        if self.get_node_details_by_code(&spec.code).is_some() {
            return Idempotent::Ignored;
        }

        let node_id = ChartNodeId::new();
        let ledger_account_set_id = CalaAccountSetId::new();

        let new_chart_node = NewChartNode {
            id: node_id,
            chart_id: self.id,
            spec: spec.clone(),
            ledger_account_set_id,
        };

        self.chart_nodes.add_new(new_chart_node);

        let parent_account_set_id = if let Some(parent) = spec.parent.as_ref() {
            self.get_node_details_by_code(parent)
                .map(|details| details.account_set_id)
                .expect("Parent was not added to all_accounts as yet")
        } else {
            self.account_set_id
        };

        let new_account_set = NewAccountSet::builder()
            .id(ledger_account_set_id)
            .journal_id(journal_id)
            .name(spec.name.to_string())
            .description(spec.name.to_string())
            .external_id(spec.code.account_set_external_id(self.id))
            .normal_balance_type(spec.normal_balance_type)
            .build()
            .expect("Could not build new account set");

        Idempotent::Executed(NewChartAccountDetails {
            new_account_set,
            parent_account_set_id,
        })
    }

    pub(super) fn create_child_node(
        &mut self,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
        journal_id: CalaJournalId,
    ) -> Result<Idempotent<NewChartAccountDetails>, ChartOfAccountsError> {
        let parent_normal_balance_type = self
            .get_node_details_by_code(&parent_code)
            .map(|details| details.spec.normal_balance_type)
            .ok_or(ChartOfAccountsError::ParentAccountNotFound(
                parent_code.to_string(),
            ))?;

        let spec = AccountSpec::try_new(
            Some(parent_code),
            code.into(),
            name,
            parent_normal_balance_type,
        )?;

        Ok(self.create_node(&spec, journal_id))
    }

    pub(super) fn trial_balance_account_ids_from_new_accounts(
        &self,
        new_account_set_ids: &[CalaAccountSetId],
    ) -> impl Iterator<Item = CalaAccountSetId> {
        self.chart_nodes
            .iter_persisted()
            .filter(move |node| {
                node.spec.code.len_sections() == 2
                    && new_account_set_ids.contains(&node.account_set_id)
            })
            .map(move |node| node.account_set_id)
    }

    pub(super) fn trial_balance_account_id_from_new_account(
        &self,
        new_account_set_id: CalaAccountSetId,
    ) -> Option<CalaAccountSetId> {
        self.chart_nodes.iter_persisted().find_map(|node| {
            if node.spec.code.len_sections() == 2 && new_account_set_id == node.account_set_id {
                Some(node.account_set_id)
            } else {
                None
            }
        })
    }

    /// Returns ancestors, in this chart of accounts, of an account with `code` (not included).
    /// The lower in hierarchy the parent is, the lower index it will have in the resulting vector;
    /// the root of the chart of accounts will be last.
    pub fn ancestors<T: From<CalaAccountSetId>>(&self, code: &AccountCode) -> Vec<T> {
        let mut result = Vec::new();
        let mut current_code = code.clone();

        if let Some(node) = self.get_node_details_by_code(&current_code) {
            current_code = match node.spec.parent {
                Some(parent_code) => parent_code,
                None => return result,
            };
        } else {
            return result;
        }

        while let Some(node) = self.get_node_details_by_code(&current_code) {
            result.push(T::from(node.account_set_id));
            match node.spec.parent {
                Some(parent_code) => current_code = parent_code,
                None => break,
            }
        }

        result
    }

    /// Returns direct children, in this chart of accounts, of an account with `code` (not included).
    /// No particular order of the children is guaranteed.
    pub fn children(
        &self,
        code: &AccountCode,
    ) -> impl Iterator<Item = (&AccountCode, CalaAccountSetId)> {
        self.chart_nodes.iter_persisted().filter_map(move |node| {
            if node.spec.parent.as_ref() == Some(code) {
                Some((&node.spec.code, node.account_set_id))
            } else {
                None
            }
        })
    }

    fn get_node_details_by_code(&self, code: &AccountCode) -> Option<ChartNodeDetails> {
        self.chart_nodes
            .find_map_persisted(|node| (node.spec.code == *code).then(|| node.into()))
            .or_else(|| {
                self.chart_nodes
                    .find_map_new(|node| (node.spec.code == *code).then(|| node.into()))
            })
    }

    fn get_node_by_code_mut(&mut self, code: &AccountCode) -> Option<&mut ChartNode> {
        self.chart_nodes
            .iter_persisted_mut()
            .find(|node| node.spec.code == *code)
    }

    fn get_node_by_manual_transaction_account_id(
        &self,
        id: &LedgerAccountId,
    ) -> Option<&ChartNode> {
        self.chart_nodes
            .iter_persisted()
            .find(|node| node.manual_transaction_account_id == Some(*id))
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<CalaAccountSetId, ChartOfAccountsError> {
        self.get_node_details_by_code(code)
            .map(|details| details.account_set_id)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))
    }

    pub fn check_can_have_manual_transactions(
        &self,
        code: &AccountCode,
    ) -> Result<(), ChartOfAccountsError> {
        match self.children(code).next() {
            None => Ok(()),
            _ => Err(ChartOfAccountsError::NonLeafAccount(code.to_string())),
        }
    }

    pub fn manual_transaction_account(
        &mut self,
        account_id_or_code: AccountIdOrCode,
    ) -> Result<ManualAccountFromChart, ChartOfAccountsError> {
        match account_id_or_code {
            AccountIdOrCode::Id(id) => {
                Ok(match self.get_node_by_manual_transaction_account_id(&id) {
                    Some(node) => {
                        self.check_can_have_manual_transactions(&node.spec.code)?;
                        ManualAccountFromChart::IdInChart(id)
                    }
                    None => ManualAccountFromChart::NonChartId(id),
                })
            }
            AccountIdOrCode::Code(code) => {
                let node = self
                    .get_node_details_by_code(&code)
                    .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))?;

                self.check_can_have_manual_transactions(&code)?;

                if let Some(existing_id) = node.manual_transaction_account_id {
                    return Ok(ManualAccountFromChart::IdInChart(existing_id));
                }

                let node_mut = self.get_node_by_code_mut(&code).expect("Node should exist");
                match node_mut.assign_manual_transaction_account() {
                    Idempotent::Executed(new_account) => Ok(ManualAccountFromChart::NewAccount((
                        node_mut.account_set_id,
                        new_account,
                    ))),
                    Idempotent::Ignored => Ok(ManualAccountFromChart::IdInChart(
                        node_mut
                            .manual_transaction_account_id
                            .expect("Manual transaction account id should be set"),
                    )),
                }
            }
        }
    }

    pub fn close_last_monthly_period(
        &mut self,
    ) -> Result<Idempotent<NaiveDate>, ChartOfAccountsError> {
        let last_recorded_date = self.events.iter_all().rev().find_map(|event| match event {
            ChartEvent::AccountingPeriodClosed { effective, .. } => Some(*effective),
            _ => None,
        });
        let new_monthly_closing_date = match last_recorded_date {
            Some(last_effective) => {
                let last_day_of_previous_month = crate::time::now()
                    .date_naive()
                    .with_day(1)
                    .and_then(|d| d.pred_opt())
                    .expect("Failed to compute last day of previous month");
                if last_effective == last_day_of_previous_month {
                    return Ok(Idempotent::Ignored);
                }

                last_effective
                    .checked_add_months(Months::new(2))
                    .and_then(|d| d.with_day(1))
                    .and_then(|d| d.pred_opt())
                    .expect("Failed to compute new monthly closing date")
            }
            None => self
                .events
                .iter_all()
                .find_map(|event| match event {
                    ChartEvent::Initialized {
                        first_period_opened_as_of,
                        ..
                    } => Some(*first_period_opened_as_of),
                    _ => None,
                })
                .ok_or(ChartOfAccountsError::AccountPeriodStartNotFound)?
                .checked_add_months(Months::new(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.pred_opt())
                .expect("Failed to compute new monthly closing date"),
        };

        let recorded_at = crate::time::now();
        self.events.push(ChartEvent::AccountingPeriodClosed {
            effective: new_monthly_closing_date,
            recorded_at,
        });
        self.closing = AccountingClosing::monthly_from(new_monthly_closing_date, recorded_at);

        Ok(Idempotent::Executed(new_monthly_closing_date))
    }

    pub fn chart(&self) -> tree::ChartTree {
        tree::project_from_nodes(self.id, &self.name, self.chart_nodes.iter_persisted())
    }
}

impl TryFromEvents<ChartEvent> for Chart {
    fn try_from_events(events: EntityEvents<ChartEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ChartBuilder::default();

        for event in events.iter_all() {
            match event {
                ChartEvent::Initialized {
                    id,
                    account_set_id,
                    reference,
                    name,
                    first_period_opened_as_of,
                    first_period_opened_at,
                    ..
                } => {
                    let last_monthly_closed_as_of = first_period_opened_as_of
                        .pred_opt()
                        .expect("Failed to get day prior to opening date");
                    let closing = AccountingClosing::monthly_from(
                        last_monthly_closed_as_of,
                        *first_period_opened_at,
                    );

                    builder = builder
                        .id(*id)
                        .account_set_id(*account_set_id)
                        .reference(reference.to_string())
                        .closing(closing)
                        .name(name.to_string());
                }
                ChartEvent::AccountingPeriodClosed {
                    effective,
                    recorded_at,
                } => {
                    builder =
                        builder.closing(AccountingClosing::monthly_from(*effective, *recorded_at))
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewChart {
    #[builder(setter(into))]
    pub(super) id: ChartId,
    #[builder(setter(into))]
    pub(super) account_set_id: CalaAccountSetId,
    pub(super) name: String,
    pub(super) reference: String,
    pub(super) first_period_opened_as_of: chrono::NaiveDate,
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
                account_set_id: self.account_set_id,
                name: self.name,
                reference: self.reference,
                first_period_opened_as_of: self.first_period_opened_as_of,
                first_period_opened_at: crate::time::now(),
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

pub struct NewChartAccountDetails {
    pub new_account_set: NewAccountSet,
    pub parent_account_set_id: CalaAccountSetId,
}

pub struct ChartNodeDetails {
    account_set_id: CalaAccountSetId,
    spec: AccountSpec,
    manual_transaction_account_id: Option<LedgerAccountId>,
}

impl From<&ChartNode> for ChartNodeDetails {
    fn from(node: &ChartNode) -> Self {
        Self {
            account_set_id: node.account_set_id,
            spec: node.spec.clone(),
            manual_transaction_account_id: node.manual_transaction_account_id,
        }
    }
}

impl From<&NewChartNode> for ChartNodeDetails {
    fn from(node: &NewChartNode) -> Self {
        Self {
            account_set_id: node.ledger_account_set_id,
            spec: node.spec.clone(),
            manual_transaction_account_id: None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AccountingClosing {
    pub monthly: Option<PeriodClosing>,
}

impl AccountingClosing {
    fn monthly_from(effective: NaiveDate, recorded_at: DateTime<Utc>) -> Self {
        Self {
            monthly: Some(PeriodClosing {
                closed_as_of: effective,
                closed_at: recorded_at,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodClosing {
    pub closed_as_of: chrono::NaiveDate,
    pub closed_at: DateTime<Utc>,
}

#[cfg(test)]
mod test {

    use super::*;

    fn chart_from(events: Vec<ChartEvent>) -> Chart {
        Chart::try_from_events(EntityEvents::init(ChartId::new(), events)).unwrap()
    }

    fn initial_events_with_opened_date(first_period_opened_as_of: NaiveDate) -> Vec<ChartEvent> {
        vec![ChartEvent::Initialized {
            id: ChartId::new(),
            account_set_id: CalaAccountSetId::new(),
            name: "Test Chart".to_string(),
            reference: "test-chart".to_string(),
            first_period_opened_at: crate::time::now(),
            first_period_opened_as_of,
        }]
    }

    fn initial_events() -> Vec<ChartEvent> {
        initial_events_with_opened_date("2025-01-01".parse::<NaiveDate>().unwrap())
    }

    fn default_chart() -> (
        Chart,
        (CalaAccountSetId, CalaAccountSetId, CalaAccountSetId),
    ) {
        let mut chart = chart_from(initial_events());
        let NewChartAccountDetails {
            new_account_set: level_1_new_account_set,
            ..
        } = chart
            .create_node(
                &AccountSpec::try_new(
                    None,
                    vec![section("1")],
                    "Assets".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                )
                .unwrap(),
                CalaJournalId::new(),
            )
            .expect("Already executed");
        let NewChartAccountDetails {
            new_account_set: level_2_new_account_set,
            ..
        } = chart
            .create_node(
                &AccountSpec::try_new(
                    Some(code("1")),
                    vec![section("1"), section("1")],
                    "Current Assets".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                )
                .unwrap(),
                CalaJournalId::new(),
            )
            .expect("Already executed");
        let NewChartAccountDetails {
            new_account_set: level_3_new_account_set,
            ..
        } = chart
            .create_node(
                &AccountSpec::try_new(
                    Some(code("1.1")),
                    vec![section("1"), section("1"), section("1")],
                    "Cash".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                )
                .unwrap(),
                CalaJournalId::new(),
            )
            .expect("Already executed");
        hydrate_chart_of_accounts(&mut chart);
        (
            chart,
            (
                level_1_new_account_set.id,
                level_2_new_account_set.id,
                level_3_new_account_set.id,
            ),
        )
    }

    fn hydrate_chart_of_accounts(chart: &mut Chart) {
        let new_entities = chart
            .chart_nodes
            .new_entities_mut()
            .drain(..)
            .map(|new| ChartNode::try_from_events(new.into_events()).expect("hydrate failed"))
            .collect::<Vec<_>>();
        chart.chart_nodes.load(new_entities);
    }

    fn section(s: &str) -> AccountCodeSection {
        s.parse::<AccountCodeSection>().unwrap()
    }

    fn code(s: &str) -> AccountCode {
        s.parse::<AccountCode>().unwrap()
    }

    #[test]
    fn errors_for_create_child_node_if_parent_node_does_not_exist() {
        let (mut chart, _) = default_chart();

        let res = chart.create_child_node(
            code("1.9"),
            code("1.9.1"),
            "Cash".parse::<AccountName>().unwrap(),
            CalaJournalId::new(),
        );

        assert!(matches!(
            res,
            Err(ChartOfAccountsError::ParentAccountNotFound(_))
        ))
    }

    #[test]
    #[should_panic]
    fn unchecked_fails_to_create_node_if_parent_node_does_not_exist() {
        let (mut chart, _) = default_chart();

        let _ = chart.create_node(
            &AccountSpec::try_new(
                Some(code("1.9")),
                vec![section("1"), section("9"), section("1")],
                "Cash".parse::<AccountName>().unwrap(),
                DebitOrCredit::Debit,
            )
            .unwrap(),
            CalaJournalId::new(),
        );
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

        let NewChartAccountDetails {
            new_account_set:
                NewAccountSet {
                    id: new_account_set_id,
                    ..
                },
            ..
        } = chart
            .create_node(
                &AccountSpec::try_new(
                    Some(code("1")),
                    vec![section("1"), section("2")],
                    "Long-term Assets".parse::<AccountName>().unwrap(),
                    DebitOrCredit::Debit,
                )
                .unwrap(),
                CalaJournalId::new(),
            )
            .expect("Already executed");
        hydrate_chart_of_accounts(&mut chart);
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
            .manual_transaction_account(AccountIdOrCode::Id(random_id))
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

        let (account_set_id, new_account) = match chart
            .manual_transaction_account(AccountIdOrCode::Code(acct_code.clone()))
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
                .get_node_by_manual_transaction_account_id(&new_account.id.into())
                .is_some()
        );

        let node = chart
            .get_node_by_manual_transaction_account_id(&new_account.id.into())
            .unwrap();

        assert_eq!(node.spec.code, acct_code);
        assert_eq!(
            node.manual_transaction_account_id.unwrap(),
            new_account.id.into()
        );
    }

    #[test]
    fn manual_transaction_account_by_code_existing_account() {
        let (mut chart, _) = default_chart();
        let acct_code = code("1.1.1");

        let first = chart
            .manual_transaction_account(AccountIdOrCode::Code(acct_code.clone()))
            .unwrap();
        let ledger_id = match first {
            ManualAccountFromChart::NewAccount((_, new_account)) => new_account.id,
            _ => panic!("expected NewAccount"),
        };

        let second = chart
            .manual_transaction_account(AccountIdOrCode::Code(acct_code.clone()))
            .unwrap();
        match second {
            ManualAccountFromChart::IdInChart(id) => assert_eq!(id, ledger_id.into()),
            other => panic!("expected IdInChart, got {other:?}"),
        }
    }

    #[test]
    fn manual_transaction_account_by_id_in_chart() {
        let (mut chart, _) = default_chart();
        let acct_code = code("1.1.1");

        let ManualAccountFromChart::NewAccount((_, new_account)) = chart
            .manual_transaction_account(AccountIdOrCode::Code(acct_code.clone()))
            .unwrap()
        else {
            panic!("expected NewAccount");
        };

        let ledger_id = LedgerAccountId::from(new_account.id);
        let id = match chart
            .manual_transaction_account(AccountIdOrCode::Id(ledger_id))
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
            .manual_transaction_account(AccountIdOrCode::Code(bad_code.clone()))
            .unwrap_err();

        match err {
            ChartOfAccountsError::CodeNotFoundInChart(c) => assert_eq!(c, bad_code),
            other => panic!("expected CodeNotFoundInChart, got {other:?}"),
        }
    }

    #[test]
    fn manual_transaction_non_leaf_code() {
        let (mut chart, _) = default_chart();
        let acct_code = code("1.1");

        let res = chart.manual_transaction_account(AccountIdOrCode::Code(acct_code.clone()));
        assert!(matches!(res, Err(ChartOfAccountsError::NonLeafAccount(_))));
    }

    #[test]
    fn manual_transaction_non_leaf_account_id_in_chart() {
        let (mut chart, _) = default_chart();
        let random_id = LedgerAccountId::new();
        chart
            .get_node_by_code_mut(&code("1.1"))
            .unwrap()
            .manual_transaction_account_id = Some(random_id);

        let res = chart.manual_transaction_account(AccountIdOrCode::Id(random_id));
        assert!(matches!(res, Err(ChartOfAccountsError::NonLeafAccount(_))));
    }

    #[test]
    fn test_project_chart_structure() {
        let chart = default_chart().0;
        let tree = chart.chart();

        assert_eq!(tree.id, chart.id);
        assert_eq!(tree.name, chart.name);
        assert_eq!(tree.children.len(), 1);

        let assets = &tree.children[0];
        assert_eq!(assets.code, AccountCode::new(vec!["1".parse().unwrap()]));
        assert_eq!(assets.children.len(), 1);

        let current_assets = &assets.children[0];
        assert_eq!(
            current_assets.code,
            AccountCode::new(["1", "1"].iter().map(|c| c.parse().unwrap()).collect())
        );
        assert_eq!(current_assets.children.len(), 1);

        let cash = &current_assets.children[0];
        assert_eq!(
            cash.code,
            AccountCode::new(["1", "1", "1"].iter().map(|c| c.parse().unwrap()).collect())
        );
        assert!(cash.children.is_empty());
    }

    mod close_monthly {
        use super::*;

        #[test]
        fn last_monthly_closed_as_of_set_after_open_first_accounting_period() {
            let starts_at = "2024-01-15".parse::<NaiveDate>().unwrap();
            let expected_last_closed = "2024-01-14".parse::<NaiveDate>().unwrap();

            let chart = chart_from(initial_events_with_opened_date(starts_at));
            assert_eq!(
                chart.closing.monthly.unwrap().closed_as_of,
                expected_last_closed
            );
        }

        #[test]
        fn close_last_monthly_period_first_time() {
            let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
            let expected_closed_date = "2024-01-31".parse::<NaiveDate>().unwrap();
            let mut chart = chart_from(initial_events_with_opened_date(period_start));

            let closed_date = chart.close_last_monthly_period().unwrap().unwrap();
            assert_eq!(closed_date, expected_closed_date);
            assert_eq!(
                chart.closing.monthly.unwrap().closed_as_of,
                expected_closed_date
            );

            let closing_event_date = chart
                .events
                .iter_all()
                .rev()
                .find_map(|e| match e {
                    ChartEvent::AccountingPeriodClosed { effective, .. } => Some(*effective),
                    _ => None,
                })
                .unwrap();
            assert_eq!(closing_event_date, expected_closed_date);
        }

        #[test]
        fn close_last_monthly_period_after_prior_closes() {
            let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
            let expected_second_closed_date = "2024-02-29".parse::<NaiveDate>().unwrap();
            let mut chart = chart_from(initial_events_with_opened_date(period_start));

            let _ = chart.close_last_monthly_period().unwrap();

            let second_closing_date = chart.close_last_monthly_period().unwrap().unwrap();
            assert_eq!(second_closing_date, expected_second_closed_date);
            assert_eq!(
                chart.closing.monthly.unwrap().closed_as_of,
                expected_second_closed_date
            );
        }

        #[test]
        fn close_last_monthly_ignored_for_current_month() {
            let first_day_of_last_month = chrono::Utc::now()
                .date_naive()
                .checked_sub_months(chrono::Months::new(1))
                .and_then(|d| d.with_day(1))
                .unwrap();
            let mut chart = chart_from(initial_events_with_opened_date(first_day_of_last_month));
            let _ = chart.close_last_monthly_period().unwrap();

            let second_closing_date = chart.close_last_monthly_period().unwrap();
            assert!(second_closing_date.was_ignored());
        }
    }
}
