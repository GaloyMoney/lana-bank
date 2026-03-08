use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{LintRule, Violation};

const RULE_NAME: &str = "tainted-transaction-use";

/// Rule that flags `_in_op` calls on a potentially tainted database transaction.
///
/// When a repo method with `_in_op` suffix fails, the underlying database
/// transaction is tainted and all subsequent operations on it will also fail.
/// This rule detects when code catches/swallows an error from an `_in_op` call
/// and then continues to use the same transaction.
///
/// Specifically it flags:
/// 1. `_in_op` calls inside non-exiting `Err` arms of a `match` on an `_in_op` result
/// 2. `_in_op` calls after a `match` that has a non-exiting `Err` arm
/// 3. `_in_op` calls after error-swallowing chains (`.ok()`, `.unwrap_or*()`)
///    on an `_in_op` result
///
/// Suppression: `// lint:allow(tainted-transaction-use)` on the flagged line
/// or up to 3 lines above.
pub struct TaintedTransactionUseRule;

impl TaintedTransactionUseRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TaintedTransactionUseRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a method name ends with `_in_op`.
fn is_in_op_call(name: &str) -> bool {
    name.ends_with("_in_op")
}

/// Check if a type represents a database operation parameter.
fn is_db_op_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(type_ref) => is_db_op_type(&type_ref.elem),
        syn::Type::Path(type_path) => {
            let path_str: String = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            path_str.contains("DbOp") || path_str.contains("DbOpWithTime")
        }
        syn::Type::ImplTrait(impl_trait) => impl_trait.bounds.iter().any(|bound| {
            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                let path_str: String = trait_bound
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                path_str.contains("AtomicOperation")
            } else {
                false
            }
        }),
        _ => false,
    }
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

/// Recursively check whether an expression contains an awaited `_in_op` method call.
fn expr_contains_in_op_call(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::MethodCall(call) => {
            if is_in_op_call(&call.method.to_string()) {
                return true;
            }
            expr_contains_in_op_call(&call.receiver)
        }
        syn::Expr::Await(aw) => expr_contains_in_op_call(&aw.base),
        syn::Expr::Try(tr) => expr_contains_in_op_call(&tr.expr),
        syn::Expr::Paren(p) => expr_contains_in_op_call(&p.expr),
        syn::Expr::Call(c) => {
            expr_contains_in_op_call(&c.func) || c.args.iter().any(expr_contains_in_op_call)
        }
        _ => false,
    }
}

/// Check if a match arm pattern is an `Err(...)` variant.
fn is_err_pattern(pat: &syn::Pat) -> bool {
    match pat {
        syn::Pat::TupleStruct(ts) => ts
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Err"),
        syn::Pat::Or(or_pat) => or_pat.cases.iter().any(is_err_pattern),
        syn::Pat::Wild(_) => true, // `_` catches Err when Ok is handled explicitly
        _ => false,
    }
}

/// Check if an expression always exits the current control flow
/// (return, break, continue, or a panic/bail macro).
fn expr_always_exits(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_) => true,
        syn::Expr::Block(block) => block_always_exits(&block.block),
        syn::Expr::Macro(mac) => is_exit_macro(&mac.mac),
        _ => false,
    }
}

fn block_always_exits(block: &syn::Block) -> bool {
    block.stmts.last().is_some_and(|stmt| match stmt {
        syn::Stmt::Expr(expr, _) => expr_always_exits(expr),
        syn::Stmt::Macro(mac) => is_exit_macro(&mac.mac),
        _ => false,
    })
}

fn is_exit_macro(mac: &syn::Macro) -> bool {
    mac.path.get_ident().is_some_and(|ident| {
        matches!(
            ident.to_string().as_str(),
            "panic" | "unreachable" | "todo" | "unimplemented" | "bail"
        )
    })
}

/// Check whether a method call chain on an `_in_op` result swallows the error.
/// Detects patterns like `in_op_call().await.ok()`, `.unwrap_or(...)`, etc.
fn is_error_swallowing_chain(expr: &syn::Expr) -> bool {
    if let syn::Expr::MethodCall(mc) = expr {
        let method = mc.method.to_string();
        let swallows = matches!(
            method.as_str(),
            "ok" | "unwrap_or" | "unwrap_or_else" | "unwrap_or_default" | "is_err" | "is_ok"
        );
        if swallows && expr_contains_in_op_call(&mc.receiver) {
            return true;
        }
    }
    false
}

// ── Visitors ────────────────────────────────────────────────────────

/// Finds `begin_op()` calls in a function body.
struct BeginOpFinder {
    found: bool,
}

impl BeginOpFinder {
    fn new() -> Self {
        Self { found: false }
    }
}

impl<'a> Visit<'a> for BeginOpFinder {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        if node.method == "begin_op" {
            self.found = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
}

/// Collects all awaited `_in_op` method calls and their locations.
struct InOpCallCollector {
    calls: Vec<(usize, String)>,
    in_await: bool,
}

impl InOpCallCollector {
    fn new() -> Self {
        Self {
            calls: Vec::new(),
            in_await: false,
        }
    }
}

impl<'a> Visit<'a> for InOpCallCollector {
    fn visit_expr_await(&mut self, node: &'a syn::ExprAwait) {
        let prev = self.in_await;
        self.in_await = true;
        syn::visit::visit_expr_await(self, node);
        self.in_await = prev;
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let name = node.method.to_string();
        if self.in_await && is_in_op_call(&name) {
            self.calls.push((node.method.span().start().line, name));
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
}

/// A point where the transaction becomes potentially tainted.
struct TaintPoint {
    /// Line of the `_in_op` call in the match scrutinee (excluded from flagging).
    scrutinee_line: usize,
    /// End line of the match expression; calls after this line are flagged.
    match_end_line: usize,
    /// `_in_op` calls found inside non-exiting Err arms (immediate violations).
    in_err_arm_calls: Vec<(usize, String)>,
    /// True if all non-exiting Err arms re-initialize the transaction via
    /// `begin_op()`. When true, code after the match is safe.
    reinitializes: bool,
}

/// Walks the AST to find taint points: match expressions on `_in_op` results
/// with non-exiting Err arms, and error-swallowing method chains.
struct TaintPointFinder {
    taint_points: Vec<TaintPoint>,
    /// Lines of error-swallowing chains (`.ok()` etc. on `_in_op` results).
    swallow_lines: Vec<usize>,
}

impl TaintPointFinder {
    fn new() -> Self {
        Self {
            taint_points: Vec::new(),
            swallow_lines: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for TaintPointFinder {
    fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
        if expr_contains_in_op_call(&node.expr) {
            // Check if any Err arm does not exit
            let mut has_non_exiting_err = false;
            let mut in_err_arm_calls = Vec::new();

            // First check if there's an explicit Ok arm (for _ pattern relevance)
            let has_explicit_ok = node.arms.iter().any(|arm| {
                if let syn::Pat::TupleStruct(ts) = &arm.pat {
                    ts.path.segments.last().is_some_and(|seg| seg.ident == "Ok")
                } else {
                    false
                }
            });

            let mut all_non_exiting_reinit = true;

            for arm in &node.arms {
                let is_err = if has_explicit_ok {
                    is_err_pattern(&arm.pat)
                } else {
                    // Without explicit Ok, only flag explicit Err patterns
                    matches!(&arm.pat, syn::Pat::TupleStruct(ts)
                        if ts.path.segments.last()
                            .is_some_and(|seg| seg.ident == "Err"))
                        || matches!(&arm.pat, syn::Pat::Or(or_pat)
                            if or_pat.cases.iter().any(|p| matches!(p, syn::Pat::TupleStruct(ts)
                                if ts.path.segments.last()
                                    .is_some_and(|seg| seg.ident == "Err"))))
                };

                if is_err && !expr_always_exits(&arm.body) {
                    has_non_exiting_err = true;

                    // Check if this arm re-initializes the transaction via begin_op()
                    let mut begin_finder = BeginOpFinder::new();
                    begin_finder.visit_expr(&arm.body);

                    if begin_finder.found {
                        // Arm re-initializes the tx — only flag _in_op calls
                        // BEFORE the begin_op() line within this arm
                        let reinit_line = find_begin_op_line(&arm.body).unwrap_or(0);
                        let mut collector = InOpCallCollector::new();
                        collector.visit_expr(&arm.body);
                        in_err_arm_calls.extend(
                            collector
                                .calls
                                .into_iter()
                                .filter(|(line, _)| *line < reinit_line),
                        );
                    } else {
                        all_non_exiting_reinit = false;
                        // No re-init — all _in_op calls in this arm are tainted
                        let mut collector = InOpCallCollector::new();
                        collector.visit_expr(&arm.body);
                        in_err_arm_calls.extend(collector.calls);
                    }
                }
            }

            if has_non_exiting_err {
                let scrutinee_line = find_in_op_call_line(&node.expr).unwrap_or(0);
                let match_end_line = node.span().end().line;

                self.taint_points.push(TaintPoint {
                    scrutinee_line,
                    match_end_line,
                    in_err_arm_calls,
                    // If all non-exiting Err arms re-initialize, code after the
                    // match is safe (the tx is fresh on every path).
                    reinitializes: all_non_exiting_reinit && has_non_exiting_err,
                });
            }
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        if is_error_swallowing_chain(&syn::Expr::MethodCall(node.clone())) {
            self.swallow_lines.push(node.method.span().start().line);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
}

/// Find the source line of an `_in_op` call within an expression.
fn find_in_op_call_line(expr: &syn::Expr) -> Option<usize> {
    match expr {
        syn::Expr::MethodCall(call) => {
            if is_in_op_call(&call.method.to_string()) {
                Some(call.method.span().start().line)
            } else {
                find_in_op_call_line(&call.receiver)
            }
        }
        syn::Expr::Await(aw) => find_in_op_call_line(&aw.base),
        syn::Expr::Try(tr) => find_in_op_call_line(&tr.expr),
        syn::Expr::Paren(p) => find_in_op_call_line(&p.expr),
        syn::Expr::Call(c) => {
            find_in_op_call_line(&c.func).or_else(|| c.args.iter().find_map(find_in_op_call_line))
        }
        _ => None,
    }
}

/// Find the source line of a `begin_op()` call within an expression.
fn find_begin_op_line(expr: &syn::Expr) -> Option<usize> {
    struct Finder {
        line: Option<usize>,
    }
    impl<'a> Visit<'a> for Finder {
        fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
            if node.method == "begin_op" && self.line.is_none() {
                self.line = Some(node.method.span().start().line);
            }
            syn::visit::visit_expr_method_call(self, node);
        }
        fn visit_item_fn(&mut self, _node: &'a syn::ItemFn) {}
    }
    let mut finder = Finder { line: None };
    finder.visit_expr(expr);
    finder.line
}

// ── Top-level function visitor ──────────────────────────────────────

struct FunctionVisitor<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
}

impl<'a> FunctionVisitor<'a> {
    fn check_function(&mut self, fn_name: &str, sig: &syn::Signature, block: &syn::Block) {
        // Only check functions with a transaction scope
        let has_op_param = has_db_op_parameter(&sig.inputs);
        if !has_op_param {
            let mut finder = BeginOpFinder::new();
            finder.visit_block(block);
            if !finder.found {
                return;
            }
        }

        // Phase 1: find taint points
        let mut taint_finder = TaintPointFinder::new();
        taint_finder.visit_block(block);

        if taint_finder.taint_points.is_empty() && taint_finder.swallow_lines.is_empty() {
            return;
        }

        // Phase 2a: flag _in_op calls inside non-exiting Err arms
        for tp in &taint_finder.taint_points {
            for (line, method_name) in &tp.in_err_arm_calls {
                self.violations.push(
                    Violation::new(
                        RULE_NAME,
                        self.path.display().to_string(),
                        format!(
                            "in function `{}`: `{}` called on potentially tainted transaction \
                             — a prior `_in_op` call in the same transaction may have failed",
                            fn_name, method_name,
                        ),
                    )
                    .with_line(*line),
                );
            }
        }

        // Phase 2b: flag _in_op calls after taint points
        let mut call_collector = InOpCallCollector::new();
        call_collector.visit_block(block);

        // Collect scrutinee lines and err-arm call lines to avoid double-flagging
        let scrutinee_lines: Vec<usize> = taint_finder
            .taint_points
            .iter()
            .map(|tp| tp.scrutinee_line)
            .collect();
        let err_arm_lines: Vec<usize> = taint_finder
            .taint_points
            .iter()
            .flat_map(|tp| tp.in_err_arm_calls.iter().map(|(l, _)| *l))
            .collect();

        for (line, method_name) in &call_collector.calls {
            // Skip the scrutinee call itself and already-flagged err-arm calls
            if scrutinee_lines.contains(line) || err_arm_lines.contains(line) {
                continue;
            }

            let after_match_taint = taint_finder
                .taint_points
                .iter()
                .any(|tp| *line > tp.match_end_line && !tp.reinitializes);
            let after_swallow_taint = taint_finder.swallow_lines.iter().any(|sl| *line > *sl);

            if after_match_taint || after_swallow_taint {
                self.violations.push(
                    Violation::new(
                        RULE_NAME,
                        self.path.display().to_string(),
                        format!(
                            "in function `{}`: `{}` called on potentially tainted transaction \
                             — a prior `_in_op` call in the same transaction may have failed",
                            fn_name, method_name,
                        ),
                    )
                    .with_line(*line),
                );
            }
        }
    }
}

impl<'a> Visit<'a> for FunctionVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        self.check_function(&node.sig.ident.to_string(), &node.sig, &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        self.check_function(&node.sig.ident.to_string(), &node.sig, &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }
}

// ── LintRule impl ───────────────────────────────────────────────────

impl LintRule for TaintedTransactionUseRule {
    fn name(&self) -> &'static str {
        RULE_NAME
    }

    fn description(&self) -> &'static str {
        "Flags _in_op calls on a transaction that may be tainted by a prior failed _in_op call"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        let mut visitor = FunctionVisitor {
            violations: Vec::new(),
            path,
        };
        visitor.visit_file(file);
        visitor.violations
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse test code");
        let rule = TaintedTransactionUseRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    // ── Should flag ──────────────────────────────────────────────────

    #[test]
    fn flags_in_op_inside_non_exiting_err_arm() {
        let code = r#"
            impl Foo {
                async fn bootstrap(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let item = match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => item,
                        Err(e) if e.was_duplicate() => self
                            .repo
                            .find_by_id_in_op(&mut db, id)
                            .await?
                            .expect("must exist"),
                        Err(e) => return Err(e.into()),
                    };
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation for find_by_id_in_op in Err arm: {:?}",
            violations
        );
        assert!(violations[0].message.contains("find_by_id_in_op"));
    }

    #[test]
    fn flags_in_op_after_match_with_non_exiting_err() {
        let code = r#"
            impl Foo {
                async fn bootstrap(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let mut item = match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => item,
                        Err(e) if e.was_duplicate() => default_item(),
                        Err(e) => return Err(e.into()),
                    };
                    self.repo.update_in_op(&mut db, &mut item).await?;
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation for update_in_op after tainted match: {:?}",
            violations
        );
        assert!(violations[0].message.contains("update_in_op"));
    }

    #[test]
    fn flags_both_err_arm_and_after_match() {
        let code = r#"
            impl Foo {
                async fn bootstrap(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    self.audit.record_in_op(&mut db).await?;
                    let mut item = match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => item,
                        Err(e) if e.was_duplicate() => self
                            .repo
                            .find_by_name_in_op(&mut db, name)
                            .await?
                            .expect("must exist"),
                        Err(e) => return Err(e.into()),
                    };
                    if item.add_member(id).did_execute() {
                        self.repo.update_in_op(&mut db, &mut item).await?;
                    }
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            2,
            "Expected 2 violations (err arm + after match): {:?}",
            violations
        );
    }

    #[test]
    fn flags_ok_on_in_op_followed_by_in_op() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let maybe = self.repo.create_in_op(&mut db, data).await.ok();
                    self.repo.update_in_op(&mut db, &mut item).await?;
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            !violations.is_empty(),
            "Expected violation for _in_op after .ok() swallowing: {:?}",
            violations
        );
    }

    // ── Should NOT flag ──────────────────────────────────────────────

    #[test]
    fn no_flag_when_all_err_arms_exit() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let item = match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => item,
                        Err(e) => return Err(e.into()),
                    };
                    self.repo.update_in_op(&mut db, &mut item).await?;
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "All Err arms exit, no taint: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_when_error_propagated_with_question_mark() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let item = self.repo.create_in_op(&mut db, data).await?;
                    self.repo.update_in_op(&mut db, &mut item).await?;
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "? propagates error, no taint: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_without_transaction_scope() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let item = match self.repo.create(data).await {
                        Ok(item) => item,
                        Err(e) => default_item(),
                    };
                    self.repo.update(&mut item).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "No transaction scope, no taint: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_when_err_arm_uses_non_in_op_call() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => {
                            db.commit().await?;
                            Ok(item)
                        }
                        Err(e) if e.is_duplicate() => {
                            let item = self.repo.find_by_id(id).await?;
                            Ok(item)
                        }
                        Err(e) => Err(e.into()),
                    }
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Err arm uses fresh connection (non _in_op), no taint: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_with_db_op_param_but_no_taint() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    db: &mut es_entity::DbOp<'_>,
                ) -> Result<(), Error> {
                    let item = self.repo.find_by_id_in_op(db, id).await?;
                    self.repo.update_in_op(db, &mut item).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "All errors propagated with ?, no taint: {:?}",
            violations
        );
    }

    #[test]
    fn flags_with_db_op_param_and_taint() {
        let code = r#"
            impl Foo {
                async fn process_in_op(
                    &self,
                    db: &mut es_entity::DbOp<'_>,
                ) -> Result<(), Error> {
                    let item = match self.repo.create_in_op(db, data).await {
                        Ok(item) => item,
                        Err(_) => default_item(),
                    };
                    self.repo.update_in_op(db, &mut item).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected violation for update_in_op after tainted match: {:?}",
            violations
        );
    }

    #[test]
    fn no_flag_when_err_arm_reinitializes_tx() {
        // If the Err arm calls begin_op() to re-initialize the transaction,
        // subsequent _in_op calls in the arm (after begin_op) and after the
        // match are safe.
        let code = r#"
            impl Foo {
                async fn bootstrap(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let mut item = match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => item,
                        Err(e) if e.was_duplicate() => {
                            db = self.repo.begin_op().await?;
                            self.repo
                                .find_by_name_in_op(&mut db, name)
                                .await?
                                .expect("must exist")
                        }
                        Err(e) => return Err(e.into()),
                    };
                    self.repo.update_in_op(&mut db, &mut item).await?;
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "begin_op() in Err arm re-initializes tx, no taint: {:?}",
            violations
        );
    }

    #[test]
    fn flags_in_op_before_reinit_in_err_arm() {
        // _in_op calls BEFORE begin_op() in the Err arm are still tainted
        let code = r#"
            impl Foo {
                async fn bootstrap(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    let item = match self.repo.create_in_op(&mut db, data).await {
                        Ok(item) => item,
                        Err(e) if e.was_duplicate() => {
                            let old = self.repo.find_by_id_in_op(&mut db, id).await?;
                            db = self.repo.begin_op().await?;
                            self.repo
                                .find_by_name_in_op(&mut db, name)
                                .await?
                                .expect("must exist")
                        }
                        Err(e) => return Err(e.into()),
                    };
                    db.commit().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation for find_by_id_in_op before reinit: {:?}",
            violations
        );
        assert!(violations[0].message.contains("find_by_id_in_op"));
    }
}
