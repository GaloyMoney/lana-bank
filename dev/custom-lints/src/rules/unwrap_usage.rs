use std::path::Path;

use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that prohibits `.unwrap()` in production code.
/// Test code is exempt (test modules, test functions, test files).
pub struct UnwrapUsageRule;

impl UnwrapUsageRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnwrapUsageRule {
    fn default() -> Self {
        Self::new()
    }
}

struct UnwrapVisitor<'a> {
    path: &'a Path,
    violations: Vec<Violation>,
    in_test_context: bool,
}

impl<'a> UnwrapVisitor<'a> {
    fn new(path: &'a Path) -> Self {
        // Check if the file itself is a test file
        let path_str = path.to_string_lossy();
        let is_test_file = path_str.ends_with("_test.rs")
            || path_str.contains("/tests/")
            || path_str.starts_with("tests/")
            || path_str == "tests.rs";

        Self {
            path,
            violations: Vec::new(),
            in_test_context: is_test_file,
        }
    }

    fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            let path = attr.path();
            // Check for #[test], #[tokio::test], #[test_case], etc.
            if path.is_ident("test") {
                return true;
            }
            // Check for #[cfg(test)]
            if path.is_ident("cfg")
                && let Ok(nested) = attr.parse_args::<syn::Ident>()
                && nested == "test"
            {
                return true;
            }
            // Check for tokio::test, rstest, etc.
            if let Some(segment) = path.segments.last()
                && segment.ident == "test"
            {
                return true;
            }
            false
        })
    }

    fn has_cfg_test_attribute(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if attr.path().is_ident("cfg")
                && let Ok(nested) = attr.parse_args::<syn::Ident>()
            {
                return nested == "test";
            }
            false
        })
    }

    fn check_expr_for_unwrap(&mut self, expr: &syn::Expr) {
        if self.in_test_context {
            return;
        }

        if let syn::Expr::MethodCall(method_call) = expr
            && method_call.method == "unwrap"
            && method_call.args.is_empty()
        {
            let line = method_call.method.span().start().line;
            self.violations.push(
                Violation::new(
                    "unwrap-usage",
                    self.path.display().to_string(),
                    "Use `expect(\"reason\")` instead of `unwrap()` to provide context on failure",
                )
                .with_line(line),
            );
        }
    }
}

impl<'ast> Visit<'ast> for UnwrapVisitor<'ast> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        // Check if this module has #[cfg(test)]
        let was_in_test = self.in_test_context;
        if Self::has_cfg_test_attribute(&node.attrs) {
            self.in_test_context = true;
        }

        // Also check for modules named "test" or "tests"
        let mod_name = node.ident.to_string();
        if mod_name == "test" || mod_name == "tests" {
            self.in_test_context = true;
        }

        syn::visit::visit_item_mod(self, node);
        self.in_test_context = was_in_test;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let was_in_test = self.in_test_context;
        if Self::has_test_attribute(&node.attrs) {
            self.in_test_context = true;
        }

        syn::visit::visit_item_fn(self, node);
        self.in_test_context = was_in_test;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let was_in_test = self.in_test_context;
        if Self::has_test_attribute(&node.attrs) {
            self.in_test_context = true;
        }

        syn::visit::visit_impl_item_fn(self, node);
        self.in_test_context = was_in_test;
    }

    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        self.check_expr_for_unwrap(node);
        syn::visit::visit_expr(self, node);
    }
}

impl LintRule for UnwrapUsageRule {
    fn name(&self) -> &'static str {
        "unwrap-usage"
    }

    fn description(&self) -> &'static str {
        "Prohibits unwrap() in production code; use expect() instead"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        let mut visitor = UnwrapVisitor::new(path);
        visitor.visit_file(file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        check_code_with_path(code, "test.rs")
    }

    fn check_code_with_path(code: &str, path: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse test code");
        let rule = UnwrapUsageRule::new();
        rule.check_file(&file, Path::new(path))
    }

    #[test]
    fn detects_unwrap_in_production_code() {
        let code = r#"
            fn foo() {
                let x = Some(1).unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expect"));
    }

    #[test]
    fn allows_expect_in_production_code() {
        let code = r#"
            fn foo() {
                let x = Some(1).expect("should have value");
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_unwrap_in_test_function() {
        let code = r#"
            #[test]
            fn test_foo() {
                let x = Some(1).unwrap();
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_unwrap_in_tokio_test() {
        let code = r#"
            #[tokio::test]
            async fn test_foo() {
                let x = Some(1).unwrap();
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_unwrap_in_cfg_test_module() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                fn helper() {
                    let x = Some(1).unwrap();
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_unwrap_in_test_file() {
        let code = r#"
            fn foo() {
                let x = Some(1).unwrap();
            }
        "#;
        let violations = check_code_with_path(code, "src/foo_test.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_unwrap_in_tests_directory() {
        let code = r#"
            fn foo() {
                let x = Some(1).unwrap();
            }
        "#;
        let violations = check_code_with_path(code, "tests/integration.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_multiple_unwraps() {
        let code = r#"
            fn foo() {
                let x = Some(1).unwrap();
                let y = Ok::<_, ()>(2).unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn detects_unwrap_in_impl_method() {
        let code = r#"
            struct Foo;
            impl Foo {
                fn bar(&self) {
                    let x = Some(1).unwrap();
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn allows_unwrap_in_impl_test_method() {
        let code = r#"
            struct Foo;
            impl Foo {
                #[test]
                fn test_bar(&self) {
                    let x = Some(1).unwrap();
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_chained_unwrap() {
        let code = r#"
            fn foo() {
                let x = Some(Some(1)).unwrap().unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn allows_unwrap_or() {
        let code = r#"
            fn foo() {
                let x = Some(1).unwrap_or(0);
                let y = Some(1).unwrap_or_default();
                let z = Some(1).unwrap_or_else(|| 0);
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
