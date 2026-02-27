use std::path::Path;

use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that ensures EsRepo methods use `_in_op` variants when a database transaction is in scope.
///
/// When a function has a `DbOp`, `DbOpWithTime`, or `impl AtomicOperation` parameter,
/// or calls `begin_op()`, all EsRepo method calls within that function should use
/// the `_in_op` variant to reuse the existing transaction connection instead of
/// checking out a new connection from the pool.
///
/// Only flags async (`.await`ed) method calls to avoid false positives on
/// synchronous entity mutation methods like `entity.update(...)`.
pub struct RepoInOpUsageRule;

impl RepoInOpUsageRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RepoInOpUsageRule {
    fn default() -> Self {
        Self::new()
    }
}

/// EsRepo-generated exact method names that have `_in_op` variants.
const REPO_EXACT_METHODS: &[&str] = &[
    "find_all",
    "create",
    "create_all",
    "update",
    "update_all",
    "delete",
];

/// EsRepo-generated method name prefixes that have `_in_op` variants.
/// Covers dynamically-named methods generated from column definitions:
///   find_by_{column}, maybe_find_by_{column},
///   list_by_{column}, list_for_{column}_by_{sort},
///   list_for_filters_by_{sort}
const REPO_PREFIX_METHODS: &[&str] = &["find_by_", "maybe_find_by_", "list_by_", "list_for_"];

/// Check if a method name matches a known EsRepo-generated method pattern
/// that should use an `_in_op` variant when a transaction is in scope.
fn is_repo_method_without_in_op(name: &str) -> bool {
    if name.ends_with("_in_op") {
        return false;
    }

    if REPO_EXACT_METHODS.contains(&name) {
        return true;
    }

    if REPO_PREFIX_METHODS
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return true;
    }

    false
}

/// Check if a type represents a database operation
fn is_db_op_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(type_ref) => is_db_op_type(&type_ref.elem),
        syn::Type::Path(type_path) => {
            let path_str = path_to_string(&type_path.path);
            path_str.contains("DbOp") || path_str.contains("DbOpWithTime")
        }
        syn::Type::ImplTrait(impl_trait) => {
            for bound in &impl_trait.bounds {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    let path_str = path_to_string(&trait_bound.path);
                    if path_str.contains("AtomicOperation") {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn has_db_op_parameter(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> bool {
    inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_db_op_type(&pat_type.ty)
        } else {
            false
        }
    })
}

struct FunctionVisitor<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
}

impl<'a> FunctionVisitor<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            violations: Vec::new(),
            path,
        }
    }

    fn check_function(&mut self, fn_name: &str, sig: &syn::Signature, block: &syn::Block) {
        let has_op_param = has_db_op_parameter(&sig.inputs);

        // For functions with a DbOp parameter, the entire body is in transaction scope.
        // For functions that call begin_op(), we flag everything:
        //   - Calls after begin_op() should use _in_op variant
        //   - Calls before begin_op() should be reordered to after begin_op() and use _in_op
        //   Exception: calls before begin_op() inside a loop are not flagged (common pattern
        //   of reading data in a loop to decide whether to start a transaction).
        let (begin_op_line, max_line) = if has_op_param {
            (Some(0), None) // Flag everything, no begin_op line distinction needed
        } else {
            let mut checker = BeginOpChecker::new();
            checker.visit_block(block);
            if checker.begin_op_line.is_none() {
                return;
            }
            (checker.begin_op_line, checker.last_commit_line)
        };

        let begin_op_line = begin_op_line.unwrap();

        let mut call_checker = RepoCallChecker::new(self.path, fn_name, begin_op_line, max_line);
        call_checker.visit_block(block);
        self.violations.extend(call_checker.violations);
    }
}

/// Visitor to find the earliest begin_op() and latest commit() call lines in a block
struct BeginOpChecker {
    begin_op_line: Option<usize>,
    last_commit_line: Option<usize>,
}

impl BeginOpChecker {
    fn new() -> Self {
        Self {
            begin_op_line: None,
            last_commit_line: None,
        }
    }
}

impl<'a> Visit<'a> for BeginOpChecker {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        let line = node.method.span().start().line;
        if method_name == "begin_op" {
            self.begin_op_line = Some(
                self.begin_op_line
                    .map_or(line, |existing| existing.min(line)),
            );
        } else if method_name == "commit" {
            self.last_commit_line = Some(
                self.last_commit_line
                    .map_or(line, |existing| existing.max(line)),
            );
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
}

/// Visitor that finds awaited repo method calls that should use _in_op variants
struct RepoCallChecker<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
    fn_name: String,
    /// The line of the `begin_op()` call (0 for functions with a DbOp parameter).
    begin_op_line: usize,
    /// Only flag method calls on or before this line number (the last commit() line).
    /// None means no upper bound.
    max_line: Option<usize>,
    /// Whether we are currently inside an `.await` expression.
    /// Only `.await`ed calls are repo calls; sync calls like `entity.update()` are not.
    in_await: bool,
    /// Depth of loop nesting (for/while/loop). Calls before begin_op inside
    /// a loop are exempt because the pattern of reading data in a loop body
    /// to decide whether to start a transaction is common.
    loop_depth: usize,
}

impl<'a> RepoCallChecker<'a> {
    fn new(path: &'a Path, fn_name: &str, begin_op_line: usize, max_line: Option<usize>) -> Self {
        Self {
            violations: Vec::new(),
            path,
            fn_name: fn_name.to_string(),
            begin_op_line,
            max_line,
            in_await: false,
            loop_depth: 0,
        }
    }
}

impl<'a> Visit<'a> for RepoCallChecker<'a> {
    fn visit_expr_await(&mut self, node: &'a syn::ExprAwait) {
        let prev = self.in_await;
        self.in_await = true;
        syn::visit::visit_expr_await(self, node);
        self.in_await = prev;
    }

    fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
        self.loop_depth += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.loop_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
        self.loop_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.loop_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
        self.loop_depth += 1;
        syn::visit::visit_expr_loop(self, node);
        self.loop_depth -= 1;
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        if self.in_await && is_repo_method_without_in_op(&method_name) {
            let call_line = node.method.span().start().line;
            let before_begin_op = call_line <= self.begin_op_line;
            let in_loop = self.loop_depth > 0;

            // Skip calls that are before begin_op AND inside a loop
            // (common pattern: read in loop, conditionally begin transaction)
            if before_begin_op && in_loop {
                syn::visit::visit_expr_method_call(self, node);
                return;
            }

            let before_commit = self.max_line.is_none_or(|max| call_line <= max);
            if before_commit {
                let hint = if before_begin_op {
                    format!(
                        "in function `{}`: repo method `{}` called before `begin_op()` — \
                         move it after `begin_op()` and use `{}_in_op`",
                        self.fn_name, method_name, method_name,
                    )
                } else {
                    format!(
                        "in function `{}`: repo method `{}` should use `{}_in_op` \
                         when a database transaction is in scope",
                        self.fn_name, method_name, method_name,
                    )
                };
                self.violations.push(
                    Violation::new("repo-in-op-usage", self.path.display().to_string(), hint)
                        .with_line(call_line),
                );
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
}

impl<'a> Visit<'a> for FunctionVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.check_function(&fn_name, &node.sig, &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.check_function(&fn_name, &node.sig, &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }
}

impl LintRule for RepoInOpUsageRule {
    fn name(&self) -> &'static str {
        "repo-in-op-usage"
    }

    fn description(&self) -> &'static str {
        "Ensures EsRepo methods use _in_op variants when a database transaction is in scope"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        let mut visitor = FunctionVisitor::new(path);
        visitor.visit_file(file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse test code");
        let rule = RepoInOpUsageRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    #[test]
    fn test_no_transaction_no_violation() {
        let code = r#"
            impl Foo {
                async fn get_item(&self, id: ItemId) -> Result<Item, Error> {
                    self.repo.find_by_id(id).await
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Expected no violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_db_op_param_with_in_op_call() {
        let code = r#"
            impl Foo {
                async fn create_in_op(
                    &self,
                    db: &mut es_entity::DbOp<'_>,
                    data: NewItem,
                ) -> Result<Item, Error> {
                    self.repo.create_in_op(db, data).await
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Expected no violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_db_op_param_with_non_in_op_call() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    db: &mut es_entity::DbOp<'_>,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let mut item = self.repo.find_by_id(id).await?;
                    self.repo.update_in_op(db, &mut item).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("find_by_id"));
        assert!(violations[0].message.contains("find_by_id_in_op"));
    }

    #[test]
    fn test_begin_op_with_non_in_op_call_after() {
        let code = r#"
            impl Foo {
                async fn process(&self, id: ItemId) -> Result<(), Error> {
                    let mut op = self.repo.begin_op().await?;
                    let item = self.other_repo.find_by_id(id).await?;
                    self.repo.update_in_op(&mut op, &mut item).await?;
                    op.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("find_by_id"));
    }

    #[test]
    fn test_begin_op_call_before_transaction_flagged() {
        // Calls before begin_op() should be flagged — they should be reordered after begin_op
        let code = r#"
            impl Foo {
                async fn process(&self, id: ItemId) -> Result<(), Error> {
                    let mut item = self.repo.find_by_id(id).await?;
                    if item.needs_update() {
                        let mut op = self.repo.begin_op().await?;
                        self.repo.update_in_op(&mut op, &mut item).await?;
                        op.commit().await?;
                    }
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Calls before begin_op should be flagged for reordering: {:?}",
            violations
        );
        assert!(
            violations[0].message.contains("before `begin_op()`"),
            "Message should suggest reordering: {}",
            violations[0].message
        );
    }

    #[test]
    fn test_begin_op_all_in_op_calls() {
        let code = r#"
            impl Foo {
                async fn process(&self, id: ItemId) -> Result<(), Error> {
                    let mut op = self.repo.begin_op().await?;
                    let mut item = self.other_repo.find_by_id_in_op(&mut op, id).await?;
                    self.repo.update_in_op(&mut op, &mut item).await?;
                    op.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Expected no violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_multiple_violations() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    op: &mut es_entity::DbOp<'_>,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let item = self.repo.find_by_id(id).await?;
                    let all = self.repo.find_all().await?;
                    self.repo.update(&mut item).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            3,
            "Expected 3 violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_list_for_method() {
        let code = r#"
            impl Foo {
                async fn list_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    holder_id: HolderId,
                ) -> Result<Vec<Item>, Error> {
                    self.repo
                        .list_for_holder_id_by_created_at(holder_id, Default::default(), Default::default())
                        .await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(
            violations[0]
                .message
                .contains("list_for_holder_id_by_created_at")
        );
    }

    #[test]
    fn test_list_by_id_method() {
        let code = r#"
            impl Foo {
                async fn fetch_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                ) -> Result<Vec<Item>, Error> {
                    self.repo.list_by_id(Default::default(), Default::default()).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("list_by_id"));
    }

    #[test]
    fn test_find_by_column_method() {
        let code = r#"
            impl Foo {
                async fn lookup_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    email: &str,
                ) -> Result<Item, Error> {
                    self.repo.find_by_email(email).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("find_by_email"));
    }

    #[test]
    fn test_maybe_find_by_column_method() {
        let code = r#"
            impl Foo {
                async fn lookup_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    ref_id: &str,
                ) -> Result<Option<Item>, Error> {
                    self.repo.maybe_find_by_reference(ref_id).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("maybe_find_by_reference"));
    }

    #[test]
    fn test_list_by_column_method() {
        let code = r#"
            impl Foo {
                async fn fetch_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                ) -> Result<Vec<Item>, Error> {
                    self.repo.list_by_name(Default::default(), Default::default()).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("list_by_name"));
    }

    #[test]
    fn test_delete_method() {
        let code = r#"
            impl Foo {
                async fn remove_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    entity: Item,
                ) -> Result<(), Error> {
                    self.repo.delete(entity).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("delete"));
    }

    #[test]
    fn test_list_for_filters_method() {
        let code = r#"
            impl Foo {
                async fn search_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    filters: Filters,
                ) -> Result<Vec<Item>, Error> {
                    self.repo.list_for_filters_by_id(filters, Default::default(), Default::default()).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("list_for_filters_by_id"));
    }

    #[test]
    fn test_impl_atomic_operation_param() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    op: &mut impl AtomicOperation,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let item = self.repo.find_by_id(id).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
    }

    #[test]
    fn test_db_op_with_time_param() {
        let code = r#"
            impl Foo {
                async fn complete_in_op(
                    &self,
                    db: &mut es_entity::DbOpWithTime<'_>,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let item = self.repo.find_by_id(id).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
    }

    #[test]
    fn test_trait_impl_bodies_checked() {
        let code = r#"
            impl SomeTrait for Foo {
                async fn handle(
                    &self,
                    op: &mut es_entity::DbOp<'_>,
                ) -> Result<(), Error> {
                    let item = self.repo.find_by_id(id).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Trait impl bodies should be checked: {:?}",
            violations
        );
    }

    #[test]
    fn test_create_all_violation() {
        let code = r#"
            impl Foo {
                async fn batch_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    items: Vec<NewItem>,
                ) -> Result<Vec<Item>, Error> {
                    self.repo.create_all(items).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("create_all"));
    }

    #[test]
    fn test_update_all_violation() {
        let code = r#"
            impl Foo {
                async fn batch_update_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    items: &mut [Item],
                ) -> Result<(), Error> {
                    self.repo.update_all(items).await
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("update_all"));
    }

    #[test]
    fn test_non_repo_method_no_violation() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                ) -> Result<(), Error> {
                    let result = self.service.do_something().await?;
                    self.logger.log("done").await;
                    let x = compute_value();
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Non-repo methods should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_sync_entity_update_not_flagged() {
        // Synchronous entity mutation methods should NOT be flagged
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let mut entity = self.repo.find_by_id_in_op(&mut *op, id).await?;
                    entity.update(new_values);
                    entity.create(data);
                    self.repo.update_in_op(&mut *op, &mut entity).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Sync entity methods should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_sync_update_with_chained_call_not_flagged() {
        // Pattern: entity.update(values).did_execute() - sync, not a repo call
        let code = r#"
            impl Foo {
                async fn close_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                ) -> Result<(), Error> {
                    if tracking_account_set.update(update_values).did_execute() {
                        self.cala.account_sets().persist_in_op(op, &mut tracking_account_set).await?;
                    }
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Sync chained calls should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_free_function_with_begin_op() {
        let code = r#"
            async fn process(repo: &Repo, id: ItemId) -> Result<(), Error> {
                let mut op = repo.begin_op().await?;
                let item = repo.find_by_id(id).await?;
                op.commit().await?;
                Ok(())
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
    }

    #[test]
    fn test_free_function_with_db_op_param() {
        let code = r#"
            async fn process_in_op(
                op: &mut DbOp<'_>,
                repo: &Repo,
                id: ItemId,
            ) -> Result<(), Error> {
                let item = repo.maybe_find_by_id(id).await?;
                Ok(())
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("maybe_find_by_id"));
    }

    #[test]
    fn test_in_op_suffix_not_flagged() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let item = self.repo.find_by_id_in_op(&mut *op, id).await?;
                    let all = self.repo.list_for_holder_by_date_in_op(&mut *op, id, Default::default(), Default::default()).await?;
                    self.repo.create_in_op(&mut *op, new_item).await?;
                    self.repo.update_in_op(&mut *op, &mut item).await?;
                    self.repo.create_all_in_op(&mut *op, items).await?;
                    self.repo.update_all_in_op(&mut *op, &mut items).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "All _in_op calls should pass: {:?}",
            violations
        );
    }

    #[test]
    fn test_begin_op_call_after_commit_not_flagged() {
        // Pattern: begin_op + commit in one match arm, find_by_id in sibling arm (after commit)
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<Item, Error> {
                    match result {
                        Idempotent::Executed(new_item) => {
                            let mut db = self.repo.begin_op().await?;
                            let item = self.repo.create_in_op(&mut db, new_item).await?;
                            db.commit().await?;
                            Ok(item)
                        }
                        Idempotent::AlreadyApplied => {
                            Ok(self.repo.find_by_id(id).await?)
                        }
                    }
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Calls after commit in sibling match arm should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_begin_op_call_between_begin_and_commit_flagged() {
        // Calls between begin_op and commit should still be flagged
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut op = self.repo.begin_op().await?;
                    let item = self.other_repo.find_by_id(id).await?;
                    self.repo.update_in_op(&mut op, &mut item).await?;
                    op.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation between begin_op and commit: {:?}",
            violations
        );
    }

    #[test]
    fn test_begin_op_call_before_in_loop_not_flagged() {
        // Calls before begin_op() inside a loop are exempt
        let code = r#"
            impl Foo {
                async fn process(&self, ids: Vec<ItemId>) -> Result<(), Error> {
                    for id in ids {
                        let item = self.repo.find_by_id(id).await?;
                        if item.needs_update() {
                            let mut op = self.repo.begin_op().await?;
                            self.repo.update_in_op(&mut op, &mut item).await?;
                            op.commit().await?;
                        }
                    }
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Calls before begin_op inside a loop should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_begin_op_call_before_in_while_loop_not_flagged() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    while let Some(id) = queue.pop() {
                        let item = self.repo.find_by_id(id).await?;
                        if item.needs_update() {
                            let mut op = self.repo.begin_op().await?;
                            self.repo.update_in_op(&mut op, &mut item).await?;
                            op.commit().await?;
                        }
                    }
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Calls before begin_op inside a while loop should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_begin_op_call_before_in_bare_loop_not_flagged() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    loop {
                        let item = self.repo.find_by_id(id).await?;
                        let mut op = self.repo.begin_op().await?;
                        self.repo.update_in_op(&mut op, &mut item).await?;
                        op.commit().await?;
                    }
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Calls before begin_op inside a bare loop should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_begin_op_after_in_loop_still_flagged() {
        // Calls AFTER begin_op inside a loop should still be flagged
        let code = r#"
            impl Foo {
                async fn process(&self, ids: Vec<ItemId>) -> Result<(), Error> {
                    for id in ids {
                        let mut op = self.repo.begin_op().await?;
                        let item = self.repo.find_by_id(id).await?;
                        self.repo.update_in_op(&mut op, &mut item).await?;
                        op.commit().await?;
                    }
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Calls after begin_op inside a loop should be flagged: {:?}",
            violations
        );
        assert!(violations[0].message.contains("find_by_id"));
    }

    #[test]
    fn test_mixed_correct_and_incorrect() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    op: &mut DbOp<'_>,
                    id: ItemId,
                ) -> Result<(), Error> {
                    let mut item = self.repo.find_by_id(id).await?;
                    self.repo.update_in_op(&mut *op, &mut item).await?;
                    let others = self.other_repo.list_for_parent_by_created_at_in_op(
                        &mut *op,
                        id,
                        Default::default(),
                        Default::default(),
                    ).await?;
                    self.other_repo.create(new_item).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            2,
            "Expected 2 violations (find_by_id and create): {:?}",
            violations
        );
    }
}
