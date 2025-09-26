use cala_ledger::{account::NewAccount, account_set::NewAccountSet};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::chart_node::*;
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
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub reference: String,
    pub name: String,

    events: EntityEvents<ChartEvent>,

    #[es_entity(nested)]
    #[builder(default)]
    chart_nodes: Nested<ChartNode>,
}

impl Chart {
    pub(super) fn create_node_with_existing_children(
        &mut self,
        spec: &AccountSpec,
        journal_id: CalaJournalId,
        children_node_ids: Vec<ChartNodeId>,
    ) -> Idempotent<NewAccountSetWithNodeId> {
        if self.get_node_details_by_code(&spec.code).is_some() {
            return Idempotent::Ignored;
        }

        let new_node_id = ChartNodeId::new();
        let chart_node = NewChartNode::builder()
            .id(new_node_id)
            .chart_id(self.id)
            .spec(spec.clone())
            .ledger_account_set_id(CalaAccountSetId::new())
            .children_node_ids(children_node_ids)
            .build()
            .expect("could not build NewChartNode");

        let new_account_set = chart_node.new_account_set(journal_id);
        self.chart_nodes.add_new(chart_node);

        Idempotent::Executed(NewAccountSetWithNodeId {
            new_account_set,
            node_id: new_node_id,
        })
    }

    fn create_node_without_verifying_parent(
        &mut self,
        spec: &AccountSpec,
        journal_id: CalaJournalId,
    ) -> Idempotent<NewChartAccountDetails> {
        if self.get_node_details_by_code(&spec.code).is_some() {
            return Idempotent::Ignored;
        }

        let chart_node = NewChartNode::builder()
            .id(ChartNodeId::new())
            .chart_id(self.id)
            .spec(spec.clone())
            .ledger_account_set_id(CalaAccountSetId::new())
            .build()
            .expect("could not build NewChartNode");

        let parent_account_set_id = spec
            .parent
            .as_ref()
            .and_then(|parent_code| self.get_node_by_code_mut(parent_code))
            .map(|parent_node| {
                let _ = parent_node.add_child_node(chart_node.id);
                parent_node.account_set_id
            });

        let new_account_set = chart_node.new_account_set(journal_id);
        self.chart_nodes.add_new(chart_node);

        Idempotent::Executed(NewChartAccountDetails {
            new_account_set,
            parent_account_set_id,
        })
    }

    pub(super) fn create_root_node(
        &mut self,
        spec: &AccountSpec,
        journal_id: CalaJournalId,
    ) -> Idempotent<NewChartAccountDetails> {
        self.create_node_without_verifying_parent(spec, journal_id)
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

        Ok(self.create_node_without_verifying_parent(&spec, journal_id))
    }

    pub(super) fn trial_balance_account_ids_from_new_accounts(
        &self,
        new_account_set_ids: &[CalaAccountSetId],
    ) -> impl Iterator<Item = CalaAccountSetId> {
        self.chart_nodes
            .iter_persisted()
            .filter(move |node| {
                node.is_trial_balance_account()
                    && new_account_set_ids.contains(&node.account_set_id)
            })
            .map(move |node| node.account_set_id)
    }

    pub(super) fn trial_balance_account_id_from_new_account(
        &self,
        new_account_set_id: CalaAccountSetId,
    ) -> Option<CalaAccountSetId> {
        self.chart_nodes.find_map_persisted(|node| {
            if node.is_trial_balance_account() && new_account_set_id == node.account_set_id {
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
        let mut current = self.get_node_details_by_code(code);

        while let Some(node) = current {
            if let Some(parent_node) = node
                .spec
                .parent
                .as_ref()
                .and_then(|p| self.get_node_details_by_code(p))
            {
                result.push(T::from(parent_node.account_set_id));
                current = Some(parent_node);
            } else {
                break;
            }
        }
        result
    }

    /// Returns direct children, in this chart of accounts, of an account with `code` (not included).
    /// No particular order of the children is guaranteed.
    pub fn children(
        &self,
        code: &AccountCode,
    ) -> impl Iterator<Item = (AccountCode, CalaAccountSetId)> {
        self.get_node_by_code(code)
            .into_iter()
            .flat_map(move |node| {
                node.children().filter_map(move |child_node_id| {
                    let child_node = self.chart_nodes.get_persisted(child_node_id)?;
                    Some((child_node.spec.code.clone(), child_node.account_set_id))
                })
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

    fn get_node_by_code(&self, code: &AccountCode) -> Option<&ChartNode> {
        self.chart_nodes
            .iter_persisted()
            .find(|node| node.spec.code == *code)
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<CalaAccountSetId, ChartOfAccountsError> {
        self.get_node_details_by_code(code)
            .map(|details| details.account_set_id)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))
    }

    pub fn manual_transaction_account(
        &mut self,
        account_id_or_code: AccountIdOrCode,
    ) -> Result<ManualAccountFromChart, ChartOfAccountsError> {
        match account_id_or_code {
            AccountIdOrCode::Id(id) => {
                let res = match self
                    .chart_nodes
                    .find_persisted(|node| node.manual_transaction_account_id == Some(id))
                {
                    Some(node) => {
                        if node.can_have_manual_transactions() {
                            ManualAccountFromChart::IdInChart(id)
                        } else {
                            return Err(ChartOfAccountsError::NonLeafAccount(
                                node.spec.code.to_string(),
                            ));
                        }
                    }
                    None => ManualAccountFromChart::NonChartId(id),
                };

                Ok(res)
            }
            AccountIdOrCode::Code(code) => {
                let node = self
                    .chart_nodes
                    .find_persisted_mut(|node| node.spec.code == code)
                    .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))?;

                match node.assign_manual_transaction_account()? {
                    Idempotent::Executed(new_account) => Ok(ManualAccountFromChart::NewAccount((
                        node.account_set_id,
                        new_account,
                    ))),
                    Idempotent::Ignored => Ok(ManualAccountFromChart::IdInChart(
                        node.manual_transaction_account_id
                            .expect("Manual transaction account id should be set"),
                    )),
                }
            }
        }
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
                    reference,
                    name,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .reference(reference.to_string())
                        .name(name.to_string());
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
    pub(super) name: String,
    pub(super) reference: String,
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
    pub parent_account_set_id: Option<CalaAccountSetId>,
}

pub struct NewAccountSetWithNodeId {
    pub new_account_set: NewAccountSet,
    pub node_id: ChartNodeId,
}

pub struct ChartNodeDetails {
    account_set_id: CalaAccountSetId,
    spec: AccountSpec,
}

impl From<&ChartNode> for ChartNodeDetails {
    fn from(node: &ChartNode) -> Self {
        Self {
            account_set_id: node.account_set_id,
            spec: node.spec.clone(),
        }
    }
}

impl From<&NewChartNode> for ChartNodeDetails {
    fn from(node: &NewChartNode) -> Self {
        Self {
            account_set_id: node.ledger_account_set_id,
            spec: node.spec.clone(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn chart_from(events: Vec<ChartEvent>) -> Chart {
        Chart::try_from_events(EntityEvents::init(ChartId::new(), events)).unwrap()
    }

    fn initial_events() -> Vec<ChartEvent> {
        vec![ChartEvent::Initialized {
            id: ChartId::new(),
            name: "Test Chart".to_string(),
            reference: "test-chart".to_string(),
        }]
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
            .create_node_without_verifying_parent(
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
        hydrate_chart_of_accounts(&mut chart);
        let NewChartAccountDetails {
            new_account_set: level_2_new_account_set,
            ..
        } = chart
            .create_node_without_verifying_parent(
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
        hydrate_chart_of_accounts(&mut chart);
        let NewChartAccountDetails {
            new_account_set: level_3_new_account_set,
            ..
        } = chart
            .create_node_without_verifying_parent(
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
    fn unchecked_creates_node_if_parent_node_does_not_exist() {
        let (mut chart, _) = default_chart();

        let res = chart.create_node_without_verifying_parent(
            &AccountSpec::try_new(
                Some(code("1.9")),
                vec![section("1"), section("9"), section("1")],
                "Cash".parse::<AccountName>().unwrap(),
                DebitOrCredit::Debit,
            )
            .unwrap(),
            CalaJournalId::new(),
        );
        assert!(res.did_execute());
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
            .create_node_without_verifying_parent(
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

        let node = chart
            .chart_nodes
            .find_persisted(|node| {
                node.manual_transaction_account_id == Some(new_account.id.into())
            })
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
}
