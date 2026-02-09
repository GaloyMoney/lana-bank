use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use cala_ledger::{AccountSetId, CalaLedger, account_set::AccountSetMemberId};
use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};
use uuid::Uuid;

use super::error::ChartOfAccountsError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagNodeType {
    AccountSet,
    Account,
}

#[derive(Debug, Clone)]
pub struct DagNode {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub node_type: DagNodeType,
}

#[derive(Debug, Clone)]
pub struct DagEdge {
    pub source: Uuid,
    pub target: Uuid,
}

#[derive(Debug, Clone)]
pub struct AccountDag {
    pub nodes: Vec<DagNode>,
    pub edges: Vec<DagEdge>,
}

pub async fn build_account_dag(
    cala: &CalaLedger,
    root: AccountSetId,
) -> Result<AccountDag, ChartOfAccountsError> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Add root node
    let root_set = cala.account_sets().find(root).await?;
    let root_values = root_set.into_values();
    let root_uuid: Uuid = root.into();
    nodes.push(DagNode {
        id: root_uuid,
        name: root_values.name,
        code: root_values.external_id,
        node_type: DagNodeType::AccountSet,
    });
    visited.insert(root_uuid);
    queue.push_back(root);

    while let Some(parent_set_id) = queue.pop_front() {
        let parent_uuid: Uuid = parent_set_id.into();

        let mut has_next_page = true;
        let mut after = None;

        while has_next_page {
            let PaginatedQueryRet {
                entities,
                has_next_page: next_page,
                end_cursor,
            } = cala
                .account_sets()
                .list_members_by_created_at(parent_set_id, PaginatedQueryArgs { first: 100, after })
                .await?;

            after = end_cursor;
            has_next_page = next_page;

            for member in entities {
                match member.id {
                    AccountSetMemberId::AccountSet(set_id) => {
                        let child_uuid: Uuid = set_id.into();
                        edges.push(DagEdge {
                            source: parent_uuid,
                            target: child_uuid,
                        });

                        if visited.insert(child_uuid) {
                            let account_set = cala.account_sets().find(set_id).await?;
                            let values = account_set.into_values();
                            nodes.push(DagNode {
                                id: child_uuid,
                                name: values.name,
                                code: values.external_id,
                                node_type: DagNodeType::AccountSet,
                            });
                            queue.push_back(set_id);
                        }
                    }
                    AccountSetMemberId::Account(account_id) => {
                        let child_uuid: Uuid = account_id.into();
                        edges.push(DagEdge {
                            source: parent_uuid,
                            target: child_uuid,
                        });

                        if visited.insert(child_uuid) {
                            let account = cala.accounts().find(account_id).await?;
                            let values = account.into_values();
                            nodes.push(DagNode {
                                id: child_uuid,
                                name: values.name,
                                code: values.external_id,
                                node_type: DagNodeType::Account,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(AccountDag { nodes, edges })
}

impl AccountDag {
    pub fn to_d2(&self) -> String {
        let keys = self.build_node_keys();
        let mut out = String::new();

        for node in &self.nodes {
            let key = &keys[&node.id];
            let label = match &node.code {
                Some(code) => format!("{code} - {}", node.name),
                None => node.name.clone(),
            };
            let shape = match node.node_type {
                DagNodeType::AccountSet => "rectangle",
                DagNodeType::Account => "oval",
            };
            out.push_str(&format!("{key}: \"{label}\" {{ shape: {shape} }}\n"));
        }

        for edge in &self.edges {
            let src = &keys[&edge.source];
            let tgt = &keys[&edge.target];
            out.push_str(&format!("{src} -> {tgt}\n"));
        }

        out
    }

    fn build_node_keys(&self) -> HashMap<Uuid, String> {
        let mut keys = HashMap::new();
        let mut used = HashSet::new();

        for node in &self.nodes {
            let hex = node.id.simple().to_string();
            let mut prefix_len = 12;
            loop {
                let candidate = format!("n_{}", &hex[..prefix_len.min(hex.len())]);
                if !used.contains(&candidate) || prefix_len >= hex.len() {
                    used.insert(candidate.clone());
                    keys.insert(node.id, candidate);
                    break;
                }
                prefix_len += 4;
            }
        }

        keys
    }
}

impl fmt::Display for AccountDag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_d2())
    }
}
