use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use syn::visit::Visit;

use crate::{Violation, WorkspaceRule};

use super::service_lint_utils::{
    RepoFindVarCollector, collect_entity_methods, expr_to_simple_ident, scan_rust_files,
};

/// Rule that flags service functions calling `assert_*` entity methods on a
/// variable that is also mutated in the same function.
///
/// When you see `entity.assert_something()?` followed by
/// `entity.do_mutation()`, the assertion should be moved **inside** the
/// entity's mutation method so the invariant is always enforced.
///
/// This rule does **not** support suppression comments — violations must be
/// fixed by moving the assertion into the entity.
pub struct ServiceAssertBeforeMutateRule;

const RULE_NAME: &str = "service-assert-before-mutate";

/// Only `assert_*` methods are flagged.
const ASSERTION_PREFIX: &str = "assert_";

impl ServiceAssertBeforeMutateRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ServiceAssertBeforeMutateRule {
    fn default() -> Self {
        Self::new()
    }
}

fn is_assertion_method(name: &str) -> bool {
    name.starts_with(ASSERTION_PREFIX)
}

// ── Visitors for Phase 2 ─────────────────────────────────────────────

/// Top-level visitor: enters every function, runs the assertion+mutation
/// checker on its body, and continues traversal.
struct ServiceFunctionVisitor<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
    query_methods: &'a HashSet<String>,
    mutation_methods: &'a HashSet<String>,
}

impl<'a> ServiceFunctionVisitor<'a> {
    fn new(
        path: &'a Path,
        query_methods: &'a HashSet<String>,
        mutation_methods: &'a HashSet<String>,
    ) -> Self {
        Self {
            violations: Vec::new(),
            path,
            query_methods,
            mutation_methods,
        }
    }

    fn check_function_body(&mut self, fn_name: &str, block: &syn::Block) {
        // Sub-pass 1: collect entity variables from repo find calls.
        let mut var_collector = RepoFindVarCollector::new();
        var_collector.visit_block(block);

        if var_collector.entity_vars.is_empty() {
            return;
        }

        // Sub-pass 2: collect assertion-style query calls and mutation calls.
        let mut call_collector = EntityCallCollector::new(
            &var_collector.entity_vars,
            self.query_methods,
            self.mutation_methods,
        );
        call_collector.visit_block(block);

        for (var_name, method_name, line) in &call_collector.assertion_calls {
            if call_collector.vars_with_mutations.contains(var_name) {
                self.violations.push(
                    Violation::new(
                        RULE_NAME,
                        self.path.display().to_string(),
                        format!(
                            "in function `{}`: `{}.{}()` validates entity state \
                             but `{}` is also mutated — move the assertion \
                             into the entity's mutation method",
                            fn_name, var_name, method_name, var_name,
                        ),
                    )
                    .with_line(*line),
                );
            }
        }
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

/// Collects assertion-style entity query calls and mutation calls on tracked
/// variables within a single function body.
struct EntityCallCollector<'a> {
    entity_vars: &'a HashSet<String>,
    query_methods: &'a HashSet<String>,
    mutation_methods: &'a HashSet<String>,
    /// Only assertion-prefixed query calls (assert_*).
    assertion_calls: Vec<(String, String, usize)>,
    vars_with_mutations: HashSet<String>,
}

impl<'a> EntityCallCollector<'a> {
    fn new(
        entity_vars: &'a HashSet<String>,
        query_methods: &'a HashSet<String>,
        mutation_methods: &'a HashSet<String>,
    ) -> Self {
        Self {
            entity_vars,
            query_methods,
            mutation_methods,
            assertion_calls: Vec::new(),
            vars_with_mutations: HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for EntityCallCollector<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        if let Some(var_name) = expr_to_simple_ident(&node.receiver)
            && self.entity_vars.contains(&var_name)
        {
            if self.query_methods.contains(&method_name) && is_assertion_method(&method_name) {
                let line = node.method.span().start().line;
                self.assertion_calls
                    .push((var_name.clone(), method_name, line));
            } else if self.mutation_methods.contains(&method_name) {
                self.vars_with_mutations.insert(var_name);
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }

    // Don't descend into nested function definitions.
    fn visit_item_fn(&mut self, _node: &'ast syn::ItemFn) {}
}

// ── WorkspaceRule impl ───────────────────────────────────────────────

impl WorkspaceRule for ServiceAssertBeforeMutateRule {
    fn name(&self) -> &'static str {
        RULE_NAME
    }

    fn description(&self) -> &'static str {
        "Flags assert_* calls on entities that are also mutated in the same service function"
    }

    fn check_workspace(&self, workspace_root: &Path) -> Result<Vec<Violation>> {
        let methods = collect_entity_methods(workspace_root);

        if methods.query.is_empty() && methods.mutation.is_empty() {
            return Ok(Vec::new());
        }

        let query_methods = methods.query;
        let mutation_methods = methods.mutation;
        Ok(scan_rust_files(workspace_root, |path, file, _source| {
            let mut visitor = ServiceFunctionVisitor::new(path, &query_methods, &mutation_methods);
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

    /// Helper: collect both query and mutation methods from entity code.
    fn collect_methods(code: &str) -> (HashSet<String>, HashSet<String>) {
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut collector = EsEntityCollector::new();
        collector.visit_file(&file);

        if collector.es_entity_structs.is_empty() {
            return (HashSet::new(), HashSet::new());
        }

        let mut mc = EntityMethodCollector::new(&collector.es_entity_structs);
        mc.visit_file(&file);
        (mc.query_methods, mc.mutation_methods)
    }

    /// Helper: check service code with both query and mutation method sets.
    fn check_service_code(
        code: &str,
        query_methods: &HashSet<String>,
        mutation_methods: &HashSet<String>,
    ) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut visitor = ServiceFunctionVisitor::new(
            Path::new("test_service.rs"),
            query_methods,
            mutation_methods,
        );
        visitor.visit_file(&file);
        visitor.violations
    }

    // ── Phase 1: mutation collection ─────────────────────────────────

    #[test]
    fn collects_mutation_methods() {
        let (query, mutation) = collect_methods(
            r#"
            #[derive(EsEntity)]
            pub struct Account {
                id: EntityId,
            }
            impl Account {
                pub fn status(&self) -> Status { Status::Active }
                pub fn freeze(&mut self) -> Idempotent<()> { Idempotent::Executed(()) }
                pub fn close(&mut self) -> Result<Idempotent<()>, Error> { Ok(Idempotent::Executed(())) }
            }
        "#,
        );
        assert!(query.contains("status"));
        assert!(!query.contains("freeze"));
        assert!(!query.contains("close"));
        assert!(mutation.contains("freeze"));
        assert!(mutation.contains("close"));
        assert!(!mutation.contains("status"));
    }

    // ── Assertion + mutation co-occurrence ────────────────────────────

    #[test]
    fn flags_assert_when_same_var_is_mutated() {
        let query: HashSet<String> = ["assert_allowed".to_string()].into();
        let mutation: HashSet<String> = ["initiate".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    entity.assert_allowed()?;
                    entity.initiate(data)?;
                    self.repo.update(&mut entity).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("assert_allowed"));
        assert!(violations[0].message.contains("also mutated"));
    }

    #[test]
    fn no_flag_query_only_no_mutation() {
        let query: HashSet<String> = ["assert_valid".to_string()].into();
        let mutation: HashSet<String> = ["close".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let entity = self.repo.find_by_id(id).await?;
                    entity.assert_valid()?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert!(
            violations.is_empty(),
            "Assert without mutation on same var should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_mutation_only_no_assert() {
        let query: HashSet<String> = ["status".to_string()].into();
        let mutation: HashSet<String> = ["close".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    entity.close()?;
                    self.repo.update(&mut entity).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert!(
            violations.is_empty(),
            "Mutation-only (no assert on same var) should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_assert_and_mutation_on_different_vars() {
        let query: HashSet<String> = ["assert_active".to_string()].into();
        let mutation: HashSet<String> = ["close".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let source = self.repo.find_by_id(id1).await?;
                    let mut target = self.repo.find_by_id(id2).await?;
                    source.assert_active()?;
                    target.close()?;
                    self.repo.update(&mut target).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert!(
            violations.is_empty(),
            "Assert and mutation on different vars should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn flags_multiple_asserts_when_var_also_mutated() {
        let query: HashSet<String> =
            ["assert_allowed".to_string(), "assert_valid".to_string()].into();
        let mutation: HashSet<String> = ["execute".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    entity.assert_allowed()?;
                    entity.assert_valid()?;
                    entity.execute(data)?;
                    self.repo.update(&mut entity).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert_eq!(
            violations.len(),
            2,
            "Both assert calls should be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn non_assertion_data_read_with_mutation_not_flagged() {
        let query: HashSet<String> = ["storage_path".to_string()].into();
        let mutation: HashSet<String> = ["delete".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    let path = entity.storage_path();
                    entity.delete()?;
                    self.repo.update(&mut entity).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert!(
            violations.is_empty(),
            "Non-assertion reads should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn check_prefix_with_mutation_not_flagged() {
        let query: HashSet<String> = ["check_payment_date".to_string()].into();
        let mutation: HashSet<String> = ["record_payment".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    entity.check_payment_date(date)?;
                    entity.record_payment(data)?;
                    self.repo.update(&mut entity).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        // check_ is NOT an assertion prefix — only assert_ is
        assert!(
            violations.is_empty(),
            "check_ prefix should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn no_suppression_comment_support() {
        let query: HashSet<String> = ["assert_allowed".to_string()].into();
        let mutation: HashSet<String> = ["initiate".to_string()].into();
        let violations = check_service_code(
            r#"
            impl Svc {
                async fn foo(&self) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id(id).await?;
                    // lint:allow(service-assert-before-mutate)
                    entity.assert_allowed()?;
                    entity.initiate(data)?;
                    self.repo.update(&mut entity).await?;
                    Ok(())
                }
            }
        "#,
            &query,
            &mutation,
        );
        assert_eq!(
            violations.len(),
            1,
            "Suppression comments must NOT work for this rule: {:?}",
            violations
        );
    }
}
