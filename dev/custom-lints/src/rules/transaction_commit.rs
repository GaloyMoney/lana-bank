use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that ensures every function calling `begin_op()` also calls `commit()`.
///
/// Database transactions started with `begin_op()` must be committed with `commit()`.
/// Failing to commit a transaction can lead to data loss or inconsistent state.
pub struct TransactionCommitRule;

impl TransactionCommitRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransactionCommitRule {
    fn default() -> Self {
        Self::new()
    }
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

    fn check_function_body(&mut self, fn_name: &str, block: &syn::Block, start_line: usize) {
        let mut body_checker = BodyChecker::new();
        body_checker.visit_block(block);

        // If begin_op is called but commit is not, report a violation
        if body_checker.has_begin_op && !body_checker.has_commit {
            // Allowed pattern: function returns a DbOp (wrapper functions or Job pattern)
            if !body_checker.returns_db_op {
                self.violations.push(
                    Violation::new(
                        "transaction-commit",
                        self.path.display().to_string(),
                        format!(
                            "function `{}` calls `begin_op()` but never calls `commit()`",
                            fn_name
                        ),
                    )
                    .with_line(start_line),
                );
            }
        }
    }
}

impl<'a> Visit<'a> for FunctionVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        let fn_name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        self.check_function_body(&fn_name, &node.block, start_line);

        // Continue visiting nested items
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        self.check_function_body(&fn_name, &node.block, start_line);

        // Continue visiting nested items
        syn::visit::visit_impl_item_fn(self, node);
    }
}

struct BodyChecker {
    has_begin_op: bool,
    has_commit: bool,
    /// True if the DbOp is returned from the function (wrapper pattern or Job pattern)
    returns_db_op: bool,
}

impl BodyChecker {
    fn new() -> Self {
        Self {
            has_begin_op: false,
            has_commit: false,
            returns_db_op: false,
        }
    }

    fn check_return_expr(&mut self, expr: &syn::Expr) {
        if Self::expr_returns_db_op(expr) {
            self.returns_db_op = true;
        }
    }

    /// Heuristic to detect if an expression returns a DbOp.
    /// Looks for patterns like:
    /// - `self.repo.begin_op().await` - direct return of begin_op result
    /// - `Ok(self.repo.begin_op().await?)` - wrapped in Ok
    /// - `Ok(JobCompletion::CompleteWithOp(op))` - DbOp wrapped in enum variant
    fn expr_returns_db_op(expr: &syn::Expr) -> bool {
        match expr {
            // Handle `Ok(...)` or other enum variant wrapping
            syn::Expr::Call(call) => {
                // Check if any argument contains a DbOp being returned
                if call.args.iter().any(Self::expr_returns_db_op) {
                    return true;
                }
                // Check for Ok(begin_op result)
                if let syn::Expr::Path(path) = &*call.func
                    && path.path.is_ident("Ok")
                    && let Some(arg) = call.args.first()
                {
                    return Self::expr_returns_db_op(arg) || Self::expr_contains_db_op_var(arg);
                }
                // Check if this is an enum variant containing a DbOp variable
                Self::expr_contains_db_op_var(expr)
            }
            // Handle `.await` and `?` on begin_op
            syn::Expr::Await(await_expr) => Self::expr_contains_begin_op_return(&await_expr.base),
            syn::Expr::Try(try_expr) => Self::expr_returns_db_op(&try_expr.expr),
            // Direct variable that might be a DbOp
            syn::Expr::Path(path) => Self::is_db_op_variable(path),
            _ => false,
        }
    }

    fn expr_contains_begin_op_return(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::MethodCall(call) => {
                call.method == "begin_op" || Self::expr_contains_begin_op_return(&call.receiver)
            }
            syn::Expr::Try(try_expr) => Self::expr_contains_begin_op_return(&try_expr.expr),
            syn::Expr::Await(await_expr) => Self::expr_contains_begin_op_return(&await_expr.base),
            _ => false,
        }
    }

    /// Check if expression contains a DbOp variable (for return patterns like `CompleteWithOp(op)`)
    fn expr_contains_db_op_var(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Call(call) => call.args.iter().any(Self::expr_contains_db_op_var),
            syn::Expr::Path(path) => Self::is_db_op_variable(path),
            syn::Expr::Tuple(t) => t.elems.iter().any(Self::expr_contains_db_op_var),
            syn::Expr::Struct(s) => s
                .fields
                .iter()
                .any(|f| Self::expr_contains_db_op_var(&f.expr)),
            _ => false,
        }
    }

    /// Check if a path expression is a DbOp variable
    fn is_db_op_variable(path: &syn::ExprPath) -> bool {
        if let Some(ident) = path.path.get_ident() {
            let name = ident.to_string();
            // Common names for DbOp variables
            name == "op" || name == "db" || name == "db_tx" || name == "db_op"
        } else {
            false
        }
    }
}

impl<'a> Visit<'a> for BodyChecker {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        if method_name.starts_with("begin_op") {
            self.has_begin_op = true;
        } else if method_name == "commit" {
            self.has_commit = true;
        }

        // Continue visiting to find nested calls
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        // Check for function-style calls like `begin_op()` or path calls
        if let syn::Expr::Path(path) = &*node.func
            && let Some(segment) = path.path.segments.last()
        {
            let fn_name = segment.ident.to_string();
            if fn_name.starts_with("begin_op") {
                self.has_begin_op = true;
            } else if fn_name == "commit" {
                self.has_commit = true;
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_return(&mut self, node: &'a syn::ExprReturn) {
        // Check explicit return statements for DbOp being returned
        if let Some(expr) = &node.expr {
            self.check_return_expr(expr);
        }
        syn::visit::visit_expr_return(self, node);
    }

    fn visit_block(&mut self, node: &'a syn::Block) {
        // Check the last statement if it's an expression (tail expression)
        if let Some(syn::Stmt::Expr(expr, None)) = node.stmts.last() {
            self.check_return_expr(expr);
        }
        syn::visit::visit_block(self, node);
    }
}

impl LintRule for TransactionCommitRule {
    fn name(&self) -> &'static str {
        "transaction-commit"
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
        let rule = TransactionCommitRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    #[test]
    fn test_valid_begin_op_with_commit() {
        let code = r#"
            impl Foo {
                async fn record(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    self.do_something(&mut db).await?;
                    db.commit().await?;
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
    fn test_missing_commit() {
        let code = r#"
            impl Foo {
                async fn record(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    self.do_something(&mut db).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("begin_op"));
        assert!(violations[0].message.contains("commit"));
    }

    #[test]
    fn test_wrapper_function_returning_db_op() {
        let code = r#"
            impl Foo {
                async fn begin_op(&self) -> Result<DbOp, Error> {
                    Ok(self.repo.begin_op().await?)
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Wrapper functions should be allowed: {:?}",
            violations
        );
    }

    #[test]
    fn test_no_begin_op_no_violation() {
        let code = r#"
            impl Foo {
                async fn other_method(&self) -> Result<(), Error> {
                    self.do_something().await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_free_function_with_missing_commit() {
        let code = r#"
            async fn process_data(repo: &Repo) -> Result<(), Error> {
                let mut db = repo.begin_op().await?;
                do_work(&mut db).await?;
                Ok(())
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_commit_in_all_branches() {
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    if condition {
                        db.commit().await?;
                    } else {
                        db.commit().await?;
                    }
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "commit exists in code: {:?}",
            violations
        );
    }

    #[test]
    fn test_db_op_passed_to_function_is_violation() {
        // Passing DbOp by value to another function is NOT allowed - must commit in same function
        let code = r#"
            impl Foo {
                async fn close(&self) -> Result<(), Error> {
                    let op = self.repo.begin_op().await?;
                    self.chart_of_accounts.post_closing_transaction(op, data).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Passing DbOp to another function should be a violation"
        );
    }

    #[test]
    fn test_db_op_returned_in_enum_variant() {
        // Pattern: return DbOp wrapped in an enum variant for caller to commit
        let code = r#"
            impl Foo {
                async fn complete_cycle(&self) -> Result<JobCompletion, Error> {
                    let mut op = self.repo.begin_op().await?;
                    self.do_work(&mut op).await?;
                    Ok(JobCompletion::CompleteWithOp(op))
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "DbOp is returned in enum variant: {:?}",
            violations
        );
    }

    #[test]
    fn test_db_op_with_ref_mut_still_needs_commit() {
        // Pattern: using &mut op doesn't transfer ownership, so commit is still needed
        let code = r#"
            impl Foo {
                async fn process(&self) -> Result<(), Error> {
                    let mut db = self.repo.begin_op().await?;
                    self.do_something(&mut db).await?;
                    self.do_more(&mut db).await?;
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Using &mut doesn't transfer ownership: {:?}",
            violations
        );
    }
}
