use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that enforces conventions for functions receiving database operation arguments.
///
/// Functions that receive a `DbOp`, `DbOpWithTime`, or `impl AtomicOperation` parameter must:
/// 1. Have the `_in_op` suffix in their function name
/// 2. Have the db operation as the first parameter (after `self` for methods)
pub struct DbOpConventionRule;

impl DbOpConventionRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DbOpConventionRule {
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

    fn check_function(
        &mut self,
        fn_name: &str,
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        start_line: usize,
        is_method: bool,
    ) {
        // Skip functions named begin_op* - these return DbOp, not receive it
        if fn_name.starts_with("begin_op") {
            return;
        }

        // Find the db op parameter and its position
        let mut db_op_info: Option<(usize, &str)> = None;
        let mut param_index = 0;

        for (idx, arg) in inputs.iter().enumerate() {
            match arg {
                syn::FnArg::Receiver(_) => {
                    // Skip self parameter, don't increment param_index
                    continue;
                }
                syn::FnArg::Typed(pat_type) => {
                    if is_db_op_type(&pat_type.ty) {
                        let param_name = extract_param_name(&pat_type.pat);
                        db_op_info = Some((param_index, param_name));
                        break;
                    }
                    param_index += 1;
                }
            }
            // Only track position for non-self params
            let _ = idx;
        }

        // No db op parameter found, nothing to check
        let Some((position, _param_name)) = db_op_info else {
            return;
        };

        // Check 1: Function name must end with _in_op
        if !fn_name.ends_with("_in_op") {
            self.violations.push(
                Violation::new(
                    "db-op-convention",
                    self.path.display().to_string(),
                    format!(
                        "function `{}` receives a db operation parameter but doesn't have `_in_op` suffix",
                        fn_name
                    ),
                )
                .with_line(start_line),
            );
        }

        // Check 2: Db op must be the first parameter (after self)
        if position != 0 {
            let position_desc = if is_method {
                format!("position {} (after self)", position + 1)
            } else {
                format!("position {}", position + 1)
            };
            self.violations.push(
                Violation::new(
                    "db-op-convention",
                    self.path.display().to_string(),
                    format!(
                        "function `{}` has db operation parameter at {} but it should be the first parameter{}",
                        fn_name,
                        position_desc,
                        if is_method { " after self" } else { "" }
                    ),
                )
                .with_line(start_line),
            );
        }
    }
}

/// Check if a type represents a database operation
fn is_db_op_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(type_ref) => is_db_op_type(&type_ref.elem),
        syn::Type::Path(type_path) => {
            let path_str = path_to_string(&type_path.path);
            // Match DbOp, DbOpWithTime, es_entity::DbOp, etc.
            path_str.contains("DbOp") || path_str.contains("DbOpWithTime")
        }
        syn::Type::ImplTrait(impl_trait) => {
            // Check for `impl AtomicOperation` or `impl es_entity::AtomicOperation`
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

/// Convert a syn::Path to a string for matching
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Extract the parameter name from a pattern
fn extract_param_name(pat: &syn::Pat) -> &'static str {
    match pat {
        syn::Pat::Ident(pat_ident) => {
            let name = pat_ident.ident.to_string();
            match name.as_str() {
                "op" => "op",
                "db" => "db",
                "db_op" => "db_op",
                _ => "unknown",
            }
        }
        _ => "unknown",
    }
}

impl<'a> Visit<'a> for FunctionVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        let fn_name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        self.check_function(&fn_name, &node.sig.inputs, start_line, false);

        // Continue visiting nested items
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        let start_line = node.span().start().line;

        // Check if this is a method (has self parameter)
        let is_method = node
            .sig
            .inputs
            .iter()
            .any(|arg| matches!(arg, syn::FnArg::Receiver(_)));

        self.check_function(&fn_name, &node.sig.inputs, start_line, is_method);

        // Continue visiting nested items
        syn::visit::visit_impl_item_fn(self, node);
    }
}

impl LintRule for DbOpConventionRule {
    fn name(&self) -> &'static str {
        "db-op-convention"
    }

    fn description(&self) -> &'static str {
        "Enforces _in_op suffix for functions taking DbOp parameters"
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
        let rule = DbOpConventionRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    #[test]
    fn test_valid_in_op_function() {
        let code = r#"
            impl Foo {
                async fn create_in_op(
                    &self,
                    db: &mut es_entity::DbOp<'_>,
                    data: SomeData,
                ) -> Result<(), Error> {
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
    fn test_missing_in_op_suffix() {
        let code = r#"
            impl Foo {
                async fn create(
                    &self,
                    db: &mut es_entity::DbOp<'_>,
                    data: SomeData,
                ) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("_in_op"));
    }

    #[test]
    fn test_db_op_not_first_param() {
        let code = r#"
            impl Foo {
                async fn create_in_op(
                    &self,
                    data: SomeData,
                    db: &mut es_entity::DbOp<'_>,
                ) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("first parameter"));
    }

    #[test]
    fn test_both_violations() {
        let code = r#"
            impl Foo {
                async fn process(
                    &self,
                    data: SomeData,
                    op: &mut DbOp<'_>,
                ) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            2,
            "Expected 2 violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_impl_atomic_operation() {
        let code = r#"
            impl Foo {
                async fn create_in_op(
                    &self,
                    op: &mut impl es_entity::AtomicOperation,
                    data: SomeData,
                ) -> Result<(), Error> {
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
    fn test_impl_atomic_operation_wrong_position() {
        let code = r#"
            impl Foo {
                async fn create_in_op(
                    &self,
                    data: SomeData,
                    op: &mut impl AtomicOperation,
                ) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("first parameter"));
    }

    #[test]
    fn test_begin_op_excluded() {
        let code = r#"
            impl Foo {
                async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, Error> {
                    self.pool.begin().await
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "begin_op should be excluded: {:?}",
            violations
        );
    }

    #[test]
    fn test_no_db_op_no_violation() {
        let code = r#"
            impl Foo {
                async fn process(&self, data: SomeData) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_free_function_with_db_op() {
        let code = r#"
            async fn process_in_op(
                op: &mut DbOp<'_>,
                data: SomeData,
            ) -> Result<(), Error> {
                Ok(())
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
    fn test_free_function_missing_suffix() {
        let code = r#"
            async fn process(
                op: &mut DbOp<'_>,
                data: SomeData,
            ) -> Result<(), Error> {
                Ok(())
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("_in_op"));
    }

    #[test]
    fn test_db_op_with_time() {
        let code = r#"
            impl Foo {
                async fn create_in_op(
                    &self,
                    db: &mut es_entity::DbOpWithTime<'_>,
                    data: SomeData,
                ) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Expected no violations for DbOpWithTime: {:?}",
            violations
        );
    }
}
