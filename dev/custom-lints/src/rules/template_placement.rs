use std::path::Path;

use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that enforces CALA template definitions (NewTxTemplate) live in `templates/` directories.
pub struct TemplatePlacementRule;

impl TemplatePlacementRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TemplatePlacementRule {
    fn default() -> Self {
        Self::new()
    }
}

struct TemplatePlacementVisitor<'a> {
    path: &'a Path,
    violations: Vec<Violation>,
    in_test_context: bool,
    file_in_templates_dir: bool,
}

impl<'a> TemplatePlacementVisitor<'a> {
    fn new(path: &'a Path) -> Self {
        let path_str = path.to_string_lossy();

        let is_test_file = path_str.ends_with("_test.rs")
            || path_str.contains("/tests/")
            || path_str.starts_with("tests/")
            || path_str == "tests.rs";

        let file_in_templates_dir = path_str.contains("/templates/");

        Self {
            path,
            violations: Vec::new(),
            in_test_context: is_test_file,
            file_in_templates_dir,
        }
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

    fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            let path = attr.path();
            if path.is_ident("test") {
                return true;
            }
            if path.is_ident("cfg")
                && let Ok(nested) = attr.parse_args::<syn::Ident>()
                && nested == "test"
            {
                return true;
            }
            if let Some(segment) = path.segments.last()
                && segment.ident == "test"
            {
                return true;
            }
            false
        })
    }

    fn check_path_for_new_tx_template(&mut self, path: &syn::Path, span_line: usize) {
        if self.in_test_context || self.file_in_templates_dir {
            return;
        }

        let has_new_tx_template = path.segments.iter().any(|seg| seg.ident == "NewTxTemplate");

        if has_new_tx_template {
            self.violations.push(
                Violation::new(
                    "template-placement",
                    self.path.display().to_string(),
                    "CALA template definitions (NewTxTemplate) must be in a templates/ directory",
                )
                .with_line(span_line),
            );
        }
    }
}

impl<'ast> Visit<'ast> for TemplatePlacementVisitor<'ast> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let was_in_test = self.in_test_context;
        if Self::has_cfg_test_attribute(&node.attrs) {
            self.in_test_context = true;
        }

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

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let line = node
            .segments
            .first()
            .map(|s| s.ident.span().start().line)
            .unwrap_or(0);
        self.check_path_for_new_tx_template(node, line);
        syn::visit::visit_path(self, node);
    }
}

impl LintRule for TemplatePlacementRule {
    fn name(&self) -> &'static str {
        "template-placement"
    }

    fn description(&self) -> &'static str {
        "Enforces CALA template definitions (NewTxTemplate) are in templates/ directories"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        let mut visitor = TemplatePlacementVisitor::new(path);
        visitor.visit_file(file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        check_code_with_path(code, "core/deposit/src/ledger/mod.rs")
    }

    fn check_code_with_path(code: &str, path: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse test code");
        let rule = TemplatePlacementRule::new();
        rule.check_file(&file, Path::new(path))
    }

    #[test]
    fn detects_new_tx_template_outside_templates_dir() {
        let code = r#"
            fn create_template() {
                let template = NewTxTemplate::builder()
                    .id(TxTemplateId::new())
                    .build()
                    .unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("templates/ directory"));
    }

    #[test]
    fn allows_new_tx_template_in_templates_dir() {
        let code = r#"
            fn create_template() {
                let template = NewTxTemplate::builder()
                    .id(TxTemplateId::new())
                    .build()
                    .unwrap();
            }
        "#;
        let violations = check_code_with_path(code, "core/deposit/src/ledger/templates/deposit.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_new_tx_template_in_test_module() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                fn create_template() {
                    let template = NewTxTemplate::builder()
                        .id(TxTemplateId::new())
                        .build()
                        .unwrap();
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_new_tx_template_in_test_file() {
        let code = r#"
            fn create_template() {
                let template = NewTxTemplate::builder()
                    .id(TxTemplateId::new())
                    .build()
                    .unwrap();
            }
        "#;
        let violations = check_code_with_path(code, "core/deposit/src/ledger/template_test.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_fully_qualified_new_tx_template() {
        let code = r#"
            fn create_template() {
                let template = cala_ledger::tx_template::NewTxTemplate::builder()
                    .build()
                    .unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn no_violation_for_code_without_new_tx_template() {
        let code = r#"
            fn foo() {
                let x = 1 + 2;
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_new_tx_template_in_test_function() {
        let code = r#"
            #[test]
            fn test_template() {
                let template = NewTxTemplate::builder()
                    .build()
                    .unwrap();
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
