use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that enforces constructor naming conventions:
/// - `new` — sync, infallible (must NOT be async, must NOT return `Result`)
/// - `try_new` — sync, fallible (must NOT be async, MUST return `Result`)
/// - `init` — async, fallible (MUST be async, MUST return `Result`)
pub struct ConstructorNamingRule;

impl ConstructorNamingRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConstructorNamingRule {
    fn default() -> Self {
        Self::new()
    }
}

struct ImplVisitor<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
    in_trait_impl: bool,
}

impl<'a> ImplVisitor<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            violations: Vec::new(),
            path,
            in_trait_impl: false,
        }
    }

    fn check_method(&mut self, node: &syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        let is_async = node.sig.asyncness.is_some();
        let has_result = returns_result(&node.sig.output);

        match fn_name.as_str() {
            "new" => {
                if is_async {
                    self.violations.push(
                        Violation::new(
                            "constructor-naming",
                            self.path.display().to_string(),
                            "`new` must not be async (use `init` for async fallible constructors)",
                        )
                        .with_line(start_line),
                    );
                }
                if has_result {
                    self.violations.push(
                        Violation::new(
                            "constructor-naming",
                            self.path.display().to_string(),
                            "`new` must not return `Result` (use `try_new` for fallible constructors)",
                        )
                        .with_line(start_line),
                    );
                }
            }
            "try_new" => {
                if is_async {
                    self.violations.push(
                        Violation::new(
                            "constructor-naming",
                            self.path.display().to_string(),
                            "`try_new` must not be async (use `init` for async fallible constructors)",
                        )
                        .with_line(start_line),
                    );
                }
                if !has_result {
                    self.violations.push(
                        Violation::new(
                            "constructor-naming",
                            self.path.display().to_string(),
                            "`try_new` must return `Result` (use `new` for infallible constructors)",
                        )
                        .with_line(start_line),
                    );
                }
            }
            "init" => {
                if !is_async {
                    self.violations.push(
                        Violation::new(
                            "constructor-naming",
                            self.path.display().to_string(),
                            "`init` must be async (use `try_new` for sync fallible constructors)",
                        )
                        .with_line(start_line),
                    );
                }
                if !has_result {
                    self.violations.push(
                        Violation::new(
                            "constructor-naming",
                            self.path.display().to_string(),
                            "`init` must return `Result` (use `new` for sync infallible constructors)",
                        )
                        .with_line(start_line),
                    );
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for ImplVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        let prev = self.in_trait_impl;
        self.in_trait_impl = node.trait_.is_some();
        syn::visit::visit_item_impl(self, node);
        self.in_trait_impl = prev;
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        if !self.in_trait_impl {
            self.check_method(node);
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Check if the return type contains `Result`
fn returns_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => type_contains_result(ty),
    }
}

/// Recursively check if a type is or contains `Result`
fn type_contains_result(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            let path = &type_path.path;

            if let Some(segment) = path.segments.last() {
                let type_name = segment.ident.to_string();

                if type_name == "Result" {
                    return true;
                }
            }

            false
        }
        syn::Type::Reference(type_ref) => type_contains_result(&type_ref.elem),
        syn::Type::Paren(paren) => type_contains_result(&paren.elem),
        _ => false,
    }
}

impl LintRule for ConstructorNamingRule {
    fn name(&self) -> &'static str {
        "constructor-naming"
    }

    fn description(&self) -> &'static str {
        "Enforces constructor naming conventions: new (sync, infallible), try_new (sync, fallible), init (async, fallible)"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        let mut visitor = ImplVisitor::new(path);
        visitor.visit_file(file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse test code");
        let rule = ConstructorNamingRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    // === Valid cases ===

    #[test]
    fn test_valid_new_sync_infallible() {
        let code = r#"
            impl Foo {
                pub fn new() -> Self {
                    Self {}
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
    fn test_valid_try_new_sync_fallible() {
        let code = r#"
            impl Foo {
                pub fn try_new(x: u32) -> Result<Self, Error> {
                    Ok(Self { x })
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
    fn test_valid_init_async_fallible() {
        let code = r#"
            impl Foo {
                pub async fn init(pool: &PgPool) -> Result<Self, Error> {
                    Ok(Self {})
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

    // === Violation cases ===

    #[test]
    fn test_new_returning_result() {
        let code = r#"
            impl Foo {
                pub fn new(x: u32) -> Result<Self, Error> {
                    Ok(Self { x })
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
                .contains("`new` must not return `Result`")
        );
    }

    #[test]
    fn test_new_async() {
        let code = r#"
            impl Foo {
                pub async fn new() -> Self {
                    Self {}
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
        assert!(violations[0].message.contains("`new` must not be async"));
    }

    #[test]
    fn test_new_async_and_result() {
        let code = r#"
            impl Foo {
                pub async fn new() -> Result<Self, Error> {
                    Ok(Self {})
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
    fn test_try_new_no_result() {
        let code = r#"
            impl Foo {
                pub fn try_new() -> Self {
                    Self {}
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
                .contains("`try_new` must return `Result`")
        );
    }

    #[test]
    fn test_try_new_async() {
        let code = r#"
            impl Foo {
                pub async fn try_new() -> Result<Self, Error> {
                    Ok(Self {})
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
                .contains("`try_new` must not be async")
        );
    }

    #[test]
    fn test_init_not_async() {
        let code = r#"
            impl Foo {
                pub fn init() -> Result<Self, Error> {
                    Ok(Self {})
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
        assert!(violations[0].message.contains("`init` must be async"));
    }

    #[test]
    fn test_init_no_result() {
        let code = r#"
            impl Foo {
                pub async fn init() -> Self {
                    Self {}
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
                .contains("`init` must return `Result`")
        );
    }

    #[test]
    fn test_init_sync_no_result() {
        let code = r#"
            impl Foo {
                pub fn init() -> Self {
                    Self {}
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

    // === Private methods are still checked ===

    #[test]
    fn test_private_new_returning_result() {
        let code = r#"
            impl Foo {
                fn new(x: u32) -> Result<Self, Error> {
                    Ok(Self { x })
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Private methods should also be checked: {:?}",
            violations
        );
    }

    #[test]
    fn test_pub_crate_checked() {
        let code = r#"
            impl Foo {
                pub(crate) fn new(x: u32) -> Result<Self, Error> {
                    Ok(Self { x })
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "pub(crate) methods should be checked: {:?}",
            violations
        );
    }

    // === Non-constructor names ignored ===

    #[test]
    fn test_other_method_names_ignored() {
        let code = r#"
            impl Foo {
                pub async fn create() -> Self {
                    Self {}
                }

                pub fn build() -> Result<Self, Error> {
                    Ok(Self {})
                }

                pub fn setup() -> Self {
                    Self {}
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Non-constructor names should be ignored: {:?}",
            violations
        );
    }

    // === Trait impls are ignored ===

    #[test]
    fn test_trait_impl_ignored() {
        let code = r#"
            impl JobInitializer for FooInit {
                type Config = FooConfig;
                fn init(
                    &self,
                    job: &Job,
                    _: JobSpawner<Self::Config>,
                ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
                    Ok(Box::new(FooRunner {}))
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Trait impl methods should be ignored: {:?}",
            violations
        );
    }

    #[test]
    fn test_trait_impl_new_ignored() {
        let code = r#"
            impl Default for Foo {
                fn new() -> Result<Self, Error> {
                    Ok(Self {})
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Trait impl methods should be ignored: {:?}",
            violations
        );
    }

    #[test]
    fn test_inherent_impl_still_checked_alongside_trait_impl() {
        let code = r#"
            impl Foo {
                pub fn new() -> Result<Self, Error> {
                    Ok(Self {})
                }
            }

            impl JobInitializer for FooInit {
                fn init(&self) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
                    Ok(Box::new(FooRunner {}))
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Only inherent impl should be checked: {:?}",
            violations
        );
        assert!(violations[0].message.contains("`new` must not return `Result`"));
    }

    #[test]
    fn test_multiple_impls_mixed() {
        let code = r#"
            impl Foo {
                pub fn new() -> Self {
                    Self {}
                }
            }

            impl Bar {
                pub fn new() -> Result<Self, Error> {
                    Ok(Self {})
                }
            }

            impl Baz {
                pub async fn init(pool: &Pool) -> Result<Self, Error> {
                    Ok(Self {})
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Only Bar::new should violate: {:?}",
            violations
        );
        assert!(
            violations[0]
                .message
                .contains("`new` must not return `Result`")
        );
    }
}
