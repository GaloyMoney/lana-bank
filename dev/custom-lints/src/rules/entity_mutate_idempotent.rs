use std::collections::HashSet;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{LintRule, Violation};

/// Rule that enforces mutating methods on EsEntity structs return `Idempotent` type.
///
/// Any public function (`pub`, `pub(crate)`, `pub(super)`) on a struct with `#[derive(EsEntity)]`
/// that takes `&mut self` must return either:
/// - `Idempotent<T>`
/// - `Result<Idempotent<T>, E>`
pub struct EntityMutateIdempotentRule;

impl EntityMutateIdempotentRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EntityMutateIdempotentRule {
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
        // Check if this struct has #[derive(EsEntity)] or #[derive(EsEntity, ...)]
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
    /// Current impl block's struct name (if it's an EsEntity)
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

        // Check if it takes &mut self
        let has_mut_self = node.sig.inputs.iter().any(|arg| {
            if let syn::FnArg::Receiver(receiver) = arg {
                receiver.mutability.is_some() && receiver.reference.is_some()
            } else {
                false
            }
        });

        if !has_mut_self {
            return;
        }

        // Check return type
        let return_type = &node.sig.output;
        if !returns_idempotent(return_type) {
            let struct_name = self.current_impl_struct.as_deref().unwrap_or("unknown");
            self.violations.push(
                Violation::new(
                    "entity-mutate-idempotent",
                    self.path.display().to_string(),
                    format!(
                        "method `{}` on EsEntity `{}` takes `&mut self` but doesn't return `Idempotent<T>` or `Result<Idempotent<T>, E>`",
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

/// Check if the return type is `Idempotent<T>` or `Result<Idempotent<T>, E>`
fn returns_idempotent(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => type_contains_idempotent(ty),
    }
}

/// Recursively check if a type is or contains `Idempotent`
fn type_contains_idempotent(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            let path = &type_path.path;

            // Get the last segment (the actual type name)
            if let Some(segment) = path.segments.last() {
                let type_name = segment.ident.to_string();

                // Direct Idempotent<T>
                if type_name == "Idempotent" {
                    return true;
                }

                // Result<Idempotent<T>, E>
                if type_name == "Result"
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                {
                    return type_contains_idempotent(inner_ty);
                }
            }

            false
        }
        // Handle references, boxes, etc.
        syn::Type::Reference(type_ref) => type_contains_idempotent(&type_ref.elem),
        syn::Type::Paren(paren) => type_contains_idempotent(&paren.elem),
        _ => false,
    }
}

impl<'a> Visit<'a> for ImplVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        // Check if this is an impl block for an EsEntity struct
        if let syn::Type::Path(type_path) = &*node.self_ty
            && let Some(segment) = type_path.path.segments.last()
        {
            let struct_name = segment.ident.to_string();
            if self.es_entity_structs.contains(&struct_name) {
                // This is an impl block for an EsEntity
                self.current_impl_struct = Some(struct_name);
                syn::visit::visit_item_impl(self, node);
                self.current_impl_struct = None;
                return;
            }
        }

        // Not an EsEntity impl, but continue visiting nested items
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        // Only check methods if we're inside an EsEntity impl block
        if self.current_impl_struct.is_some() {
            self.check_method(node);
        }

        syn::visit::visit_impl_item_fn(self, node);
    }
}

impl LintRule for EntityMutateIdempotentRule {
    fn name(&self) -> &'static str {
        "entity-mutate-idempotent"
    }

    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation> {
        // First pass: collect all EsEntity struct names
        let mut collector = EsEntityCollector::new();
        collector.visit_file(file);

        // If no EsEntity structs in this file, skip second pass
        if collector.es_entity_structs.is_empty() {
            return vec![];
        }

        // Second pass: check impl blocks
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
        let rule = EntityMutateIdempotentRule::new();
        rule.check_file(&file, Path::new("test.rs"))
    }

    #[test]
    fn test_valid_idempotent_return() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn mutate(&mut self, data: Data) -> Idempotent<()> {
                    Idempotent::Executed(())
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
    fn test_valid_result_idempotent_return() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn mutate(&mut self, data: Data) -> Result<Idempotent<SomeData>, MyError> {
                    Ok(Idempotent::Executed(SomeData {}))
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
    fn test_invalid_non_idempotent_return() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn mutate(&mut self, data: Data) -> Result<SomeData, MyError> {
                    Ok(SomeData {})
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Idempotent"));
        assert!(violations[0].message.contains("mutate"));
    }

    #[test]
    fn test_invalid_unit_return() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn mutate(&mut self) {
                    self.do_something();
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Idempotent"));
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
                fn private_mutate(&mut self) -> () {
                    self.do_something();
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
    fn test_immutable_method_ignored() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn query(&self) -> SomeData {
                    SomeData {}
                }
            }
        "#;
        let violations = check_code(code);
        assert!(
            violations.is_empty(),
            "Immutable methods should be ignored: {:?}",
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
                pub fn mutate(&mut self) -> () {
                    self.data = "changed".to_string();
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
    fn test_es_entity_with_other_derives() {
        let code = r#"
            #[derive(EsEntity, Builder, Debug)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub fn mutate(&mut self) -> Result<String, Error> {
                    Ok("bad".to_string())
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Idempotent"));
    }

    #[test]
    fn test_pub_crate_visibility() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub(crate) fn mutate(&mut self) -> String {
                    "bad".to_string()
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Idempotent"));
    }

    #[test]
    fn test_pub_super_visibility() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct MyEntity {
                id: EntityId,
                events: EntityEvents<MyEvent>,
            }

            impl MyEntity {
                pub(super) fn mutate(&mut self) -> String {
                    "bad".to_string()
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Idempotent"));
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
                // Good: returns Idempotent
                pub fn good_mutate(&mut self) -> Idempotent<()> {
                    Idempotent::Executed(())
                }

                // Bad: doesn't return Idempotent
                pub fn bad_mutate(&mut self) -> Result<(), Error> {
                    Ok(())
                }

                // Good: immutable, no Idempotent required
                pub fn query(&self) -> Data {
                    Data {}
                }

                // Good: private, no Idempotent required
                fn private_helper(&mut self) {
                    // internal logic
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
        assert!(violations[0].message.contains("bad_mutate"));
    }

    #[test]
    fn test_multiple_entities() {
        let code = r#"
            #[derive(EsEntity)]
            pub struct Entity1 {
                id: EntityId,
                events: EntityEvents<Event1>,
            }

            #[derive(Debug)]
            pub struct NotAnEntity {
                data: String,
            }

            #[derive(EsEntity)]
            pub struct Entity2 {
                id: EntityId,
                events: EntityEvents<Event2>,
            }

            impl Entity1 {
                pub fn bad_mutate(&mut self) -> () {}
            }

            impl NotAnEntity {
                pub fn ignored_mutate(&mut self) -> () {}
            }

            impl Entity2 {
                pub fn also_bad(&mut self) -> String {
                    "bad".to_string()
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
}
