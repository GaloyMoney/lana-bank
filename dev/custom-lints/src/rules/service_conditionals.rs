use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use syn::visit::Visit;

use crate::{Violation, WorkspaceRule};

use super::service_lint_utils::{
    RepoFindVarCollector, collect_entity_methods, expr_to_simple_ident, scan_rust_files,
};

/// Rule that flags the Tell-Don't-Ask anti-pattern in service functions:
///
/// A service inspects entity state (via a `&self` query method) inside an
/// `if` / `match` condition.  The decision should be pushed into the entity
/// instead.
///
/// **Phase 1** – Scan `entity.rs` files, collect `&self` (query) method names
/// from structs that derive `EsEntity`.
///
/// **Phase 2** – Scan remaining Rust source files.  For each function body,
/// track variables that originate from repository `find_*` / `maybe_find_by_*`
/// calls and flag `if` / `match` conditions that call a query method on a
/// tracked variable.
///
/// Suppression: place `// lint:allow(service-conditionals)` on the flagged line
/// or up to 3 lines above it.
pub struct ServiceConditionalsRule;

const RULE_NAME: &str = "service-conditionals";

impl ServiceConditionalsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ServiceConditionalsRule {
    fn default() -> Self {
        Self::new()
    }
}

// ── Visitors for Phase 2 ─────────────────────────────────────────────

/// Top-level visitor: enters every function, runs the conditional checker on
/// its body, and continues traversal so sibling / nested functions are checked.
struct ServiceFunctionVisitor<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
    query_methods: &'a HashSet<String>,
    source: &'a str,
}

impl<'a> ServiceFunctionVisitor<'a> {
    fn new(path: &'a Path, query_methods: &'a HashSet<String>, source: &'a str) -> Self {
        Self {
            violations: Vec::new(),
            path,
            query_methods,
            source,
        }
    }

    fn check_function_body(&mut self, fn_name: &str, block: &syn::Block) {
        // Sub-pass 1: collect entity variables from repo find calls.
        let mut var_collector = RepoFindVarCollector::new();
        var_collector.visit_block(block);

        if var_collector.entity_vars.is_empty() {
            return;
        }

        let source_lines: Vec<&str> = self.source.lines().collect();

        // Sub-pass 2: check conditionals for entity query method calls.
        let mut checker = ConditionalChecker::new(
            self.path,
            fn_name,
            &var_collector.entity_vars,
            self.query_methods,
            &source_lines,
        );
        checker.visit_block(block);
        self.violations.extend(checker.violations);
    }
}

impl<'a> Visit<'a> for ServiceFunctionVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        self.check_function_body(&node.sig.ident.to_string(), &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        self.check_function_body(&node.sig.ident.to_string(), &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Returns `true` when the given 1-indexed line is suppressed via a
/// `// lint:allow(service-conditionals)` comment on the same line or up to
/// 3 lines above (to handle multi-line `if` / `match` conditions where the
/// method call and the `if` keyword are on different lines).
fn is_suppressed(line: usize, source_lines: &[&str]) -> bool {
    for offset in 0..=3 {
        if line > offset {
            let idx = line - 1 - offset;
            if idx < source_lines.len()
                && source_lines[idx].contains("lint:allow(service-conditionals)")
            {
                return true;
            }
        }
    }
    false
}

/// Checks `if` / `match` conditions for entity query method calls on tracked
/// variables.
struct ConditionalChecker<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
    fn_name: String,
    entity_vars: &'a HashSet<String>,
    query_methods: &'a HashSet<String>,
    source_lines: &'a [&'a str],
}

impl<'a> ConditionalChecker<'a> {
    fn new(
        path: &'a Path,
        fn_name: &str,
        entity_vars: &'a HashSet<String>,
        query_methods: &'a HashSet<String>,
        source_lines: &'a [&'a str],
    ) -> Self {
        Self {
            violations: Vec::new(),
            path,
            fn_name: fn_name.to_string(),
            entity_vars,
            query_methods,
            source_lines,
        }
    }

    /// Recursively search an expression for a method call of the form
    /// `<tracked_var>.<entity_query_method>(…)`.  Returns the variable name,
    /// method name, and source line on the first match.
    fn find_entity_query_call(&self, expr: &syn::Expr) -> Option<(String, String, usize)> {
        match expr {
            syn::Expr::MethodCall(mc) => {
                let method_name = mc.method.to_string();

                // Direct call on a tracked entity variable?
                if let Some(var_name) = expr_to_simple_ident(&mc.receiver)
                    && self.entity_vars.contains(&var_name)
                    && self.query_methods.contains(&method_name)
                {
                    let line = mc.method.span().start().line;
                    return Some((var_name, method_name, line));
                }

                // Recurse into receiver and arguments.
                if let Some(r) = self.find_entity_query_call(&mc.receiver) {
                    return Some(r);
                }
                for arg in &mc.args {
                    if let Some(r) = self.find_entity_query_call(arg) {
                        return Some(r);
                    }
                }
                None
            }
            syn::Expr::Binary(b) => self
                .find_entity_query_call(&b.left)
                .or_else(|| self.find_entity_query_call(&b.right)),
            syn::Expr::Unary(u) => self.find_entity_query_call(&u.expr),
            syn::Expr::Paren(p) => self.find_entity_query_call(&p.expr),
            syn::Expr::Reference(r) => self.find_entity_query_call(&r.expr),
            syn::Expr::Call(c) => {
                if let Some(r) = self.find_entity_query_call(&c.func) {
                    return Some(r);
                }
                for arg in &c.args {
                    if let Some(r) = self.find_entity_query_call(arg) {
                        return Some(r);
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl<'a> Visit<'a> for ConditionalChecker<'a> {
    fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
        if let Some((var_name, method_name, line)) = self.find_entity_query_call(&node.cond)
            && !is_suppressed(line, self.source_lines)
        {
            self.violations.push(
                Violation::new(
                    RULE_NAME,
                    self.path.display().to_string(),
                    format!(
                        "in function `{}`: conditional checks `{}.{}()` — \
                         consider moving this logic into the entity (Tell, Don't Ask)",
                        self.fn_name, var_name, method_name,
                    ),
                )
                .with_line(line),
            );
        }
        syn::visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
        if let Some((var_name, method_name, line)) = self.find_entity_query_call(&node.expr)
            && !is_suppressed(line, self.source_lines)
        {
            self.violations.push(
                Violation::new(
                    RULE_NAME,
                    self.path.display().to_string(),
                    format!(
                        "in function `{}`: match on `{}.{}()` — \
                         consider moving this logic into the entity (Tell, Don't Ask)",
                        self.fn_name, var_name, method_name,
                    ),
                )
                .with_line(line),
            );
        }
        syn::visit::visit_expr_match(self, node);
    }

    // Don't descend into nested function definitions.
    fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
}

// ── WorkspaceRule impl ───────────────────────────────────────────────

impl WorkspaceRule for ServiceConditionalsRule {
    fn name(&self) -> &'static str {
        RULE_NAME
    }

    fn description(&self) -> &'static str {
        "Flags service functions that inspect entity state in conditionals (Tell, Don't Ask)"
    }

    fn check_workspace(&self, workspace_root: &Path) -> Result<Vec<Violation>> {
        let methods = collect_entity_methods(workspace_root);

        if methods.query.is_empty() {
            return Ok(Vec::new());
        }

        let query_methods = methods.query;
        Ok(scan_rust_files(workspace_root, |path, file, source| {
            let mut visitor = ServiceFunctionVisitor::new(path, &query_methods, source);
            visitor.visit_file(file);
            visitor.violations
        }))
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::service_lint_utils::{EntityMethodCollector, EsEntityCollector};
    use super::*;

    /// Helper: collect entity query (&self) methods.
    fn collect_methods(code: &str) -> HashSet<String> {
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut collector = EsEntityCollector::new();
        collector.visit_file(&file);

        if collector.es_entity_structs.is_empty() {
            return HashSet::new();
        }

        let mut mc = EntityMethodCollector::new(&collector.es_entity_structs);
        mc.visit_file(&file);
        mc.query_methods
    }

    /// Helper: check service code with a set of entity query methods.
    fn check_service_code(code: &str, query_methods: &HashSet<String>) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut visitor =
            ServiceFunctionVisitor::new(Path::new("test_service.rs"), query_methods, code);
        visitor.visit_file(&file);
        visitor.violations
    }

    // ── Phase 1 tests ────────────────────────────────────────────────

    #[test]
    fn collects_self_methods_only() {
        let methods = collect_methods(
            r#"
            #[derive(EsEntity)]
            pub struct Customer {
                id: EntityId,
            }
            impl Customer {
                pub fn is_closed(&self) -> bool { false }
                pub fn status(&self) -> Status { Status::Active }
                pub fn close(&mut self) -> Idempotent<()> { Idempotent::Executed(()) }
                fn private_query(&self) -> bool { true }
            }
        "#,
        );
        assert!(methods.contains("is_closed"));
        assert!(methods.contains("status"));
        assert!(methods.contains("private_query"));
        assert!(
            !methods.contains("close"),
            "&mut self methods must be excluded"
        );
    }

    #[test]
    fn skips_non_es_entity_structs() {
        let methods = collect_methods(
            r#"
            #[derive(Debug)]
            pub struct Config { value: String }
            impl Config {
                pub fn is_enabled(&self) -> bool { true }
            }
        "#,
        );
        assert!(methods.is_empty());
    }

    #[test]
    fn skips_trait_impl_methods() {
        let methods = collect_methods(
            r#"
            #[derive(EsEntity)]
            pub struct Loan { id: EntityId }
            impl std::fmt::Display for Loan {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "loan")
                }
            }
            impl Loan {
                pub fn is_active(&self) -> bool { true }
            }
        "#,
        );
        assert!(methods.contains("is_active"));
        assert!(
            !methods.contains("fmt"),
            "trait impl methods must be excluded"
        );
    }

    #[test]
    fn collects_from_multiple_entities() {
        let methods = collect_methods(
            r#"
            #[derive(EsEntity)]
            pub struct A { id: EntityId }
            #[derive(EsEntity)]
            pub struct B { id: EntityId }
            impl A { pub fn foo(&self) -> bool { true } }
            impl B { pub fn bar(&self) -> u32 { 0 } }
        "#,
        );
        assert!(methods.contains("foo"));
        assert!(methods.contains("bar"));
    }

    // ── Phase 2 tests ────────────────────────────────────────────────

    #[test]
    fn flags_if_condition_on_entity_query() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customer = self.repo.find_by_id(id).await?;
                    if customer.is_closed() {
                        return Err(Error::Closed);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("is_closed"));
        assert!(violations[0].message.contains("customer"));
    }

    #[test]
    fn flags_match_on_entity_query() {
        let methods: HashSet<String> = ["status".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn bar(&self) -> Result<(), Error> {
                    let entity = self.repo.find_by_id(id).await?;
                    match entity.status() {
                        Status::Active => {},
                        _ => return Err(Error),
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("status"));
    }

    #[test]
    fn flags_negated_condition() {
        let methods: HashSet<String> = ["is_active".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn check(&self) -> Result<(), Error> {
                    let e = self.repo.find_by_id(id).await?;
                    if !e.is_active() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn flags_binary_comparison() {
        let methods: HashSet<String> = ["status".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn check(&self) -> Result<(), Error> {
                    let entity = self.repo.find_by_id(id).await?;
                    if entity.status() == Status::Active {
                        do_something();
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn no_flag_mut_self_method_not_in_registry() {
        // add_member is not in the query_methods set (it would be &mut self)
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    if entity.add_member(id).did_execute() {
                        self.repo.update(&mut entity).await?;
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert!(
            violations.is_empty(),
            "Methods not in the query registry must not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_non_entity_variable() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let config = self.get_config().await?;
                    if config.is_closed() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert!(
            violations.is_empty(),
            "Variables not from repo find calls must not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_function_parameter() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self, customer: Customer) -> Result<(), Error> {
                    if customer.is_closed() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert!(
            violations.is_empty(),
            "Function parameters (not from repo find) must not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn suppression_on_line_above() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customer = self.repo.find_by_id(id).await?;
                    // lint:allow(service-conditionals)
                    if customer.is_closed() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert!(
            violations.is_empty(),
            "Suppression comment should prevent violation: {:?}",
            violations
        );
    }

    #[test]
    fn suppression_on_same_line() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customer = self.repo.find_by_id(id).await?;
                    if customer.is_closed() { // lint:allow(service-conditionals)
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert!(
            violations.is_empty(),
            "Inline suppression should prevent violation: {:?}",
            violations
        );
    }

    #[test]
    fn tracks_maybe_find_by_calls() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customer = self.repo.maybe_find_by_email(email).await?;
                    if customer.is_closed() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn tracks_find_all_calls() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customers = self.repo.find_all().await?;
                    if customers.is_closed() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn tracks_in_op_variant() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customer = self.repo.find_by_id_in_op(op, id).await?;
                    if customer.is_closed() {
                        return Err(Error);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn no_flag_without_conditional() {
        let methods: HashSet<String> = ["status".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<Status, Error> {
                    let entity = self.repo.find_by_id(id).await?;
                    Ok(entity.status())
                }
            }
        "#,
            &methods,
        );
        assert!(
            violations.is_empty(),
            "Using entity method outside conditional is fine: {:?}",
            violations
        );
    }

    #[test]
    fn multiple_violations_in_one_function() {
        let methods: HashSet<String> = ["is_closed".to_string(), "is_frozen".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let customer = self.repo.find_by_id(id).await?;
                    if customer.is_closed() {
                        return Err(Error::Closed);
                    }
                    if customer.is_frozen() {
                        return Err(Error::Frozen);
                    }
                    Ok(())
                }
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn free_function_also_checked() {
        let methods: HashSet<String> = ["is_closed".to_string()].into();
        let violations = check_service_code(
            r#"
            async fn process(repo: &Repo) -> Result<(), Error> {
                let entity = repo.find_by_id(id).await?;
                if entity.is_closed() {
                    return Err(Error);
                }
                Ok(())
            }
        "#,
            &methods,
        );
        assert_eq!(violations.len(), 1);
    }
}
