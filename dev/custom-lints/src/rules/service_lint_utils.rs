use std::collections::HashSet;
use std::path::Path;

use syn::visit::Visit;
use walkdir::WalkDir;

use crate::Violation;

pub(super) const DIRS_TO_CHECK: &[&str] = &["core", "lana", "lib"];

// ── Phase 1: Entity Method Registry ──────────────────────────────────

/// Registry of entity method names, split by receiver type.
pub(super) struct EntityMethods {
    /// `&self` query / assertion methods.
    pub query: HashSet<String>,
    /// `&mut self` mutation methods.
    pub mutation: HashSet<String>,
}

/// Collect `&self` and `&mut self` method names from `EsEntity` structs across
/// `entity.rs` files in the workspace.
pub(super) fn collect_entity_methods(workspace_root: &Path) -> EntityMethods {
    let mut all_query = HashSet::new();
    let mut all_mutation = HashSet::new();

    for dir in DIRS_TO_CHECK {
        let dir_path = workspace_root.join(dir);
        for entry in WalkDir::new(&dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().file_name().is_some_and(|name| name == "entity.rs")
            })
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let parsed = match syn::parse_file(&content) {
                Ok(f) => f,
                Err(_) => continue,
            };

            // Pass 1: find EsEntity structs
            let mut collector = EsEntityCollector::new();
            collector.visit_file(&parsed);

            if collector.es_entity_structs.is_empty() {
                continue;
            }

            // Pass 2: collect &self and &mut self methods on those structs
            let mut mc = EntityMethodCollector::new(&collector.es_entity_structs);
            mc.visit_file(&parsed);
            all_query.extend(mc.query_methods);
            all_mutation.extend(mc.mutation_methods);
        }
    }

    EntityMethods {
        query: all_query,
        mutation: all_mutation,
    }
}

/// First-pass visitor: collect struct names that derive `EsEntity`.
pub(super) struct EsEntityCollector {
    pub es_entity_structs: HashSet<String>,
}

impl EsEntityCollector {
    pub fn new() -> Self {
        Self {
            es_entity_structs: HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for EsEntityCollector {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        for attr in &node.attrs {
            if attr.path().is_ident("derive")
                && let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
            {
                for path in nested {
                    if path.is_ident("EsEntity") {
                        self.es_entity_structs.insert(node.ident.to_string());
                        break;
                    }
                }
            }
        }
        syn::visit::visit_item_struct(self, node);
    }
}

/// Second-pass visitor: collect `&self` and `&mut self` method names from
/// `EsEntity` impl blocks.
pub(super) struct EntityMethodCollector<'a> {
    es_entity_structs: &'a HashSet<String>,
    pub query_methods: HashSet<String>,
    pub mutation_methods: HashSet<String>,
    current_impl_struct: Option<String>,
}

impl<'a> EntityMethodCollector<'a> {
    pub fn new(es_entity_structs: &'a HashSet<String>) -> Self {
        Self {
            es_entity_structs,
            query_methods: HashSet::new(),
            mutation_methods: HashSet::new(),
            current_impl_struct: None,
        }
    }
}

impl<'a> Visit<'a> for EntityMethodCollector<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        // Skip trait impl blocks — only inherent impls define entity queries.
        if node.trait_.is_some() {
            syn::visit::visit_item_impl(self, node);
            return;
        }

        if let syn::Type::Path(type_path) = &*node.self_ty
            && let Some(segment) = type_path.path.segments.last()
        {
            let struct_name = segment.ident.to_string();
            if self.es_entity_structs.contains(&struct_name) {
                self.current_impl_struct = Some(struct_name);
                syn::visit::visit_item_impl(self, node);
                self.current_impl_struct = None;
                return;
            }
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        if self.current_impl_struct.is_some() {
            let (has_immutable_self, has_mut_self) =
                node.sig
                    .inputs
                    .iter()
                    .fold((false, false), |(imm, mt), arg| {
                        if let syn::FnArg::Receiver(receiver) = arg {
                            if receiver.reference.is_some() {
                                if receiver.mutability.is_some() {
                                    (imm, true)
                                } else {
                                    (true, mt)
                                }
                            } else {
                                (imm, mt)
                            }
                        } else {
                            (imm, mt)
                        }
                    });

            let name = node.sig.ident.to_string();
            if has_immutable_self {
                self.query_methods.insert(name);
            } else if has_mut_self {
                self.mutation_methods.insert(name);
            }
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

// ── Phase 2: Shared Helpers ─────────────────────────────────────────

/// Repo find method prefixes that produce entity instances.
const REPO_FIND_PREFIXES: &[&str] = &["find_by_", "maybe_find_by_"];
const REPO_FIND_EXACT: &[&str] = &["find_all"];

fn is_repo_find_method(name: &str) -> bool {
    let base = name.strip_suffix("_in_op").unwrap_or(name);
    REPO_FIND_EXACT.contains(&base)
        || REPO_FIND_PREFIXES
            .iter()
            .any(|prefix| base.starts_with(prefix))
}

/// Walk an expression chain (through `.await` and `?`) to see if it contains a
/// repo find method call.
pub(super) fn contains_repo_find(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Try(try_expr) => contains_repo_find(&try_expr.expr),
        syn::Expr::Await(await_expr) => contains_repo_find(&await_expr.base),
        syn::Expr::MethodCall(mc) => {
            if is_repo_find_method(&mc.method.to_string()) {
                return true;
            }
            contains_repo_find(&mc.receiver)
        }
        _ => false,
    }
}

/// Extract the simple variable name from a `let` pattern.
pub(super) fn extract_var_name(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
        syn::Pat::Type(pt) => extract_var_name(&pt.pat),
        _ => None,
    }
}

/// Extract a simple single-segment identifier from an expression (e.g. a bare
/// variable name like `entity`).
pub(super) fn expr_to_simple_ident(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Path(p) = expr
        && p.path.segments.len() == 1
    {
        return Some(p.path.segments[0].ident.to_string());
    }
    None
}

/// Collects variable names assigned from repo find calls within a single
/// function body.
pub(super) struct RepoFindVarCollector {
    pub entity_vars: HashSet<String>,
}

impl RepoFindVarCollector {
    pub fn new() -> Self {
        Self {
            entity_vars: HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for RepoFindVarCollector {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(init) = &node.init
            && contains_repo_find(&init.expr)
            && let Some(var_name) = extract_var_name(&node.pat)
        {
            self.entity_vars.insert(var_name);
        }
        syn::visit::visit_local(self, node);
    }

    // Don't descend into nested function definitions.
    fn visit_item_fn(&mut self, _node: &'ast syn::ItemFn) {}
}

/// Scan all Rust source files in the workspace directories, calling `check_fn`
/// for each successfully parsed file.  Returns all collected violations.
pub(super) fn scan_rust_files<F>(workspace_root: &Path, mut check_fn: F) -> Vec<Violation>
where
    F: FnMut(&Path, &syn::File, &str) -> Vec<Violation>,
{
    let mut violations = Vec::new();

    for dir in DIRS_TO_CHECK {
        let dir_path = workspace_root.join(dir);
        for entry in WalkDir::new(&dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "rs")
            })
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let parsed = match syn::parse_file(&content) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let relative_path = entry
                .path()
                .strip_prefix(workspace_root)
                .unwrap_or(entry.path());

            violations.extend(check_fn(relative_path, &parsed, &content));
        }
    }

    violations
}
