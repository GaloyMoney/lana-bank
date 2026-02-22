use std::collections::HashMap;

use tui_tree_widget::{TreeItem, TreeState};
use uuid::Uuid;

use crate::db;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Lana,
    Cala,
}

/// Info about available jump actions for the UI.
pub enum JumpInfo {
    NotAvailable,
    /// On LANA view, ready to jump into CALA (shows how many CALA locations).
    LanaToCala {
        total: usize,
    },
    /// On CALA view with a ring active — shows position within the full cycle.
    /// `step` is 1-based within the CALA portion, `cala_total` is # of CALA locations.
    Ring {
        step: usize,
        cala_total: usize,
    },
    /// On CALA view without a ring (e.g. navigated here manually), can jump to LANA.
    CalaToLana,
}

/// Tracks a LANA ↔ CALA round-robin.
/// index 0 = LANA, index 1..N = CALA locations.
struct JumpRing {
    lana_path: Vec<String>,
    cala_paths: Vec<Vec<String>>,
    /// 0 = LANA, 1..=cala_paths.len() = CALA positions
    index: usize,
}

pub struct App<'a> {
    pub active_view: ActiveView,
    pub lana_tree_state: TreeState<String>,
    pub cala_tree_state: TreeState<String>,
    pub lana_items: Vec<TreeItem<'a, String>>,
    pub cala_items: Vec<TreeItem<'a, String>>,
    pub charts: Vec<db::ChartRow>,
    pub chart_nodes: HashMap<Uuid, Vec<db::ChartNodeRow>>,
    pub cala_sets: Vec<db::CalaAccountSetRow>,
    // Lookup maps for details
    pub node_by_set_id: HashMap<Uuid, db::ChartNodeRow>,
    pub set_by_id: HashMap<Uuid, db::CalaAccountSetRow>,
    pub account_members_by_set: HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>>,
    pub set_children_by_parent: HashMap<Uuid, Vec<Uuid>>,
    pub balances_by_account: HashMap<Uuid, Vec<db::AccountBalanceRow>>,
    // Jump ring for cycling through CALA matches
    jump_ring: Option<JumpRing>,
    /// Whether transitive accounts are shown in the CALA tree.
    pub show_transitive: bool,
}

impl<'a> App<'a> {
    pub fn new(
        charts: Vec<db::ChartRow>,
        chart_nodes: HashMap<Uuid, Vec<db::ChartNodeRow>>,
        cala_sets: Vec<db::CalaAccountSetRow>,
        cala_set_members: Vec<db::CalaSetMemberSetRow>,
        cala_account_members: Vec<db::CalaSetMemberAccountRow>,
        balances: Vec<db::AccountBalanceRow>,
    ) -> Self {
        let node_by_set_id: HashMap<Uuid, db::ChartNodeRow> = chart_nodes
            .values()
            .flatten()
            .map(|n| (n.account_set_id, n.clone()))
            .collect();

        let set_by_id: HashMap<Uuid, db::CalaAccountSetRow> =
            cala_sets.iter().map(|s| (s.id, s.clone())).collect();

        let mut account_members_by_set: HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>> =
            HashMap::new();
        for m in &cala_account_members {
            account_members_by_set
                .entry(m.account_set_id)
                .or_default()
                .push(m.clone());
        }

        let mut set_children_by_parent: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for m in &cala_set_members {
            set_children_by_parent
                .entry(m.account_set_id)
                .or_default()
                .push(m.member_account_set_id);
        }

        let mut balances_by_account: HashMap<Uuid, Vec<db::AccountBalanceRow>> = HashMap::new();
        for b in balances {
            balances_by_account.entry(b.account_id).or_default().push(b);
        }

        let show_transitive = false;
        let lana_items = build_lana_tree(&charts, &chart_nodes);
        let cala_items = build_cala_tree(
            &cala_sets,
            &set_children_by_parent,
            &account_members_by_set,
            show_transitive,
        );

        let mut lana_tree_state = TreeState::default();
        lana_tree_state.select_first();
        let mut cala_tree_state = TreeState::default();
        cala_tree_state.select_first();

        Self {
            active_view: ActiveView::Lana,
            lana_tree_state,
            cala_tree_state,
            lana_items,
            cala_items,
            charts,
            chart_nodes,
            cala_sets,
            node_by_set_id,
            set_by_id,
            account_members_by_set,
            set_children_by_parent,
            balances_by_account,
            jump_ring: None,
            show_transitive,
        }
    }

    pub fn toggle_view(&mut self) {
        self.jump_ring = None;
        self.active_view = match self.active_view {
            ActiveView::Lana => ActiveView::Cala,
            ActiveView::Cala => ActiveView::Lana,
        };
    }

    /// Clear the jump ring (call when user navigates normally).
    pub fn clear_jump_ring(&mut self) {
        self.jump_ring = None;
    }

    /// Toggle showing/hiding transitive accounts in the CALA tree.
    pub fn toggle_transitive(&mut self) {
        self.show_transitive = !self.show_transitive;
        // Preserve selection and opened state across rebuild
        let selected = self.cala_tree_state.selected().to_vec();
        let opened: Vec<Vec<String>> = self.cala_tree_state.opened().iter().cloned().collect();
        self.cala_items = build_cala_tree(
            &self.cala_sets,
            &self.set_children_by_parent,
            &self.account_members_by_set,
            self.show_transitive,
        );
        for path in opened {
            self.cala_tree_state.open(path);
        }
        self.cala_tree_state.select(selected);
        self.jump_ring = None;
    }

    /// Returns the account_set_id UUID of the currently selected node, if any.
    fn selected_set_id(&self) -> Option<Uuid> {
        let selected = match self.active_view {
            ActiveView::Lana => self.lana_tree_state.selected(),
            ActiveView::Cala => self.cala_tree_state.selected(),
        };
        if selected.is_empty() {
            return None;
        }
        let last = &selected[selected.len() - 1];
        // Skip member accounts (acct: prefix)
        if last.starts_with("acct:") {
            return None;
        }
        let uuid = last.parse::<Uuid>().ok()?;
        // For LANA view, skip chart root IDs (they're not account sets)
        if self.active_view == ActiveView::Lana && self.charts.iter().any(|c| c.id == uuid) {
            return None;
        }
        Some(uuid)
    }

    /// Info about the jump ring for the status bar / details panel.
    pub fn jump_info(&self) -> JumpInfo {
        // If there's an active ring, report position
        if let Some(ref ring) = self.jump_ring {
            if ring.index == 0 {
                // We're on the LANA step of the ring
                return JumpInfo::LanaToCala {
                    total: ring.cala_paths.len(),
                };
            } else {
                return JumpInfo::Ring {
                    step: ring.index,
                    cala_total: ring.cala_paths.len(),
                };
            }
        }

        // No ring — check if a jump is possible from scratch
        let Some(set_id) = self.selected_set_id() else {
            return JumpInfo::NotAvailable;
        };
        match self.active_view {
            ActiveView::Lana => {
                let paths = find_all_paths_in_tree(&self.cala_items, &set_id.to_string());
                if paths.is_empty() {
                    JumpInfo::NotAvailable
                } else {
                    JumpInfo::LanaToCala { total: paths.len() }
                }
            }
            ActiveView::Cala => {
                if self.node_by_set_id.contains_key(&set_id) {
                    JumpInfo::CalaToLana
                } else {
                    JumpInfo::NotAvailable
                }
            }
        }
    }

    /// Handle the `g` key press.
    /// Cycles: LANA → CALA#1 → CALA#2 → … → LANA → CALA#1 → …
    pub fn jump(&mut self) {
        // If there's an active ring, advance to the next step
        if let Some(ref mut ring) = self.jump_ring {
            let total = 1 + ring.cala_paths.len(); // 1 for LANA + N CALA
            ring.index = (ring.index + 1) % total;

            if ring.index == 0 {
                // Back to LANA
                let path = ring.lana_path.clone();
                open_ancestors(&mut self.lana_tree_state, &path);
                self.lana_tree_state.select(path);
                self.active_view = ActiveView::Lana;
            } else {
                // CALA position (1-based → 0-based into cala_paths)
                let path = ring.cala_paths[ring.index - 1].clone();
                open_ancestors(&mut self.cala_tree_state, &path);
                self.cala_tree_state.select(path);
                self.active_view = ActiveView::Cala;
            }
            return;
        }

        // No ring yet — start one
        let Some(set_id) = self.selected_set_id() else {
            return;
        };
        let target = set_id.to_string();

        match self.active_view {
            ActiveView::Lana => {
                let cala_paths = find_all_paths_in_tree(&self.cala_items, &target);
                if cala_paths.is_empty() {
                    return;
                }
                let lana_path = self.lana_tree_state.selected().to_vec();
                let path = cala_paths[0].clone();
                open_ancestors(&mut self.cala_tree_state, &path);
                self.cala_tree_state.select(path);
                self.jump_ring = Some(JumpRing {
                    lana_path,
                    cala_paths,
                    index: 1, // we just moved to CALA#1
                });
                self.active_view = ActiveView::Cala;
            }
            ActiveView::Cala => {
                // No ring, manual CALA→LANA jump
                if !self.node_by_set_id.contains_key(&set_id) {
                    return;
                }
                if let Some(path) = find_path_in_tree(&self.lana_items, &target) {
                    open_ancestors(&mut self.lana_tree_state, &path);
                    self.lana_tree_state.select(path);
                    self.active_view = ActiveView::Lana;
                }
            }
        }
    }

    pub fn selected_details(&self) -> Vec<String> {
        match self.active_view {
            ActiveView::Lana => self.lana_selected_details(),
            ActiveView::Cala => self.cala_selected_details(),
        }
    }

    fn lana_selected_details(&self) -> Vec<String> {
        let selected = self.lana_tree_state.selected();
        if selected.is_empty() {
            return vec!["No selection".into()];
        }

        let last_id = &selected[selected.len() - 1];

        // Check if it's a chart-level ID
        for chart in &self.charts {
            if last_id == &chart.id.to_string() {
                let mut lines = vec![
                    format!("Chart: {}", chart.name),
                    format!("Reference: {}", chart.reference),
                    format!("Chart ID: {}", chart.id),
                ];
                if let Some(set_id) = chart.account_set_id {
                    lines.push(format!("Root Account Set: {set_id}"));
                }
                return lines;
            }
        }

        // Check if it's a node-level ID (we use account_set_id as the tree identifier)
        if let Some(node) = self
            .node_by_set_id
            .values()
            .find(|n| last_id == &n.account_set_id.to_string())
        {
            let mut lines = vec![
                format!("Code: {}", node.code),
                format!("Name: {}", node.name),
                format!("Normal Balance: {}", node.normal_balance_type),
                String::new(),
                format!("LANA Node ID: {}", node.id),
                format!("CALA Set ID: {}", node.account_set_id),
            ];

            // Show member counts from CALA
            let child_sets = self
                .set_children_by_parent
                .get(&node.account_set_id)
                .map(|v| v.len())
                .unwrap_or(0);
            let member_accounts = self
                .account_members_by_set
                .get(&node.account_set_id)
                .map(|v| v.len())
                .unwrap_or(0);
            let direct = self
                .account_members_by_set
                .get(&node.account_set_id)
                .map(|v| v.iter().filter(|a| !a.transitive).count())
                .unwrap_or(0);
            let transitive = member_accounts - direct;

            lines.push(String::new());
            lines.push("CALA Members:".into());
            lines.push(format!("  {child_sets} child sets"));
            lines.push(format!("  {direct} direct accounts"));
            lines.push(format!("  {transitive} transitive accounts"));

            lines.push(String::new());
            let cala_paths =
                find_all_paths_in_tree(&self.cala_items, &node.account_set_id.to_string());
            if !cala_paths.is_empty() {
                lines.push(format!(
                    "[g] Jump to CALA view ({} locations) →",
                    cala_paths.len()
                ));
            }

            return lines;
        }

        vec!["Unknown selection".into()]
    }

    fn cala_selected_details(&self) -> Vec<String> {
        let selected = self.cala_tree_state.selected();
        if selected.is_empty() {
            return vec!["No selection".into()];
        }

        let last_id = &selected[selected.len() - 1];

        // Check if it's an account (prefixed with "acct:")
        if let Some(acct_id_str) = last_id.strip_prefix("acct:") {
            if let Ok(acct_id) = acct_id_str.parse::<Uuid>() {
                // Find this account in our member data
                for members in self.account_members_by_set.values() {
                    for m in members {
                        if m.account_id == acct_id {
                            let mut lines = vec![
                                "[Account - no LANA node]".into(),
                                String::new(),
                                format!("ID: {}", m.account_id),
                                format!("Code: {}", m.account_code),
                                format!("Name: {}", m.account_name),
                                format!(
                                    "External: {}",
                                    m.account_external_id.as_deref().unwrap_or("(none)")
                                ),
                                format!("Normal Balance: {}", m.normal_balance_type),
                                format!(
                                    "Transitive: {} ({})",
                                    if m.transitive { "yes" } else { "no" },
                                    if m.transitive { "transitive" } else { "direct" }
                                ),
                            ];
                            format_balances(&self.balances_by_account, acct_id, &mut lines);
                            return lines;
                        }
                    }
                }
            }
            return vec!["Account not found".into()];
        }

        // It's an account set
        if let Ok(set_id) = last_id.parse::<Uuid>() {
            if let Some(set) = self.set_by_id.get(&set_id) {
                let mut lines = vec![
                    format!("Account Set: {}", set.name),
                    format!(
                        "External ID: {}",
                        set.external_id.as_deref().unwrap_or("(none)")
                    ),
                    format!("CALA Set ID: {set_id}"),
                ];

                // Check if this has a LANA equivalent
                if let Some(node) = self.node_by_set_id.get(&set_id) {
                    lines.push(String::new());
                    lines.push("LANA Node:".into());
                    lines.push(format!("  Code: {}", node.code));
                    lines.push(format!("  Name: {}", node.name));
                    lines.push(format!("  Node ID: {}", node.id));
                    lines.push(format!("  Normal Balance: {}", node.normal_balance_type));
                    lines.push(String::new());
                    if let Some(ref ring) = self.jump_ring {
                        if ring.cala_paths.len() > 1 {
                            lines.push(format!(
                                "[g] Next → (CALA location {}/{})",
                                ring.index,
                                ring.cala_paths.len()
                            ));
                        } else {
                            lines.push("[g] Jump to LANA view ←".into());
                        }
                    } else {
                        lines.push("[g] Jump to LANA view ←".into());
                    }
                } else {
                    lines.push(String::new());
                    lines.push("No LANA chart node".into());
                }

                let child_sets = self
                    .set_children_by_parent
                    .get(&set_id)
                    .map(|v| v.len())
                    .unwrap_or(0);
                let member_accounts = self
                    .account_members_by_set
                    .get(&set_id)
                    .map(|v| v.len())
                    .unwrap_or(0);
                let direct = self
                    .account_members_by_set
                    .get(&set_id)
                    .map(|v| v.iter().filter(|a| !a.transitive).count())
                    .unwrap_or(0);
                let transitive = member_accounts - direct;

                lines.push(String::new());
                lines.push("Members:".into());
                lines.push(format!("  {child_sets} child sets"));
                lines.push(format!("  {direct} direct accounts"));
                lines.push(format!("  {transitive} transitive accounts"));

                return lines;
            }
        }

        vec!["Unknown selection".into()]
    }
}

fn format_balances(
    balances_by_account: &HashMap<Uuid, Vec<db::AccountBalanceRow>>,
    account_id: Uuid,
    lines: &mut Vec<String>,
) {
    if let Some(bals) = balances_by_account.get(&account_id) {
        lines.push(String::new());
        lines.push("Balances:".into());
        for b in bals {
            lines.push(format!("  {} ({}):", b.currency, short_uuid(b.journal_id)));
            let settled_dr: f64 = b.settled_dr.parse().unwrap_or(0.0);
            let settled_cr: f64 = b.settled_cr.parse().unwrap_or(0.0);
            let pending_dr: f64 = b.pending_dr.parse().unwrap_or(0.0);
            let pending_cr: f64 = b.pending_cr.parse().unwrap_or(0.0);
            let encumbrance_dr: f64 = b.encumbrance_dr.parse().unwrap_or(0.0);
            let encumbrance_cr: f64 = b.encumbrance_cr.parse().unwrap_or(0.0);

            lines.push(format!("    settled     DR {settled_dr}  CR {settled_cr}"));
            if pending_dr != 0.0 || pending_cr != 0.0 {
                lines.push(format!("    pending     DR {pending_dr}  CR {pending_cr}"));
            }
            if encumbrance_dr != 0.0 || encumbrance_cr != 0.0 {
                lines.push(format!(
                    "    encumbrance DR {encumbrance_dr}  CR {encumbrance_cr}"
                ));
            }
        }
    }
}

fn short_uuid(id: Uuid) -> String {
    let s = id.to_string();
    s[..8].to_string()
}

/// Find the full path (sequence of identifiers) to the first occurrence of a node in the tree.
fn find_path_in_tree(items: &[TreeItem<'_, String>], target: &str) -> Option<Vec<String>> {
    for item in items {
        if item.identifier() == target {
            return Some(vec![target.to_string()]);
        }
        if let Some(mut path) = find_path_in_tree(item.children(), target) {
            path.insert(0, item.identifier().clone());
            return Some(path);
        }
    }
    None
}

/// Find ALL paths to a given identifier in the tree (it can appear in multiple branches).
fn find_all_paths_in_tree(items: &[TreeItem<'_, String>], target: &str) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    collect_paths(items, target, &[], &mut results);
    results
}

fn collect_paths(
    items: &[TreeItem<'_, String>],
    target: &str,
    prefix: &[String],
    results: &mut Vec<Vec<String>>,
) {
    for item in items {
        let mut current_path: Vec<String> = prefix.to_vec();
        current_path.push(item.identifier().clone());

        if item.identifier() == target {
            results.push(current_path.clone());
        }
        // Continue searching children even after a match (the same ID
        // won't appear as a descendant of itself, but other matches
        // might be in sibling subtrees).
        collect_paths(item.children(), target, &current_path, results);
    }
}

/// Open all ancestor nodes so the target is visible.
fn open_ancestors(state: &mut TreeState<String>, path: &[String]) {
    for i in 0..path.len().saturating_sub(1) {
        state.open(path[..=i].to_vec());
    }
}

fn build_lana_tree<'a>(
    charts: &[db::ChartRow],
    chart_nodes: &HashMap<Uuid, Vec<db::ChartNodeRow>>,
) -> Vec<TreeItem<'a, String>> {
    let mut items = Vec::new();

    for chart in charts {
        let nodes = match chart_nodes.get(&chart.id) {
            Some(n) => n,
            None => continue,
        };

        // Build node tree by parent_code
        let roots: Vec<&db::ChartNodeRow> =
            nodes.iter().filter(|n| n.parent_code.is_none()).collect();
        let children_by_parent: HashMap<&str, Vec<&db::ChartNodeRow>> = {
            let mut map: HashMap<&str, Vec<&db::ChartNodeRow>> = HashMap::new();
            for node in nodes {
                if let Some(ref parent) = node.parent_code {
                    map.entry(parent.as_str()).or_default().push(node);
                }
            }
            map
        };

        fn build_node_item<'a>(
            node: &db::ChartNodeRow,
            children_map: &HashMap<&str, Vec<&db::ChartNodeRow>>,
        ) -> TreeItem<'a, String> {
            let label = format!("{} {} ({})", node.code, node.name, node.normal_balance_type);

            let children: Vec<TreeItem<'a, String>> =
                if let Some(child_nodes) = children_map.get(node.code.as_str()) {
                    let mut sorted = child_nodes.clone();
                    sorted.sort_by(|a, b| a.code.cmp(&b.code));
                    sorted
                        .iter()
                        .map(|c| build_node_item(c, children_map))
                        .collect()
                } else {
                    Vec::new()
                };

            if children.is_empty() {
                TreeItem::new_leaf(node.account_set_id.to_string(), label)
            } else {
                TreeItem::new(node.account_set_id.to_string(), label, children)
                    .expect("duplicate identifiers in LANA tree")
            }
        }

        let mut root_items: Vec<TreeItem<'a, String>> = {
            let mut sorted_roots = roots;
            sorted_roots.sort_by(|a, b| a.code.cmp(&b.code));
            sorted_roots
                .iter()
                .map(|r| build_node_item(r, &children_by_parent))
                .collect()
        };

        let chart_label = format!("{} (ref: {})", chart.name, chart.reference);
        let chart_item = if root_items.is_empty() {
            TreeItem::new_leaf(chart.id.to_string(), chart_label)
        } else {
            TreeItem::new(
                chart.id.to_string(),
                chart_label,
                std::mem::take(&mut root_items),
            )
            .expect("duplicate identifiers in chart")
        };
        items.push(chart_item);
    }

    items
}

fn build_cala_tree<'a>(
    sets: &[db::CalaAccountSetRow],
    children_by_parent: &HashMap<Uuid, Vec<Uuid>>,
    account_members_by_set: &HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>>,
    show_transitive: bool,
) -> Vec<TreeItem<'a, String>> {
    let set_by_id: HashMap<Uuid, &db::CalaAccountSetRow> = sets.iter().map(|s| (s.id, s)).collect();

    // Find root sets: those that are not children of any other set
    let all_children: std::collections::HashSet<Uuid> =
        children_by_parent.values().flatten().copied().collect();
    let mut roots: Vec<Uuid> = sets
        .iter()
        .filter(|s| !all_children.contains(&s.id))
        .map(|s| s.id)
        .collect();
    roots.sort_by(|a, b| {
        let a_name = set_by_id.get(a).map(|s| s.name.as_str()).unwrap_or("");
        let b_name = set_by_id.get(b).map(|s| s.name.as_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

    fn build_set_item<'a>(
        set_id: Uuid,
        set_by_id: &HashMap<Uuid, &db::CalaAccountSetRow>,
        children_by_parent: &HashMap<Uuid, Vec<Uuid>>,
        account_members: &HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>>,
        show_transitive: bool,
    ) -> TreeItem<'a, String> {
        let set = set_by_id.get(&set_id);
        let label = set
            .map(|s| s.name.clone())
            .unwrap_or_else(|| set_id.to_string());

        let mut children: Vec<TreeItem<'a, String>> = Vec::new();

        // Add child account sets
        if let Some(child_ids) = children_by_parent.get(&set_id) {
            let mut sorted = child_ids.clone();
            sorted.sort_by(|a, b| {
                let a_name = set_by_id.get(a).map(|s| s.name.as_str()).unwrap_or("");
                let b_name = set_by_id.get(b).map(|s| s.name.as_str()).unwrap_or("");
                a_name.cmp(b_name)
            });
            for child_id in sorted {
                children.push(build_set_item(
                    child_id,
                    set_by_id,
                    children_by_parent,
                    account_members,
                    show_transitive,
                ));
            }
        }

        // Add member accounts as leaves
        if let Some(accounts) = account_members.get(&set_id) {
            for acct in accounts {
                if !show_transitive && acct.transitive {
                    continue;
                }
                let acct_label = format!(
                    "[acct] {} - {} ({})",
                    acct.account_code,
                    acct.account_name,
                    if acct.transitive {
                        "transitive"
                    } else {
                        "direct"
                    },
                );
                // Use "acct:" prefix to distinguish from set IDs
                let id = format!("acct:{}", acct.account_id);
                children.push(TreeItem::new_leaf(id, acct_label));
            }
        }

        if children.is_empty() {
            TreeItem::new_leaf(set_id.to_string(), label)
        } else {
            TreeItem::new(set_id.to_string(), label, children)
                .expect("duplicate identifiers in CALA tree")
        }
    }

    roots
        .iter()
        .map(|&root_id| {
            build_set_item(
                root_id,
                &set_by_id,
                children_by_parent,
                account_members_by_set,
                show_transitive,
            )
        })
        .collect()
}

// ── Dump helpers ──────────────────────────────────────────────────

pub fn dump_text(app: &App) {
    for chart in &app.charts {
        println!(
            "=== LANA Chart: \"{}\" (ref: {}) ===",
            chart.name, chart.reference
        );
        if let Some(nodes) = app.chart_nodes.get(&chart.id) {
            let roots: Vec<&db::ChartNodeRow> =
                nodes.iter().filter(|n| n.parent_code.is_none()).collect();
            let children_map = build_parent_map(nodes);
            let mut sorted_roots = roots;
            sorted_roots.sort_by(|a, b| a.code.cmp(&b.code));
            for root in sorted_roots {
                dump_lana_node(root, &children_map, 1);
            }
        }
        println!();
    }

    println!("=== CALA Account Sets ===");
    let all_children: std::collections::HashSet<Uuid> = app
        .set_children_by_parent
        .values()
        .flatten()
        .copied()
        .collect();
    let mut roots: Vec<&db::CalaAccountSetRow> = app
        .cala_sets
        .iter()
        .filter(|s| !all_children.contains(&s.id))
        .collect();
    roots.sort_by(|a, b| a.name.cmp(&b.name));
    for root in roots {
        dump_cala_set(root.id, app, 1);
    }
}

fn build_parent_map(nodes: &[db::ChartNodeRow]) -> HashMap<&str, Vec<&db::ChartNodeRow>> {
    let mut map: HashMap<&str, Vec<&db::ChartNodeRow>> = HashMap::new();
    for node in nodes {
        if let Some(ref parent) = node.parent_code {
            map.entry(parent.as_str()).or_default().push(node);
        }
    }
    map
}

fn dump_lana_node(
    node: &db::ChartNodeRow,
    children_map: &HashMap<&str, Vec<&db::ChartNodeRow>>,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    println!(
        "{}{} {} ({}) [set:{}]",
        indent, node.code, node.name, node.normal_balance_type, node.account_set_id
    );
    if let Some(children) = children_map.get(node.code.as_str()) {
        let mut sorted = children.clone();
        sorted.sort_by(|a, b| a.code.cmp(&b.code));
        for child in sorted {
            dump_lana_node(child, children_map, depth + 1);
        }
    }
}

fn dump_cala_set(set_id: Uuid, app: &App, depth: usize) {
    let indent = "  ".repeat(depth);
    let name = app
        .set_by_id
        .get(&set_id)
        .map(|s| s.name.as_str())
        .unwrap_or("(unknown)");
    println!("{}{} [set:{}]", indent, name, set_id);

    // Child sets
    if let Some(children) = app.set_children_by_parent.get(&set_id) {
        let mut sorted = children.clone();
        sorted.sort_by(|a, b| {
            let a_name = app.set_by_id.get(a).map(|s| s.name.as_str()).unwrap_or("");
            let b_name = app.set_by_id.get(b).map(|s| s.name.as_str()).unwrap_or("");
            a_name.cmp(b_name)
        });
        for child_id in sorted {
            dump_cala_set(child_id, app, depth + 1);
        }
    }

    // Member accounts
    if let Some(accounts) = app.account_members_by_set.get(&set_id) {
        for acct in accounts {
            println!(
                "{}  [acct] {} ({}) [id:{}]",
                indent,
                acct.account_code,
                if acct.transitive {
                    "transitive"
                } else {
                    "direct"
                },
                acct.account_id,
            );
        }
    }
}

pub fn dump_json(app: &App) {
    let mut lana_charts = Vec::new();
    for chart in &app.charts {
        let mut chart_obj = serde_json::json!({
            "id": chart.id.to_string(),
            "name": chart.name,
            "reference": chart.reference,
            "account_set_id": chart.account_set_id.map(|id| id.to_string()),
        });
        if let Some(nodes) = app.chart_nodes.get(&chart.id) {
            let node_objs: Vec<serde_json::Value> = nodes
                .iter()
                .map(|n| {
                    serde_json::json!({
                        "id": n.id.to_string(),
                        "code": n.code,
                        "name": n.name,
                        "parent_code": n.parent_code,
                        "normal_balance_type": n.normal_balance_type,
                        "account_set_id": n.account_set_id.to_string(),
                    })
                })
                .collect();
            chart_obj["nodes"] = serde_json::Value::Array(node_objs);
        }
        lana_charts.push(chart_obj);
    }

    let cala_sets: Vec<serde_json::Value> = app
        .cala_sets
        .iter()
        .map(|s| {
            let children: Vec<String> = app
                .set_children_by_parent
                .get(&s.id)
                .map(|ids| ids.iter().map(|id| id.to_string()).collect())
                .unwrap_or_default();
            let accounts: Vec<serde_json::Value> = app
                .account_members_by_set
                .get(&s.id)
                .map(|accts| {
                    accts
                        .iter()
                        .map(|a| {
                            serde_json::json!({
                                "account_id": a.account_id.to_string(),
                                "code": a.account_code,
                                "name": a.account_name,
                                "external_id": a.account_external_id,
                                "normal_balance_type": a.normal_balance_type,
                                "transitive": a.transitive,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            serde_json::json!({
                "id": s.id.to_string(),
                "name": s.name,
                "external_id": s.external_id,
                "child_set_ids": children,
                "member_accounts": accounts,
            })
        })
        .collect();

    let output = serde_json::json!({
        "lana_charts": lana_charts,
        "cala_account_sets": cala_sets,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
