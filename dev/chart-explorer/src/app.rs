use std::collections::HashMap;

use tui_tree_widget::{TreeItem, TreeState};
use uuid::Uuid;

use crate::db;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Lana,
    Cala,
    Products,
}

#[derive(Debug, Clone)]
pub struct ProductMapping {
    pub product: String,
    pub role: String,
    pub chart_code: String,
    pub chart_name: String,
    pub chart_parent_set_id: Uuid,
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
    pub product_tree_state: TreeState<String>,
    pub lana_items: Vec<TreeItem<'a, String>>,
    pub cala_items: Vec<TreeItem<'a, String>>,
    pub product_items: Vec<TreeItem<'a, String>>,
    pub charts: Vec<db::ChartRow>,
    pub chart_nodes: HashMap<Uuid, Vec<db::ChartNodeRow>>,
    pub cala_sets: Vec<db::CalaAccountSetRow>,
    // Lookup maps for details
    pub node_by_set_id: HashMap<Uuid, db::ChartNodeRow>,
    pub set_by_id: HashMap<Uuid, db::CalaAccountSetRow>,
    pub account_members_by_set: HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>>,
    pub set_children_by_parent: HashMap<Uuid, Vec<Uuid>>,
    pub balances_by_account: HashMap<Uuid, Vec<db::AccountBalanceRow>>,
    // Product integration
    pub product_by_parent_set_id: HashMap<Uuid, Vec<ProductMapping>>,
    pub product_by_child_set_id: HashMap<Uuid, ProductMapping>,
    pub product_config_keys: Vec<(String, usize)>, // (product name, mapping count)
    // Jump ring for cycling through CALA matches
    jump_ring: Option<JumpRing>,
    /// Whether transitive accounts are shown in the CALA tree.
    pub show_transitive: bool,
    /// (currency, pending_net, settled_net) per account set — rolled up from all member accounts
    pub balance_by_set: HashMap<Uuid, Vec<(String, f64, f64)>>,
    /// (currency, pending_net, settled_net) per individual account
    pub balance_by_acct: HashMap<Uuid, Vec<(String, f64, f64)>>,
}

impl<'a> App<'a> {
    pub fn new(
        charts: Vec<db::ChartRow>,
        chart_nodes: HashMap<Uuid, Vec<db::ChartNodeRow>>,
        cala_sets: Vec<db::CalaAccountSetRow>,
        cala_set_members: Vec<db::CalaSetMemberSetRow>,
        cala_account_members: Vec<db::CalaSetMemberAccountRow>,
        balances: Vec<db::AccountBalanceRow>,
        product_configs: Vec<db::ProductConfigRow>,
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

        // Parse product configs into mappings
        let (product_by_parent_set_id, product_by_child_set_id, product_config_keys) =
            parse_product_configs(&product_configs, &set_children_by_parent, &node_by_set_id);

        let balance_by_acct = compute_account_balances(&balances_by_account);
        let balance_by_set =
            compute_set_balances(&account_members_by_set, &balances_by_account);

        let show_transitive = false;
        let lana_items = build_lana_tree(
            &charts,
            &chart_nodes,
            &product_by_parent_set_id,
            &balance_by_set,
        );
        let cala_items = build_cala_tree(
            &cala_sets,
            &set_children_by_parent,
            &account_members_by_set,
            &product_by_child_set_id,
            show_transitive,
            &balance_by_set,
            &balance_by_acct,
        );
        let product_items = build_product_tree(
            &product_by_parent_set_id,
            &product_config_keys,
            &node_by_set_id,
            &set_children_by_parent,
            &set_by_id,
            &account_members_by_set,
            show_transitive,
            &balance_by_set,
            &balance_by_acct,
        );

        let mut lana_tree_state = TreeState::default();
        lana_tree_state.select_first();
        let mut cala_tree_state = TreeState::default();
        cala_tree_state.select_first();
        let mut product_tree_state = TreeState::default();
        product_tree_state.select_first();

        Self {
            active_view: ActiveView::Lana,
            lana_tree_state,
            cala_tree_state,
            product_tree_state,
            lana_items,
            cala_items,
            product_items,
            charts,
            chart_nodes,
            cala_sets,
            node_by_set_id,
            set_by_id,
            account_members_by_set,
            set_children_by_parent,
            balances_by_account,
            product_by_parent_set_id,
            product_by_child_set_id,
            product_config_keys,
            jump_ring: None,
            show_transitive,
            balance_by_set,
            balance_by_acct,
        }
    }

    pub fn toggle_view(&mut self) {
        self.jump_ring = None;
        self.active_view = match self.active_view {
            ActiveView::Lana => ActiveView::Cala,
            ActiveView::Cala => ActiveView::Products,
            ActiveView::Products => ActiveView::Lana,
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
            &self.product_by_child_set_id,
            self.show_transitive,
            &self.balance_by_set,
            &self.balance_by_acct,
        );
        for path in opened {
            self.cala_tree_state.open(path);
        }
        self.cala_tree_state.select(selected);

        // Also rebuild product tree
        let prod_selected = self.product_tree_state.selected().to_vec();
        let prod_opened: Vec<Vec<String>> =
            self.product_tree_state.opened().iter().cloned().collect();
        self.product_items = build_product_tree(
            &self.product_by_parent_set_id,
            &self.product_config_keys,
            &self.node_by_set_id,
            &self.set_children_by_parent,
            &self.set_by_id,
            &self.account_members_by_set,
            self.show_transitive,
            &self.balance_by_set,
            &self.balance_by_acct,
        );
        for path in prod_opened {
            self.product_tree_state.open(path);
        }
        self.product_tree_state.select(prod_selected);

        self.jump_ring = None;
    }

    /// Returns the account_set_id UUID of the currently selected node, if any.
    fn selected_set_id(&self) -> Option<Uuid> {
        let selected = match self.active_view {
            ActiveView::Lana => self.lana_tree_state.selected(),
            ActiveView::Cala => self.cala_tree_state.selected(),
            ActiveView::Products => self.product_tree_state.selected(),
        };
        if selected.is_empty() {
            return None;
        }
        let last = &selected[selected.len() - 1];
        // Skip member accounts (acct: prefix) and product tree synthetic IDs
        if last.starts_with("acct:") || last.starts_with("prod:") || last.starts_with("role:") {
            return None;
        }
        let uuid = last.parse::<Uuid>().ok()?;
        // For LANA view, skip chart root IDs (they're not account sets)
        if self.active_view == ActiveView::Lana && self.charts.iter().any(|c| c.id == uuid) {
            return None;
        }
        Some(uuid)
    }

    /// Like `selected_set_id()`, but also resolves `role:` nodes to their `chart_parent_set_id`.
    fn selected_jump_set_id(&self) -> Option<Uuid> {
        if let Some(id) = self.selected_set_id() {
            return Some(id);
        }
        let selected = match self.active_view {
            ActiveView::Products => self.product_tree_state.selected(),
            _ => return None,
        };
        if selected.is_empty() {
            return None;
        }
        let last = &selected[selected.len() - 1];
        let rest = last.strip_prefix("role:")?;
        let (product, role) = rest.split_once(':')?;
        for mappings in self.product_by_parent_set_id.values() {
            for m in mappings {
                if m.product == product && m.role == role {
                    return Some(m.chart_parent_set_id);
                }
            }
        }
        None
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
        let Some(set_id) = self.selected_jump_set_id() else {
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
            ActiveView::Cala | ActiveView::Products => {
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
        let Some(set_id) = self.selected_jump_set_id() else {
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
            ActiveView::Cala | ActiveView::Products => {
                // No ring, manual CALA/Products→LANA jump
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
            ActiveView::Products => self.products_selected_details(),
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

            // Aggregate balances across direct member accounts
            if let Some(members) = self.account_members_by_set.get(&node.account_set_id) {
                let direct_members: Vec<Uuid> = members
                    .iter()
                    .filter(|m| !m.transitive)
                    .map(|m| m.account_id)
                    .collect();
                format_aggregate_balances(&self.balances_by_account, &direct_members, &mut lines);
            }

            // Show product annotations for this chart node
            if let Some(mappings) = self.product_by_parent_set_id.get(&node.account_set_id) {
                lines.push(String::new());
                lines.push("Product Integrations:".into());
                for m in mappings {
                    lines.push(format!("  [{}] {}", m.product, m.role));
                }
            }

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
        if let Some(set) = last_id
            .parse::<Uuid>()
            .ok()
            .and_then(|id| self.set_by_id.get(&id).map(|s| (id, s)))
        {
            let (set_id, set) = set;
            let mut lines = vec![
                format!("Account Set: {}", set.name),
                format!(
                    "External ID: {}",
                    set.external_id.as_deref().unwrap_or("(none)")
                ),
                format!("CALA Set ID: {set_id}"),
            ];

            // Show product annotation
            if let Some(mapping) = self.product_by_child_set_id.get(&set_id) {
                lines.push(String::new());
                lines.push(format!("Product: [{}] {}", mapping.product, mapping.role));
                lines.push(format!(
                    "Chart Parent: {} ({})",
                    mapping.chart_code, mapping.chart_name
                ));
            } else if let Some(mappings) = self.product_by_parent_set_id.get(&set_id) {
                lines.push(String::new());
                lines.push("Product Integrations:".into());
                for m in mappings {
                    lines.push(format!("  [{}] {}", m.product, m.role));
                }
            }

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

            // Aggregate balances across all direct member accounts
            if let Some(members) = self.account_members_by_set.get(&set_id) {
                let direct_members: Vec<Uuid> = members
                    .iter()
                    .filter(|m| !m.transitive)
                    .map(|m| m.account_id)
                    .collect();
                format_aggregate_balances(&self.balances_by_account, &direct_members, &mut lines);
            }

            return lines;
        }

        vec!["Unknown selection".into()]
    }

    fn products_selected_details(&self) -> Vec<String> {
        let selected = self.product_tree_state.selected();
        if selected.is_empty() {
            return vec!["No selection".into()];
        }

        let last_id = &selected[selected.len() - 1];

        // Product root node: "prod:Credit" or "prod:Deposit"
        if let Some(product_name) = last_id.strip_prefix("prod:") {
            let mapping_count = self
                .product_config_keys
                .iter()
                .find(|(name, _)| name == product_name)
                .map(|(_, count)| *count)
                .unwrap_or(0);
            let key = if product_name == "Credit" {
                "credit-chart-of-accounts-integration"
            } else {
                "deposit-chart-of-accounts-integration"
            };
            return vec![
                format!("Product: {product_name}"),
                "Status: Configured".into(),
                format!("Integration Key: {key}"),
                format!("Total Mappings: {mapping_count}"),
            ];
        }

        // Role node: "role:Credit:facility_omnibus_parent"
        if let Some(rest) = last_id.strip_prefix("role:") {
            if let Some((product, role)) = rest.split_once(':') {
                // Find the mapping for this role
                for mappings in self.product_by_parent_set_id.values() {
                    for m in mappings {
                        if m.product == product && m.role == role {
                            let mut lines = vec![
                                format!("Product: {}", m.product),
                                format!("Role: {}", m.role),
                                format!("Chart Parent: {} ({})", m.chart_code, m.chart_name),
                                format!("Chart Parent Set ID: {}", m.chart_parent_set_id),
                            ];
                            // Show child sets
                            if let Some(children) =
                                self.set_children_by_parent.get(&m.chart_parent_set_id)
                            {
                                let product_children: Vec<_> = children
                                    .iter()
                                    .filter(|c| !self.node_by_set_id.contains_key(c))
                                    .collect();
                                if !product_children.is_empty() {
                                    lines.push(String::new());
                                    lines.push(format!(
                                        "Product Child Sets: {}",
                                        product_children.len()
                                    ));
                                }
                            }
                            lines.push(String::new());
                            lines.push("[g] Jump to LANA view ←".into());
                            return lines;
                        }
                    }
                }
            }
            return vec!["Unknown role".into()];
        }

        // Check if it's an account (prefixed with "acct:")
        if let Some(acct_id_str) = last_id.strip_prefix("acct:") {
            if let Ok(acct_id) = acct_id_str.parse::<Uuid>() {
                for members in self.account_members_by_set.values() {
                    for m in members {
                        if m.account_id == acct_id {
                            let mut lines = vec![
                                format!("ID: {}", m.account_id),
                                format!("Code: {}", m.account_code),
                                format!("Name: {}", m.account_name),
                                format!(
                                    "External: {}",
                                    m.account_external_id.as_deref().unwrap_or("(none)")
                                ),
                                format!("Normal Balance: {}", m.normal_balance_type),
                            ];
                            format_balances(&self.balances_by_account, acct_id, &mut lines);
                            return lines;
                        }
                    }
                }
            }
            return vec!["Account not found".into()];
        }

        // It's a CALA set ID — reuse CALA details logic
        if let Some(set) = last_id
            .parse::<Uuid>()
            .ok()
            .and_then(|id| self.set_by_id.get(&id).map(|s| (id, s)))
        {
            let (set_id, set) = set;
            let mut lines = vec![
                format!("Account Set: {}", set.name),
                format!(
                    "External ID: {}",
                    set.external_id.as_deref().unwrap_or("(none)")
                ),
                format!("CALA Set ID: {set_id}"),
            ];

            if let Some(mapping) = self.product_by_child_set_id.get(&set_id) {
                lines.push(String::new());
                lines.push(format!("Product: [{}] {}", mapping.product, mapping.role));
            }

            if let Some(node) = self.node_by_set_id.get(&set_id) {
                lines.push(String::new());
                lines.push("LANA Node:".into());
                lines.push(format!("  Code: {}", node.code));
                lines.push(format!("  Name: {}", node.name));
                lines.push(String::new());
                lines.push("[g] Jump to LANA view ←".into());
            }

            let child_sets = self
                .set_children_by_parent
                .get(&set_id)
                .map(|v| v.len())
                .unwrap_or(0);
            let direct = self
                .account_members_by_set
                .get(&set_id)
                .map(|v| v.iter().filter(|a| !a.transitive).count())
                .unwrap_or(0);
            let transitive = self
                .account_members_by_set
                .get(&set_id)
                .map(|v| v.iter().filter(|a| a.transitive).count())
                .unwrap_or(0);

            lines.push(String::new());
            lines.push("Members:".into());
            lines.push(format!("  {child_sets} child sets"));
            lines.push(format!("  {direct} direct accounts"));
            lines.push(format!("  {transitive} transitive accounts"));

            if let Some(members) = self.account_members_by_set.get(&set_id) {
                let direct_members: Vec<Uuid> = members
                    .iter()
                    .filter(|m| !m.transitive)
                    .map(|m| m.account_id)
                    .collect();
                format_aggregate_balances(&self.balances_by_account, &direct_members, &mut lines);
            }

            return lines;
        }

        vec!["No details".into()]
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

fn format_aggregate_balances(
    balances_by_account: &HashMap<Uuid, Vec<db::AccountBalanceRow>>,
    account_ids: &[Uuid],
    lines: &mut Vec<String>,
) {
    // Sum balances by (currency, journal_id)
    let mut totals: HashMap<(String, Uuid), [f64; 6]> = HashMap::new();
    for &acct_id in account_ids {
        if let Some(bals) = balances_by_account.get(&acct_id) {
            for b in bals {
                let entry = totals
                    .entry((b.currency.clone(), b.journal_id))
                    .or_insert([0.0; 6]);
                entry[0] += b.settled_dr.parse::<f64>().unwrap_or(0.0);
                entry[1] += b.settled_cr.parse::<f64>().unwrap_or(0.0);
                entry[2] += b.pending_dr.parse::<f64>().unwrap_or(0.0);
                entry[3] += b.pending_cr.parse::<f64>().unwrap_or(0.0);
                entry[4] += b.encumbrance_dr.parse::<f64>().unwrap_or(0.0);
                entry[5] += b.encumbrance_cr.parse::<f64>().unwrap_or(0.0);
            }
        }
    }

    if totals.is_empty() {
        return;
    }

    let mut keys: Vec<_> = totals.keys().cloned().collect();
    keys.sort();
    lines.push(String::new());
    lines.push("Aggregate Balances:".into());
    for (currency, journal_id) in keys {
        let t = &totals[&(currency.clone(), journal_id)];
        lines.push(format!("  {} ({}):", currency, short_uuid(journal_id)));
        lines.push(format!("    settled     DR {}  CR {}", t[0], t[1]));
        if t[2] != 0.0 || t[3] != 0.0 {
            lines.push(format!("    pending     DR {}  CR {}", t[2], t[3]));
        }
        if t[4] != 0.0 || t[5] != 0.0 {
            lines.push(format!("    encumbrance DR {}  CR {}", t[4], t[5]));
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

/// Parse product integration configs from the DB.
///
/// The DB stores the *resolved* config as JSON:
///   `{ "type": "...", "value": { "config": {...}, "<role>_parent_account_set_id": "uuid", ... } }`
///
/// Top-level fields ending in `_parent_account_set_id` contain CALA set UUIDs.
/// Nested `*_integration_meta` objects also contain `_parent_account_set_id` fields.
type ProductMaps = (
    HashMap<Uuid, Vec<ProductMapping>>,
    HashMap<Uuid, ProductMapping>,
    Vec<(String, usize)>,
);

fn parse_product_configs(
    configs: &[db::ProductConfigRow],
    set_children_by_parent: &HashMap<Uuid, Vec<Uuid>>,
    node_by_set_id: &HashMap<Uuid, db::ChartNodeRow>,
) -> ProductMaps {
    let mut by_parent: HashMap<Uuid, Vec<ProductMapping>> = HashMap::new();
    let mut by_child: HashMap<Uuid, ProductMapping> = HashMap::new();
    let mut config_keys = Vec::new();

    for config in configs {
        let product = if config.key.starts_with("credit-") {
            "Credit"
        } else {
            "Deposit"
        };

        // Unwrap the event envelope: { "type": "...", "value": { ... } }
        let inner = match config.value.get("value").and_then(|v| v.as_object()) {
            Some(o) => o,
            None => continue,
        };

        let mut mapping_count = 0;

        // Collect all _parent_account_set_id fields (top-level and nested in *_integration_meta)
        let mut set_id_fields: Vec<(String, String)> = Vec::new();
        for (field, value) in inner {
            if field == "config" {
                continue;
            }
            if field.ends_with("_parent_account_set_id") {
                if let Some(uuid_str) = value.as_str() {
                    set_id_fields.push((field.clone(), uuid_str.to_string()));
                }
            } else if field.ends_with("_integration_meta") {
                // Walk nested object for more _parent_account_set_id fields
                if let Some(meta_obj) = value.as_object() {
                    for (mf, mv) in meta_obj {
                        if mf.ends_with("_parent_account_set_id")
                            && let Some(uuid_str) = mv.as_str()
                        {
                            set_id_fields.push((mf.clone(), uuid_str.to_string()));
                        }
                    }
                }
            }
        }

        for (field, uuid_str) in &set_id_fields {
            let Some(set_id) = uuid_str.parse::<Uuid>().ok() else {
                continue;
            };

            // Role = field minus _parent_account_set_id suffix
            let role = field
                .strip_suffix("_parent_account_set_id")
                .unwrap_or(field)
                .to_string();

            // Look up the chart node for this set ID
            let (chart_code, chart_name) = node_by_set_id
                .get(&set_id)
                .map(|n| (n.code.clone(), n.name.clone()))
                .unwrap_or_else(|| ("?".into(), "(no chart node)".into()));

            let mapping = ProductMapping {
                product: product.to_string(),
                role,
                chart_code,
                chart_name,
                chart_parent_set_id: set_id,
            };
            by_parent.entry(set_id).or_default().push(mapping.clone());
            mapping_count += 1;

            // Find product child sets (children that are NOT chart nodes themselves)
            if let Some(children) = set_children_by_parent.get(&set_id) {
                for &child_id in children {
                    if !node_by_set_id.contains_key(&child_id) {
                        by_child.insert(child_id, mapping.clone());
                    }
                }
            }
        }

        config_keys.push((product.to_string(), mapping_count));
    }

    // Sort mappings by role for consistent display
    for mappings in by_parent.values_mut() {
        mappings.sort_by(|a, b| a.role.cmp(&b.role));
    }

    (by_parent, by_child, config_keys)
}

fn compute_account_balances(
    balances_by_account: &HashMap<Uuid, Vec<db::AccountBalanceRow>>,
) -> HashMap<Uuid, Vec<(String, f64, f64)>> {
    let mut result = HashMap::new();
    for (&acct_id, bals) in balances_by_account {
        let mut by_currency: HashMap<String, (f64, f64)> = HashMap::new();
        for b in bals {
            let entry = by_currency.entry(b.currency.clone()).or_insert((0.0, 0.0));
            let pending_dr: f64 = b.pending_dr.parse().unwrap_or(0.0);
            let pending_cr: f64 = b.pending_cr.parse().unwrap_or(0.0);
            let settled_dr: f64 = b.settled_dr.parse().unwrap_or(0.0);
            let settled_cr: f64 = b.settled_cr.parse().unwrap_or(0.0);
            entry.0 += pending_dr - pending_cr;
            entry.1 += settled_dr - settled_cr;
        }
        let mut nets: Vec<(String, f64, f64)> = by_currency
            .into_iter()
            .filter(|(_, (p, s))| *p != 0.0 || *s != 0.0)
            .map(|(c, (p, s))| (c, p, s))
            .collect();
        nets.sort_by(|a, b| a.0.cmp(&b.0));
        if !nets.is_empty() {
            result.insert(acct_id, nets);
        }
    }
    result
}

fn compute_set_balances(
    account_members_by_set: &HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>>,
    balances_by_account: &HashMap<Uuid, Vec<db::AccountBalanceRow>>,
) -> HashMap<Uuid, Vec<(String, f64, f64)>> {
    let mut result = HashMap::new();
    for (&set_id, members) in account_members_by_set {
        let mut by_currency: HashMap<String, (f64, f64)> = HashMap::new();
        for m in members {
            if let Some(bals) = balances_by_account.get(&m.account_id) {
                for b in bals {
                    let entry = by_currency.entry(b.currency.clone()).or_insert((0.0, 0.0));
                    let pending_dr: f64 = b.pending_dr.parse().unwrap_or(0.0);
                    let pending_cr: f64 = b.pending_cr.parse().unwrap_or(0.0);
                    let settled_dr: f64 = b.settled_dr.parse().unwrap_or(0.0);
                    let settled_cr: f64 = b.settled_cr.parse().unwrap_or(0.0);
                    entry.0 += pending_dr - pending_cr;
                    entry.1 += settled_dr - settled_cr;
                }
            }
        }
        let mut nets: Vec<(String, f64, f64)> = by_currency
            .into_iter()
            .filter(|(_, (p, s))| *p != 0.0 || *s != 0.0)
            .map(|(c, (p, s))| (c, p, s))
            .collect();
        nets.sort_by(|a, b| a.0.cmp(&b.0));
        if !nets.is_empty() {
            result.insert(set_id, nets);
        }
    }
    result
}

fn format_number(n: f64) -> String {
    let abs = n.abs();
    let sign = if n < 0.0 { "-" } else { "" };
    let integer = abs.trunc() as u64;
    let frac = ((abs - abs.trunc()) * 100.0).round() as u64;
    let int_str = integer.to_string();
    let mut with_commas = String::new();
    for (i, c) in int_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            with_commas.push(',');
        }
        with_commas.push(c);
    }
    let with_commas: String = with_commas.chars().rev().collect();
    format!("{sign}{with_commas}.{frac:02}")
}

/// Format inline balance numbers for a tree row.
///
/// `tree_indent` is the number of characters the tree widget renders before the label text
/// (highlight area + depth indentation + node symbol). For dump functions where the indent
/// is already included in `label_len`, pass 0.
fn format_balance_suffix(
    label_len: usize,
    balances: &[(String, f64, f64)],
    tree_indent: usize,
) -> String {
    if balances.is_empty() {
        return String::new();
    }
    // Target screen column where the balance numbers should start.
    // Tree widget prefix: 2 (highlight area) + depth*2 + 2 (node symbol) = tree_indent.
    const TARGET_SCREEN_COL: usize = 70;
    let screen_pos = tree_indent + label_len;
    let pad = if screen_pos < TARGET_SCREEN_COL {
        TARGET_SCREEN_COL - screen_pos
    } else {
        2
    };
    let mut suffix = " ".repeat(pad);
    for (i, (currency, pending, settled)) in balances.iter().enumerate() {
        if i > 0 {
            suffix.push_str("  ");
        }
        let p = format_number(*pending);
        let s = format_number(*settled);
        if currency.eq_ignore_ascii_case("usd") {
            suffix.push_str(&format!("{p:>15}  {s:>15}"));
        } else {
            suffix.push_str(&format!("{currency} {p:>15}  {s:>15}"));
        }
    }
    suffix
}

fn build_lana_tree<'a>(
    charts: &[db::ChartRow],
    chart_nodes: &HashMap<Uuid, Vec<db::ChartNodeRow>>,
    product_by_parent_set_id: &HashMap<Uuid, Vec<ProductMapping>>,
    balance_by_set: &HashMap<Uuid, Vec<(String, f64, f64)>>,
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
            product_map: &HashMap<Uuid, Vec<ProductMapping>>,
            bal_map: &HashMap<Uuid, Vec<(String, f64, f64)>>,
            depth: usize,
        ) -> TreeItem<'a, String> {
            let mut label = format!("{} {} ({})", node.code, node.name, node.normal_balance_type);

            // Annotate with product tags
            if let Some(mappings) = product_map.get(&node.account_set_id) {
                let mut products: Vec<&str> = mappings.iter().map(|m| m.product.as_str()).collect();
                products.dedup();
                for p in products {
                    label.push_str(&format!(" [{p}]"));
                }
            }

            // Append inline balance — tree_indent = 4 + depth*2 (highlight + node symbol + indentation)
            if let Some(bals) = bal_map.get(&node.account_set_id) {
                label.push_str(&format_balance_suffix(label.len(), bals, depth * 2 + 4));
            }

            let children: Vec<TreeItem<'a, String>> =
                if let Some(child_nodes) = children_map.get(node.code.as_str()) {
                    let mut sorted = child_nodes.clone();
                    sorted.sort_by(|a, b| a.code.cmp(&b.code));
                    sorted
                        .iter()
                        .map(|c| build_node_item(c, children_map, product_map, bal_map, depth + 1))
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
                .map(|r| build_node_item(r, &children_by_parent, product_by_parent_set_id, balance_by_set, 1))
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
    product_by_child_set_id: &HashMap<Uuid, ProductMapping>,
    show_transitive: bool,
    balance_by_set: &HashMap<Uuid, Vec<(String, f64, f64)>>,
    balance_by_acct: &HashMap<Uuid, Vec<(String, f64, f64)>>,
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
        product_map: &HashMap<Uuid, ProductMapping>,
        show_transitive: bool,
        bal_set: &HashMap<Uuid, Vec<(String, f64, f64)>>,
        bal_acct: &HashMap<Uuid, Vec<(String, f64, f64)>>,
        depth: usize,
    ) -> TreeItem<'a, String> {
        let set = set_by_id.get(&set_id);
        let mut label = set
            .map(|s| s.name.clone())
            .unwrap_or_else(|| set_id.to_string());

        // Annotate with product tag
        if let Some(mapping) = product_map.get(&set_id) {
            label = format!("[{}] {}", mapping.product, label);
        }

        // Append inline balance
        let tree_indent = depth * 2 + 4;
        if let Some(bals) = bal_set.get(&set_id) {
            label.push_str(&format_balance_suffix(label.len(), bals, tree_indent));
        }

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
                    product_map,
                    show_transitive,
                    bal_set,
                    bal_acct,
                    depth + 1,
                ));
            }
        }

        // Add member accounts as leaves
        let acct_indent = (depth + 1) * 2 + 4;
        if let Some(accounts) = account_members.get(&set_id) {
            for acct in accounts {
                if !show_transitive && acct.transitive {
                    continue;
                }
                let acct_ref = acct
                    .account_external_id
                    .as_deref()
                    .unwrap_or(&acct.account_code);
                let mut acct_label = format!(
                    "[acct] {} ({})",
                    acct_ref,
                    if acct.transitive {
                        "transitive"
                    } else {
                        "direct"
                    },
                );
                // Append inline balance for account
                if let Some(bals) = bal_acct.get(&acct.account_id) {
                    acct_label.push_str(&format_balance_suffix(acct_label.len(), bals, acct_indent));
                }
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
                product_by_child_set_id,
                show_transitive,
                balance_by_set,
                balance_by_acct,
                0,
            )
        })
        .collect()
}

fn build_product_tree<'a>(
    product_by_parent_set_id: &HashMap<Uuid, Vec<ProductMapping>>,
    product_config_keys: &[(String, usize)],
    node_by_set_id: &HashMap<Uuid, db::ChartNodeRow>,
    set_children_by_parent: &HashMap<Uuid, Vec<Uuid>>,
    set_by_id: &HashMap<Uuid, db::CalaAccountSetRow>,
    account_members_by_set: &HashMap<Uuid, Vec<db::CalaSetMemberAccountRow>>,
    show_transitive: bool,
    balance_by_set: &HashMap<Uuid, Vec<(String, f64, f64)>>,
    balance_by_acct: &HashMap<Uuid, Vec<(String, f64, f64)>>,
) -> Vec<TreeItem<'a, String>> {
    let mut items = Vec::new();

    for (product_name, _) in product_config_keys {
        // Collect all mappings for this product, grouped by role
        let mut role_mappings: Vec<&ProductMapping> = product_by_parent_set_id
            .values()
            .flatten()
            .filter(|m| &m.product == product_name)
            .collect();
        role_mappings.sort_by(|a, b| a.role.cmp(&b.role));
        // Dedup by role (same role can appear in multiple parent set entries)
        role_mappings.dedup_by(|a, b| a.role == b.role);

        let mut role_items: Vec<TreeItem<'a, String>> = Vec::new();

        for mapping in &role_mappings {
            let mut role_label = format!(
                "{} -> {} {}",
                mapping.role, mapping.chart_code, mapping.chart_name
            );
            // Append inline balance for the chart parent set (role nodes at depth 1)
            if let Some(bals) = balance_by_set.get(&mapping.chart_parent_set_id) {
                role_label.push_str(&format_balance_suffix(role_label.len(), bals, 6));
            }
            let role_id = format!("role:{}:{}", mapping.product, mapping.role);

            // Find product child sets under this chart parent
            let mut child_items: Vec<TreeItem<'a, String>> = Vec::new();
            if let Some(children) = set_children_by_parent.get(&mapping.chart_parent_set_id) {
                for &child_id in children {
                    // Only show non-chart-node children (product sets)
                    if node_by_set_id.contains_key(&child_id) {
                        continue;
                    }
                    let child_name = set_by_id
                        .get(&child_id)
                        .map(|s| s.name.as_str())
                        .unwrap_or("(unknown)");
                    let mut child_label = child_name.to_string();
                    // Append inline balance for child set (depth 2)
                    if let Some(bals) = balance_by_set.get(&child_id) {
                        child_label.push_str(&format_balance_suffix(child_label.len(), bals, 8));
                    }

                    // Show member accounts under this product set
                    let mut acct_items: Vec<TreeItem<'a, String>> = Vec::new();
                    if let Some(accounts) = account_members_by_set.get(&child_id) {
                        for acct in accounts {
                            if !show_transitive && acct.transitive {
                                continue;
                            }
                            let acct_ref = acct
                                .account_external_id
                                .as_deref()
                                .unwrap_or(&acct.account_code);
                            let mut acct_label = format!("[acct] {acct_ref}");
                            // Append inline balance for account (depth 3)
                            if let Some(bals) = balance_by_acct.get(&acct.account_id) {
                                acct_label
                                    .push_str(&format_balance_suffix(acct_label.len(), bals, 10));
                            }
                            let acct_tree_id = format!("acct:{}", acct.account_id);
                            acct_items.push(TreeItem::new_leaf(acct_tree_id, acct_label));
                        }
                    }

                    if acct_items.is_empty() {
                        child_items.push(TreeItem::new_leaf(child_id.to_string(), child_label));
                    } else {
                        child_items.push(
                            TreeItem::new(child_id.to_string(), child_label, acct_items)
                                .expect("duplicate in product tree"),
                        );
                    }
                }
            }

            if child_items.is_empty() {
                role_items.push(TreeItem::new_leaf(role_id, role_label));
            } else {
                role_items.push(
                    TreeItem::new(role_id, role_label, child_items)
                        .expect("duplicate in product tree"),
                );
            }
        }

        let product_label = format!("{product_name} ({} roles)", role_mappings.len());
        let product_id = format!("prod:{product_name}");

        if role_items.is_empty() {
            items.push(TreeItem::new_leaf(product_id, product_label));
        } else {
            items.push(
                TreeItem::new(product_id, product_label, role_items)
                    .expect("duplicate in product tree"),
            );
        }
    }

    items
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
                dump_lana_node(root, &children_map, &app.product_by_parent_set_id, &app.balance_by_set, 1);
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

    // Dump product integrations
    if !app.product_config_keys.is_empty() {
        println!();
        println!("=== Product Integrations ===");
        for (product_name, count) in &app.product_config_keys {
            println!("  {product_name} ({count} roles)");
            let mut mappings: Vec<&ProductMapping> = app
                .product_by_parent_set_id
                .values()
                .flatten()
                .filter(|m| &m.product == product_name)
                .collect();
            mappings.sort_by(|a, b| a.role.cmp(&b.role));
            mappings.dedup_by(|a, b| a.role == b.role);
            for m in mappings {
                println!(
                    "    {} -> {} {} [set:{}]",
                    m.role, m.chart_code, m.chart_name, m.chart_parent_set_id
                );
            }
        }
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
    product_map: &HashMap<Uuid, Vec<ProductMapping>>,
    balance_by_set: &HashMap<Uuid, Vec<(String, f64, f64)>>,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    let mut suffix = String::new();
    if let Some(mappings) = product_map.get(&node.account_set_id) {
        let mut products: Vec<&str> = mappings.iter().map(|m| m.product.as_str()).collect();
        products.dedup();
        for p in products {
            suffix.push_str(&format!(" [{p}]"));
        }
    }
    let label = format!(
        "{}{} {} ({}) [set:{}]{}",
        indent, node.code, node.name, node.normal_balance_type, node.account_set_id, suffix
    );
    if let Some(bals) = balance_by_set.get(&node.account_set_id) {
        let bal_suffix = format_balance_suffix(label.len(), bals, 0);
        println!("{label}{bal_suffix}");
    } else {
        println!("{label}");
    }
    if let Some(children) = children_map.get(node.code.as_str()) {
        let mut sorted = children.clone();
        sorted.sort_by(|a, b| a.code.cmp(&b.code));
        for child in sorted {
            dump_lana_node(child, children_map, product_map, balance_by_set, depth + 1);
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
    let label = format!("{indent}{name} [set:{set_id}]");
    if let Some(bals) = app.balance_by_set.get(&set_id) {
        let bal_suffix = format_balance_suffix(label.len(), bals, 0);
        println!("{label}{bal_suffix}");
    } else {
        println!("{label}");
    }

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
            let acct_ref = acct
                .account_external_id
                .as_deref()
                .unwrap_or(&acct.account_code);
            let acct_label = format!(
                "{}  [acct] {} ({}) [id:{}]",
                indent,
                acct_ref,
                if acct.transitive {
                    "transitive"
                } else {
                    "direct"
                },
                acct.account_id,
            );
            if let Some(bals) = app.balance_by_acct.get(&acct.account_id) {
                let bal_suffix = format_balance_suffix(acct_label.len(), bals, 0);
                println!("{acct_label}{bal_suffix}");
            } else {
                println!("{acct_label}");
            }
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
