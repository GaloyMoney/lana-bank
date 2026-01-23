use cala_ledger::{account::NewAccount, account_set::NewAccountSet};
use chrono::NaiveDate;
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::chart_node::*;
use crate::{
    chart_of_accounts::ledger::ClosingTxParentIdsAndDetails,
    primitives::{AccountingBaseConfig, *},
};

use super::{bulk_import::*, error::*, tree};

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
    },
    BaseConfigSet {
        base_config: AccountingBaseConfig,
    },
    ClosedAsOf {
        closed_as_of: NaiveDate,
    },
    ClosingTransactionPosted {
        posted_as_of: NaiveDate,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Chart {
    pub id: ChartId,
    pub account_set_id: CalaAccountSetId,
    pub reference: String,
    pub name: String,
    #[builder(default)]
    pub base_config: Option<AccountingBaseConfig>,

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
        if self.find_node_details_by_code(&spec.code).is_some() {
            return Idempotent::AlreadyApplied;
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
        if self.find_node_details_by_code(&spec.code).is_some() {
            return Idempotent::AlreadyApplied;
        }

        let node_id = ChartNodeId::new();
        let ledger_account_set_id = CalaAccountSetId::new();

        let chart_node = NewChartNode {
            id: node_id,
            chart_id: self.id,
            spec: spec.clone(),
            ledger_account_set_id,
            children_node_ids: vec![],
        };

        let parent_account_set_id = spec
            .parent
            .as_ref()
            .and_then(|parent_code| {
                self.chart_nodes
                    .find_persisted_mut(|node| node.spec.code == *parent_code)
            })
            .map(|parent_node| {
                let _ = parent_node.add_child_node(chart_node.id);
                parent_node.account_set_id
            })
            .unwrap_or(self.account_set_id);

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

    pub(super) fn configure_with_initial_accounts(
        &mut self,
        account_specs: Vec<AccountSpec>,
        base_config: AccountingBaseConfig,
        journal_id: CalaJournalId,
    ) -> Result<Idempotent<BulkImportResult>, ChartOfAccountsError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ChartEvent::BaseConfigSet { base_config: existing, .. } if &base_config == existing,
        );
        if self.base_config.is_some() {
            return Err(ChartOfAccountsError::BaseConfigAlreadyInitializedWithDifferentConfig);
        }

        let res = BulkAccountImport::new(self, journal_id).import(account_specs);

        self.check_base_config_codes_exists_in_chart(&base_config)?;
        self.events.push(ChartEvent::BaseConfigSet {
            base_config: base_config.clone(),
        });
        self.base_config = Some(base_config);

        Ok(Idempotent::Executed(res))
    }

    pub(super) fn create_child_node(
        &mut self,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
        journal_id: CalaJournalId,
    ) -> Result<Idempotent<NewChartAccountDetails>, ChartOfAccountsError> {
        let parent_normal_balance_type = self
            .find_node_details_by_code(&parent_code)
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
        let mut current = self.find_node_details_by_code(code);

        while let Some(node) = current {
            if let Some(parent_node) = node
                .spec
                .parent
                .as_ref()
                .and_then(|p| self.find_node_details_by_code(p))
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
        self.chart_nodes
            .find_persisted(|node| node.spec.code == *code)
            .into_iter()
            .flat_map(move |node| {
                node.children().filter_map(move |child_node_id| {
                    let child_node = self.chart_nodes.get_persisted(child_node_id)?;
                    Some((child_node.spec.code.clone(), child_node.account_set_id))
                })
            })
    }

    fn find_node_details_by_code(&self, code: &AccountCode) -> Option<ChartNodeDetails> {
        self.chart_nodes
            .find_map_persisted(|node| (node.spec.code == *code).then(|| node.into()))
            .or_else(|| {
                self.chart_nodes
                    .find_map_new(|node| (node.spec.code == *code).then(|| node.into()))
            })
    }

    pub fn account_set_id_from_code(
        &self,
        code: &AccountCode,
    ) -> Result<CalaAccountSetId, ChartOfAccountsError> {
        self.find_node_details_by_code(code)
            .map(|details| details.account_set_id)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))
    }

    pub fn maybe_account_set_id_from_code(&self, code: &AccountCode) -> Option<CalaAccountSetId> {
        self.find_node_details_by_code(code)
            .map(|details| details.account_set_id)
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
                        // Need to re-check eligibility because
                        // incase it now has children but didn't previously
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
                    Idempotent::AlreadyApplied => Ok(ManualAccountFromChart::IdInChart(
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

    pub fn accounting_base_config(&self) -> Option<AccountingBaseConfig> {
        self.base_config.clone()
    }

    fn check_base_config_codes_exists_in_chart(
        &self,
        base_config: &AccountingBaseConfig,
    ) -> Result<(), ChartOfAccountsError> {
        self.check_top_level_account_code(&base_config.assets_code)?;
        self.check_top_level_account_code(&base_config.liabilities_code)?;
        self.check_top_level_account_code(&base_config.equity_code)?;
        self.check_top_level_account_code(&base_config.revenue_code)?;
        self.check_top_level_account_code(&base_config.cost_of_revenue_code)?;
        self.check_top_level_account_code(&base_config.expenses_code)?;

        self.find_node_details_by_code(&base_config.equity_retained_earnings_gain_code)
            .ok_or_else(|| {
                ChartOfAccountsError::CodeNotFoundInChart(
                    base_config.equity_retained_earnings_gain_code.clone(),
                )
            })?;
        self.find_node_details_by_code(&base_config.equity_retained_earnings_loss_code)
            .ok_or_else(|| {
                ChartOfAccountsError::CodeNotFoundInChart(
                    base_config.equity_retained_earnings_loss_code.clone(),
                )
            })?;

        Ok(())
    }

    fn check_top_level_account_code(&self, code: &AccountCode) -> Result<(), ChartOfAccountsError> {
        let details = self
            .find_node_details_by_code(code)
            .ok_or_else(|| ChartOfAccountsError::CodeNotFoundInChart(code.clone()))?;

        if details.spec.parent.is_some() {
            return Err(ChartOfAccountsError::AccountCodeHasInvalidParent(
                code.to_string(),
            ));
        }

        Ok(())
    }

    pub(super) fn close_as_of(&mut self, closed_as_of: NaiveDate) -> Idempotent<NaiveDate> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            ChartEvent::ClosedAsOf { closed_as_of: prev_date, .. } if prev_date >= &closed_as_of,
            => ChartEvent::ClosedAsOf { .. }
        );
        self.events.push(ChartEvent::ClosedAsOf { closed_as_of });
        Idempotent::Executed(closed_as_of)
    }

    fn closing_account_set_ids_from_codes(
        &self,
        account_codes: ClosingAccountCodes,
    ) -> Result<ClosingAccountSetIds, ChartOfAccountsError> {
        Ok(ClosingAccountSetIds {
            revenue: self.account_set_id_from_code(&account_codes.revenue)?,
            cost_of_revenue: self.account_set_id_from_code(&account_codes.cost_of_revenue)?,
            expenses: self.account_set_id_from_code(&account_codes.expenses)?,
            equity_retained_earnings: self
                .account_set_id_from_code(&account_codes.equity_retained_earnings)?,
            equity_retained_losses: self
                .account_set_id_from_code(&account_codes.equity_retained_losses)?,
        })
    }

    pub(super) fn post_closing_tx_as_of(
        &mut self,
        account_codes: ClosingAccountCodes,
        tx_details: ClosingTxDetails,
    ) -> Result<Idempotent<ClosingTxParentIdsAndDetails>, ChartOfAccountsError> {
        let closing_tx_params = ClosingTxParentIdsAndDetails::new(
            self.closing_account_set_ids_from_codes(account_codes)?,
            tx_details,
        );
        let posted_as_of = closing_tx_params.posted_as_of();

        idempotency_guard!(
            self.events.iter_all().rev(),
            ChartEvent::ClosingTransactionPosted { posted_as_of: prev_date, .. } if prev_date >= &posted_as_of,
            => ChartEvent::ClosingTransactionPosted { .. }
        );

        self.events
            .push(ChartEvent::ClosingTransactionPosted { posted_as_of });

        Ok(Idempotent::Executed(closing_tx_params))
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .account_set_id(*account_set_id)
                        .reference(reference.to_string())
                        .name(name.to_string());
                }
                ChartEvent::BaseConfigSet { base_config } => {
                    builder = builder.base_config(Some(base_config.clone()));
                }
                ChartEvent::ClosedAsOf { .. } => {}
                ChartEvent::ClosingTransactionPosted { .. } => {}
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
            account_set_id: CalaAccountSetId::new(),
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
    fn adds_from_all_new_trial_balance_accounts() {
        let (chart, (level_1_id, level_2_id, level_3_id)) = default_chart();

        let new_ids = chart
            .trial_balance_account_ids_from_new_accounts(&[level_1_id, level_2_id, level_3_id])
            .collect::<Vec<_>>();
        assert_eq!(new_ids.len(), 1);
        assert!(new_ids.contains(&level_1_id));
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
                    None,
                    vec![section("5")],
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
            .chart_nodes
            .find_persisted_mut(|node| node.spec.code == code("1.1"))
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

    #[test]
    fn closed_as_of_is_chronologically_enforced() {
        let mut chart = chart_from(initial_events());
        let first_date = "2025-01-31".parse::<NaiveDate>().unwrap();
        let _ = chart.close_as_of(first_date);
        let second_date = "2025-02-28".parse::<NaiveDate>().unwrap();
        let second_close_date = chart.close_as_of(second_date);
        assert!(second_close_date.did_execute());
        let invalid_closing_date = "2025-02-01".parse::<NaiveDate>().unwrap();
        let invalid_close_date = chart.close_as_of(invalid_closing_date);
        assert!(invalid_close_date.was_already_applied());
    }

    #[test]
    fn configure_with_initial_accounts_fails_when_base_config_code_not_in_chart() {
        let mut chart = default_chart().0;

        let base_config = AccountingBaseConfig::try_new(
            code("2"),
            code("3"),
            code("4"),
            code("4.1"),
            code("4.2"),
            code("5"),
            code("6"),
            code("7"),
        )
        .unwrap();

        let res = chart.configure_with_initial_accounts(vec![], base_config, CalaJournalId::new());
        assert!(matches!(
            res,
            Err(ChartOfAccountsError::CodeNotFoundInChart(_))
        ));
    }

    mod configure_with_initial_accounts {
        use super::*;

        fn account_specs_for_base_config() -> Vec<AccountSpec> {
            vec![
                AccountSpec {
                    name: "Assets".parse().unwrap(),
                    parent: None,
                    code: code("1"),
                    normal_balance_type: DebitOrCredit::Debit,
                },
                AccountSpec {
                    name: "Liabilities".parse().unwrap(),
                    parent: None,
                    code: code("2"),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                AccountSpec {
                    name: "Equity".parse().unwrap(),
                    parent: None,
                    code: code("3"),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                AccountSpec {
                    name: "Retained Earnings Gain".parse().unwrap(),
                    parent: Some(code("3")),
                    code: code("3.1"),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                AccountSpec {
                    name: "Retained Earnings Loss".parse().unwrap(),
                    parent: Some(code("3")),
                    code: code("3.2"),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                AccountSpec {
                    name: "Revenue".parse().unwrap(),
                    parent: None,
                    code: code("4"),
                    normal_balance_type: DebitOrCredit::Credit,
                },
                AccountSpec {
                    name: "Cost of Revenue".parse().unwrap(),
                    parent: None,
                    code: code("5"),
                    normal_balance_type: DebitOrCredit::Debit,
                },
                AccountSpec {
                    name: "Expenses".parse().unwrap(),
                    parent: None,
                    code: code("6"),
                    normal_balance_type: DebitOrCredit::Debit,
                },
            ]
        }

        fn base_config() -> AccountingBaseConfig {
            AccountingBaseConfig::try_new(
                code("1"),
                code("2"),
                code("3"),
                code("3.1"),
                code("3.2"),
                code("4"),
                code("5"),
                code("6"),
            )
            .unwrap()
        }

        #[test]
        fn first_call_returns_executed() {
            let mut chart = chart_from(initial_events());
            let journal_id = CalaJournalId::new();
            let specs = account_specs_for_base_config();
            let config = base_config();

            let result = chart
                .configure_with_initial_accounts(specs, config, journal_id)
                .unwrap();

            assert!(result.did_execute());
            let bulk_result = result.expect("should be executed");
            assert_eq!(bulk_result.new_account_sets.len(), 8);
        }

        #[test]
        fn second_call_with_same_config_returns_already_applied() {
            let mut chart = chart_from(initial_events());
            let journal_id = CalaJournalId::new();
            let specs = account_specs_for_base_config();
            let config = base_config();

            let first_result = chart
                .configure_with_initial_accounts(specs.clone(), config.clone(), journal_id)
                .unwrap();
            assert!(first_result.did_execute());

            hydrate_chart_of_accounts(&mut chart);

            let second_result = chart
                .configure_with_initial_accounts(specs, config, journal_id)
                .unwrap();

            assert!(second_result.was_already_applied());
        }

        #[test]
        fn errors_when_called_with_different_config() {
            let mut chart = chart_from(initial_events());
            let journal_id = CalaJournalId::new();
            let specs = account_specs_for_base_config();
            let config = base_config();

            let first_result = chart
                .configure_with_initial_accounts(specs.clone(), config, journal_id)
                .unwrap();
            assert!(first_result.did_execute());

            hydrate_chart_of_accounts(&mut chart);

            let different_config = AccountingBaseConfig::try_new(
                code("1"),
                code("2"),
                code("3"),
                code("3.2"),
                code("3.1"),
                code("4"),
                code("5"),
                code("6"),
            )
            .unwrap();

            let second_result =
                chart.configure_with_initial_accounts(specs, different_config, journal_id);

            assert!(matches!(
                second_result,
                Err(ChartOfAccountsError::BaseConfigAlreadyInitializedWithDifferentConfig)
            ));
        }
    }
}
