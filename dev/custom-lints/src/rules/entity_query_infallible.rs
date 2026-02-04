use std::collections::HashSet;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that enforces query methods on EsEntity structs do not return `Result`.
///
/// Any public function (`pub`, `pub(crate)`, `pub(super)`) on a struct with `#[derive(EsEntity)]`
/// that takes `&self` (not `&mut self`) must NOT return `Result`.
/// Query methods should return direct values or `Option<T>`.
pub struct EntityQueryInfallibleRule;

impl EntityQueryInfallibleRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EntityQueryInfallibleRule {
    fn default() -> Self {
        Self::new()
    }
}

/// First pass: collect all struct names that derive EsEntity
struct EsEntityCollector {
    es_entity_structs: HashSet<String>,
}

impl EsEntityCollector {
    fn new() -> Self {
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

/// Second pass: check impl blocks for EsEntity structs
struct ImplVisitor<'a> {
    violations: Vec<Violation>,
    path: &'a Path,
    es_entity_structs: &'a HashSet<String>,
    current_impl_struct: Option<String>,
}

impl<'a> ImplVisitor<'a> {
    fn new(path: &'a Path, es_entity_structs: &'a HashSet<String>) -> Self {
        Self {
            violations: Vec::new(),
            path,
            es_entity_structs,
            current_impl_struct: None,
        }
    }

    fn check_method(&mut self, node: &syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        let start_line = node.span().start().line;

        // Check if this is a public method
        if !is_public(&node.vis) {
            return;
        }

        // Check if it takes &self (immutable, not &mut self)
        let has_immutable_self = node.sig.inputs.iter().any(|arg| {
            if let syn::FnArg::Receiver(receiver) = arg {
                receiver.mutability.is_none() && receiver.reference.is_some()
            } else {
                false
            }
        });

        if !has_immutable_self {
            return;
        }

        // Check if return type contains Result
        if returns_result(&node.sig.output) {
            let struct_name = self.current_impl_struct.as_deref().unwrap_or("unknown");
            self.violations.push(
                Violation::new(
                    "entity-query-infallible",
                    self.path.display().to_string(),
                    format!(
                        "query method `{}` on EsEntity `{}` takes `&self` but returns `Result` — query methods should return direct values or `Option<T>`",
                        fn_name, struct_name
                    ),
                )
                .with_line(start_line),
            );
        }
    }
}

/// Check if visibility is public (pub, pub(crate), pub(super), etc.)
fn is_public(vis: &syn::Visibility) -> bool {
    !matches!(vis, syn::Visibility::Inherited)
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

impl<'a> Visit<'a> for ImplVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
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
            self.check_method(node);
        }

        syn::visit::visit_impl_item_fn(self, node);
    }
}

impl LintRule for EntityQueryInfallibleRule {
    fn name(&self) -> &'static str {
        "entity-query-infallible"
    }

    fn description(&self) -> &'static str {
        "Ensures EsEntity query methods (&self) do not return Result"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        let mut collector = EsEntityCollector::new();
        collector.visit_file(file);

        if collector.es_entity_structs.is_empty() {
            return vec![];
        }

        let mut visitor = ImplVisitor::new(path, &collector.es_entity_structs);
        visitor.visit_file(file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).expect("Failed to parse test code");
        let rule = EntityQueryInfallibleRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    #[test]
    fn test_valid_query_returns_value() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn name(&self) -> &str {
                    &self.name
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
    fn test_valid_query_returns_option() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn find_thing(&self) -> Option<ThingId> {
                    None
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
    fn test_invalid_query_returns_result() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn validate(&self) -> Result<(), MyError> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Result"));
        assert!(violations[0].message.contains("validate"));
    }

    #[test]
    fn test_private_method_ignored() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                fn internal_check(&self) -> Result<(), MyError> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Private methods should be ignored: {:?}",
            violations
        );
    }

    #[test]
    fn test_mut_self_method_ignored() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn mutate(&mut self) -> Result<Idempotent<()>, MyError> {
                    Ok(Idempotent::Executed(()))
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "&mut self methods should be ignored: {:?}",
            violations
        );
    }

    #[test]
    fn test_non_es_entity_ignored() {
        let code = r#"
            #[derive(Debug)]
            pub struct RegularStruct {
                data: String,
            }

            impl RegularStruct {
                pub fn check(&self) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Non-EsEntity structs should be ignored: {:?}",
            violations
        );
    }

    #[test]
    fn test_pub_crate_query_returning_result() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub(crate) fn check(&self) -> Result<Data, MyError> {
                    Ok(Data {})
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("check"));
    }

    #[test]
    fn test_pub_super_query_returning_result() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub(super) fn check(&self) -> Result<Data, MyError> {
                    Ok(Data {})
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("check"));
    }

    #[test]
    fn test_multiple_methods_mixed() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                // Good: query returns value
                pub fn name(&self) -> &str {
                    &self.name
                }

                // Good: query returns Option
                pub fn find(&self) -> Option<Data> {
                    None
                }

                // Bad: query returns Result
                pub fn validate(&self) -> Result<(), MyError> {
                    Ok(())
                }

                // Good: mutation (not checked by this rule)
                pub fn mutate(&mut self) -> Result<Idempotent<()>, MyError> {
                    Ok(Idempotent::Executed(()))
                }

                // Good: private method
                fn internal(&self) -> Result<(), MyError> {
                    Ok(())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(
            violations.len(),
            1,
            "Expected exactly 1 violation: {:?}",
            violations
        );
        assert!(violations[0].message.contains("validate"));
    }

    #[test]
    fn test_es_entity_with_other_derives() {
        let code = r#"
            #[derive(EsEntity, Builder, Debug)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn check(&self) -> Result<String, Error> {
                    Ok("bad".to_string())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Result"));
    }
}
