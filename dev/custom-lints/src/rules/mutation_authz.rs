use std::path::{Path, PathBuf};

use syn::visit::Visit;
use walkdir::WalkDir;

use crate::{Violation, WorkspaceRule};

const RULE_NAME: &str = "mutation-authorization";
const SCHEMA_PATH: &str = "lana/admin-server/src/graphql/schema.rs";

/// Rule that ensures all GraphQL mutations enforce authorization.
///
/// Shallow checks (all mutations):
/// 1. Extract the subject via `app_and_sub_from_ctx!(ctx)`
/// 2. Bind it as `sub` (not `_sub`)
/// 3. Pass `sub` to at least one domain method call or macro invocation
///
/// Deep check (mutations using `exec_mutation!`):
/// 4. The called domain method must call `enforce_permission`
///
/// Methods gated behind `#[cfg(...)]` are skipped (e.g., test-only mutations).
pub struct MutationAuthzRule;

impl MutationAuthzRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MutationAuthzRule {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRule for MutationAuthzRule {
    fn name(&self) -> &'static str {
        RULE_NAME
    }

    fn description(&self) -> &'static str {
        "All GraphQL mutations must enforce authorization by passing `sub` to domain methods"
    }

    fn check_workspace(&self, workspace_root: &Path) -> anyhow::Result<Vec<Violation>> {
        let schema_path = workspace_root.join(SCHEMA_PATH);
        let content = std::fs::read_to_string(&schema_path)?;
        let file = syn::parse_file(&content)?;

        let source_files = collect_source_files(workspace_root);

        let mut checker = MutationChecker {
            violations: Vec::new(),
            workspace_root: workspace_root.to_path_buf(),
            source_files,
        };
        checker.visit_file(&file);

        Ok(checker.violations)
    }
}

/// A pre-collected Rust source file (path + contents).
struct SourceFile {
    path: PathBuf,
    content: String,
}

/// Collect all `.rs` files from `core/`, `lana/app/`, and `lana/contract-creation/` for method lookup.
fn collect_source_files(workspace_root: &Path) -> Vec<SourceFile> {
    let dirs = ["core", "lana/app", "lana/contract-creation"];
    let mut files = Vec::new();

    for dir in &dirs {
        let dir_path = workspace_root.join(dir);
        for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "rs")
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    files.push(SourceFile {
                        path: entry.path().to_path_buf(),
                        content,
                    });
                }
            }
        }
    }

    files
}

struct MutationChecker {
    violations: Vec<Violation>,
    workspace_root: PathBuf,
    source_files: Vec<SourceFile>,
}

impl<'ast> Visit<'ast> for MutationChecker {
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if !is_mutation_impl(node) {
            return;
        }

        for item in &node.items {
            if let syn::ImplItem::Fn(method) = item {
                self.check_mutation_method(method);
            }
        }
    }
}

impl MutationChecker {
    fn check_mutation_method(&mut self, method: &syn::ImplItemFn) {
        let method_name = method.sig.ident.to_string();
        let line = method.sig.ident.span().start().line;

        // Skip non-async methods
        if method.sig.asyncness.is_none() {
            return;
        }

        // Skip cfg-gated methods (e.g., test-only mutations)
        if has_cfg_attribute(&method.attrs) {
            return;
        }

        match find_sub_extraction(&method.block) {
            SubExtraction::None => {
                self.violations.push(
                    Violation::new(
                        RULE_NAME,
                        SCHEMA_PATH,
                        format!(
                            "Mutation '{method_name}' does not extract subject via app_and_sub_from_ctx!"
                        ),
                    )
                    .with_line(line),
                );
                return;
            }
            SubExtraction::Ignored => {
                self.violations.push(
                    Violation::new(
                        RULE_NAME,
                        SCHEMA_PATH,
                        format!(
                            "Mutation '{method_name}' ignores subject (_sub) — authorization not enforced"
                        ),
                    )
                    .with_line(line),
                );
                return;
            }
            SubExtraction::Extracted => {
                if !body_passes_sub(&method.block) {
                    self.violations.push(
                        Violation::new(
                            RULE_NAME,
                            SCHEMA_PATH,
                            format!(
                                "Mutation '{method_name}' extracts subject but never passes it to domain methods"
                            ),
                        )
                        .with_line(line),
                    );
                    return;
                }
            }
        }

        // Check that exec_mutation! macro is used
        if find_exec_mutation_macro(&method.block).is_none() {
            self.violations.push(
                Violation::new(
                    RULE_NAME,
                    SCHEMA_PATH,
                    format!("Mutation '{method_name}' does not use exec_mutation! macro"),
                )
                .with_line(line),
            );
            return;
        }

        // Deep check: verify the domain method calls enforce_permission
        self.check_domain_method_authz(method, &method_name, line);
    }

    fn check_domain_method_authz(
        &mut self,
        method: &syn::ImplItemFn,
        mutation_name: &str,
        line: usize,
    ) {
        let Some(mac) = find_exec_mutation_macro(&method.block) else {
            return; // Not using exec_mutation!, skip deep check for now
        };

        let Some(call_info) = extract_call_from_exec_mutation(mac) else {
            return; // Can't parse the call expression
        };

        let search_dirs = search_dirs_for_accessor(call_info.accessors.first());

        let matches: Vec<&SourceFile> = self
            .source_files
            .iter()
            .filter(|sf| {
                let relative = sf
                    .path
                    .strip_prefix(&self.workspace_root)
                    .unwrap_or(&sf.path);
                let in_search_dir = search_dirs.iter().any(|dir| relative.starts_with(dir));
                in_search_dir
                    && sf
                        .content
                        .contains(&format!("fn {}(", call_info.method_name))
            })
            .collect();

        if matches.is_empty() {
            return; // Method not found, skip
        }

        let any_has_enforce = matches
            .iter()
            .any(|sf| method_calls_enforce_permission(&sf.content, &call_info.method_name));

        if any_has_enforce {
            return;
        }

        // Cross-struct follow: check if target method delegates to self.field().method(sub, ...)
        // and that delegated method has the authz check
        for sf in &matches {
            let delegated = extract_delegated_calls(&sf.content, &call_info.method_name);
            for delegated_method in &delegated {
                let found = self.source_files.iter().any(|other_sf| {
                    other_sf
                        .content
                        .contains(&format!("fn {delegated_method}("))
                        && method_calls_enforce_permission(&other_sf.content, delegated_method)
                });
                if found {
                    return;
                }
            }
        }

        let locations: Vec<String> = matches
            .iter()
            .map(|sf| {
                sf.path
                    .strip_prefix(&self.workspace_root)
                    .unwrap_or(&sf.path)
                    .display()
                    .to_string()
            })
            .collect();

        self.violations.push(
            Violation::new(
                RULE_NAME,
                SCHEMA_PATH,
                format!(
                    "Mutation '{mutation_name}' calls '{method}' ({locations}) which does not call enforce_permission",
                    method = call_info.method_name,
                    locations = locations.join(", "),
                ),
            )
            .with_line(line),
        );
    }
}

// ---------------------------------------------------------------------------
// Shallow check helpers
// ---------------------------------------------------------------------------

fn is_mutation_impl(node: &syn::ItemImpl) -> bool {
    if let syn::Type::Path(type_path) = &*node.self_ty {
        type_path
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "Mutation")
    } else {
        false
    }
}

fn has_cfg_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("cfg"))
}

enum SubExtraction {
    /// No `app_and_sub_from_ctx!` call found
    None,
    /// Uses `_sub` — explicitly ignoring the subject
    Ignored,
    /// Uses `sub` — subject is available for authorization
    Extracted,
}

fn find_sub_extraction(block: &syn::Block) -> SubExtraction {
    for stmt in &block.stmts {
        if let syn::Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                if is_app_and_sub_macro(&init.expr) {
                    return check_sub_binding(&local.pat);
                }
            }
        }
    }
    SubExtraction::None
}

fn is_app_and_sub_macro(expr: &syn::Expr) -> bool {
    if let syn::Expr::Macro(expr_macro) = expr {
        expr_macro
            .mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "app_and_sub_from_ctx")
    } else {
        false
    }
}

fn check_sub_binding(pat: &syn::Pat) -> SubExtraction {
    if let syn::Pat::Tuple(tuple) = pat {
        for elem in &tuple.elems {
            if let syn::Pat::Ident(pat_ident) = elem {
                let name = pat_ident.ident.to_string();
                if name == "_sub" {
                    return SubExtraction::Ignored;
                }
                if name == "sub" {
                    return SubExtraction::Extracted;
                }
            }
        }
    }
    // If the pattern doesn't match the expected shape, assume extracted
    SubExtraction::Extracted
}

/// Check whether `sub` is passed as an argument to any method call or macro.
fn body_passes_sub(block: &syn::Block) -> bool {
    let mut checker = SubUsageChecker { found: false };
    checker.visit_block(block);
    checker.found
}

struct SubUsageChecker {
    found: bool,
}

impl<'ast> Visit<'ast> for SubUsageChecker {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if !self.found {
            for arg in &node.args {
                if expr_is_sub(arg) {
                    self.found = true;
                    return;
                }
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if !self.found {
            for arg in &node.args {
                if expr_is_sub(arg) {
                    self.found = true;
                    return;
                }
            }
            syn::visit::visit_expr_call(self, node);
        }
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if !self.found && tokens_contain_sub_ident(&mac.tokens) {
            self.found = true;
        }
    }
}

fn expr_is_sub(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Path(path) => path.path.is_ident("sub"),
        syn::Expr::Reference(ref_expr) => expr_is_sub(&ref_expr.expr),
        _ => false,
    }
}

fn tokens_contain_sub_ident(tokens: &proc_macro2::TokenStream) -> bool {
    for token in tokens.clone() {
        match token {
            proc_macro2::TokenTree::Ident(ref ident) if *ident == "sub" => return true,
            proc_macro2::TokenTree::Group(ref group) => {
                if tokens_contain_sub_ident(&group.stream()) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Deep check helpers — exec_mutation! → domain method → enforce_permission
// ---------------------------------------------------------------------------

struct MethodCallInfo {
    /// Accessor chain after `app.`, e.g., `["deposits"]` or `["credit", "collaterals"]`
    accessors: Vec<String>,
    /// The final method name, e.g., `"record_deposit"`
    method_name: String,
}

/// Find the `exec_mutation!` macro invocation in a method body.
fn find_exec_mutation_macro(block: &syn::Block) -> Option<&syn::Macro> {
    struct Finder<'a> {
        found: Option<&'a syn::Macro>,
    }

    impl<'a> Visit<'a> for Finder<'a> {
        fn visit_macro(&mut self, mac: &'a syn::Macro) {
            if self.found.is_none()
                && mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "exec_mutation")
            {
                self.found = Some(mac);
            }
        }
    }

    let mut finder = Finder { found: None };
    finder.visit_block(block);
    finder.found
}

/// Extract the method call info from the last argument of `exec_mutation!`.
fn extract_call_from_exec_mutation(mac: &syn::Macro) -> Option<MethodCallInfo> {
    let args = split_macro_args(&mac.tokens);
    let load_tokens = args.last()?;
    let expr: syn::Expr = syn::parse2(load_tokens.clone()).ok()?;
    extract_method_call_chain(&expr)
}

/// Split a macro token stream by top-level commas.
///
/// Commas inside groups (parentheses, brackets, braces) are not split points
/// because `proc_macro2` already wraps them in `Group` tokens.
fn split_macro_args(tokens: &proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    let mut args = Vec::new();
    let mut current = Vec::new();

    for token in tokens.clone() {
        if let proc_macro2::TokenTree::Punct(ref punct) = token {
            if punct.as_char() == ',' {
                let ts: proc_macro2::TokenStream = current.drain(..).collect();
                if !ts.is_empty() {
                    args.push(ts);
                }
                continue;
            }
        }
        current.push(token);
    }
    if !current.is_empty() {
        let ts: proc_macro2::TokenStream = current.into_iter().collect();
        args.push(ts);
    }

    args
}

/// Walk a method-call chain like `app.credit().collaterals().record_update(sub, ...)`
/// and extract the accessor chain and the final method name.
fn extract_method_call_chain(expr: &syn::Expr) -> Option<MethodCallInfo> {
    if let syn::Expr::MethodCall(call) = expr {
        let method_name = call.method.to_string();
        let mut accessors = Vec::new();

        let mut current: &syn::Expr = &call.receiver;
        loop {
            match current {
                syn::Expr::MethodCall(inner_call) => {
                    accessors.push(inner_call.method.to_string());
                    current = &inner_call.receiver;
                }
                syn::Expr::Path(_) => break, // Reached `app`
                _ => break,
            }
        }

        accessors.reverse();
        Some(MethodCallInfo {
            accessors,
            method_name,
        })
    } else {
        None
    }
}

/// Map the first accessor name to search directories.
fn search_dirs_for_accessor(accessor: Option<&String>) -> Vec<&'static str> {
    match accessor.map(|s| s.as_str()) {
        Some("deposits") => vec!["core/deposit"],
        Some("credit") => vec!["core/credit"],
        Some("accounting") => vec!["core/accounting"],
        Some("customers" | "customer_kyc") => vec!["core/customer"],
        Some("custody") => vec!["core/custody"],
        Some("access") => vec!["core/access"],
        Some("governance") => vec!["core/governance"],
        Some("reports") => vec!["core/report"],
        Some("exposed_domain_configs") => vec!["core/domain-config"],
        Some("terms_templates") => vec!["core/credit"],
        Some("contract_creation") => vec!["lana/contract-creation"],
        None => vec!["lana/app"], // Direct method on LanaApp
        Some(_) => vec!["core", "lana/app", "lana/contract-creation"], // Unknown, search broadly
    }
}

/// Parse a source file, find the named method, and check for authorization.
///
/// Checks for `enforce_permission` or `evaluate_permission` both directly and
/// through one level of `self.method()` delegation within the same file.
fn method_calls_enforce_permission(content: &str, method_name: &str) -> bool {
    let file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(_) => return true, // Can't parse → don't report
    };

    for item in &file.items {
        if let syn::Item::Impl(impl_block) = item {
            for impl_item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == method_name {
                        return check_method_authz(method, &file);
                    }
                }
            }
        }
    }

    true // Method not found in any impl block → don't report
}

/// Check if a method enforces authorization, either directly or via one level
/// of `self.helper()` delegation within the same file.
fn check_method_authz(method: &syn::ImplItemFn, file: &syn::File) -> bool {
    // Direct check
    if has_authz_call(&method.block) {
        return true;
    }

    // One-level follow: check self.method() calls within the same file
    let self_calls = find_self_method_calls(&method.block);
    for called_name in &self_calls {
        for item in &file.items {
            if let syn::Item::Impl(impl_block) = item {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Fn(other_method) = impl_item {
                        if other_method.sig.ident == *called_name
                            && has_authz_call(&other_method.block)
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Check for `enforce_permission` or `evaluate_permission` calls in a block.
fn has_authz_call(block: &syn::Block) -> bool {
    struct Checker {
        found: bool,
    }

    impl<'ast> Visit<'ast> for Checker {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            if node.method == "enforce_permission" || node.method == "evaluate_permission" {
                self.found = true;
            }
            if !self.found {
                syn::visit::visit_expr_method_call(self, node);
            }
        }
    }

    let mut checker = Checker { found: false };
    checker.visit_block(block);
    checker.found
}

/// Find all `self.method_name(...)` calls (where receiver is exactly `self`).
fn find_self_method_calls(block: &syn::Block) -> Vec<String> {
    struct Finder {
        calls: Vec<String>,
    }

    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            if let syn::Expr::Path(path) = &*node.receiver {
                if path.path.is_ident("self") {
                    self.calls.push(node.method.to_string());
                }
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let mut finder = Finder { calls: Vec::new() };
    finder.visit_block(block);
    finder.calls
}

/// Parse a source file, find the named method, and extract all method calls
/// where `sub` is passed as an argument (cross-struct delegation).
fn extract_delegated_calls(content: &str, method_name: &str) -> Vec<String> {
    let file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    for item in &file.items {
        if let syn::Item::Impl(impl_block) = item {
            for impl_item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == method_name {
                        return find_method_calls_with_sub(&method.block);
                    }
                }
            }
        }
    }

    Vec::new()
}

/// Find all method calls in a block where `sub` is passed as an argument.
fn find_method_calls_with_sub(block: &syn::Block) -> Vec<String> {
    struct Finder {
        calls: Vec<String>,
    }

    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            let has_sub = node.args.iter().any(|arg| expr_is_sub(arg));
            if has_sub {
                self.calls.push(node.method.to_string());
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let mut finder = Finder { calls: Vec::new() };
    finder.visit_block(block);
    finder.calls
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_mutations(code: &str) -> Vec<Violation> {
        let file = syn::parse_file(code).unwrap();
        let mut checker = MutationChecker {
            violations: Vec::new(),
            workspace_root: PathBuf::from("/nonexistent"),
            source_files: Vec::new(),
        };
        checker.visit_file(&file);
        checker.violations
    }

    // -- Shallow check tests --

    #[test]
    fn valid_mutation_with_exec_macro_4arg() {
        let code = r#"
            impl Mutation {
                async fn user_create(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, sub) = app_and_sub_from_ctx!(ctx);
                    exec_mutation!(
                        Payload,
                        User,
                        ctx,
                        app.access().create_user(sub, input.email)
                    )
                }
            }
        "#;
        assert!(check_mutations(code).is_empty());
    }

    #[test]
    fn valid_mutation_with_exec_macro_2arg() {
        let code = r#"
            impl Mutation {
                async fn link_generate(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, sub) = app_and_sub_from_ctx!(ctx);
                    exec_mutation!(
                        Payload,
                        app.module().generate_link(sub, input.id)
                    )
                }
            }
        "#;
        assert!(check_mutations(code).is_empty());
    }

    #[test]
    fn violation_missing_exec_mutation_macro() {
        let code = r#"
            impl Mutation {
                async fn something(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, sub) = app_and_sub_from_ctx!(ctx);
                    let result = app.module().method(sub, id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        let violations = check_mutations(code);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("does not use exec_mutation!")
        );
    }

    #[test]
    fn violation_ignored_sub() {
        let code = r#"
            impl Mutation {
                async fn something(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, _sub) = app_and_sub_from_ctx!(ctx);
                    let result = app.module().method(id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        let violations = check_mutations(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ignores subject"));
    }

    #[test]
    fn violation_missing_sub_extraction() {
        let code = r#"
            impl Mutation {
                async fn something(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let app = ctx.data_unchecked::<App>();
                    let result = app.module().method(id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        let violations = check_mutations(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("does not extract subject"));
    }

    #[test]
    fn violation_sub_extracted_but_not_passed() {
        let code = r#"
            impl Mutation {
                async fn something(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, sub) = app_and_sub_from_ctx!(ctx);
                    let result = app.module().method(id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        let violations = check_mutations(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("never passes it"));
    }

    #[test]
    fn cfg_gated_mutation_is_skipped() {
        let code = r#"
            impl Mutation {
                #[cfg(feature = "testing")]
                async fn test_only(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, _sub) = app_and_sub_from_ctx!(ctx);
                    let result = app.module().method(id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        assert!(check_mutations(code).is_empty());
    }

    #[test]
    fn non_async_methods_are_skipped() {
        let code = r#"
            impl Mutation {
                fn helper(&self) -> i32 {
                    42
                }
            }
        "#;
        assert!(check_mutations(code).is_empty());
    }

    #[test]
    fn non_mutation_impl_is_skipped() {
        let code = r#"
            impl Query {
                async fn something(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let app = ctx.data_unchecked::<App>();
                    let result = app.module().method(id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        assert!(check_mutations(code).is_empty());
    }

    #[test]
    fn multiple_mutations_checked_independently() {
        let code = r#"
            impl Mutation {
                async fn good(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, sub) = app_and_sub_from_ctx!(ctx);
                    exec_mutation!(
                        Payload,
                        User,
                        ctx,
                        app.module().method(sub, id)
                    )
                }
                async fn bad(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, _sub) = app_and_sub_from_ctx!(ctx);
                    let result = app.module().method(id).await;
                    Ok(Payload::from(result))
                }
            }
        "#;
        let violations = check_mutations(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("bad"));
    }

    #[test]
    fn sub_passed_by_reference_with_exec_mutation() {
        let code = r#"
            impl Mutation {
                async fn something(&self, ctx: &Context<'_>) -> Result<Payload> {
                    let (app, sub) = app_and_sub_from_ctx!(ctx);
                    exec_mutation!(
                        Payload,
                        User,
                        ctx,
                        app.module().method(&sub, id)
                    )
                }
            }
        "#;
        assert!(check_mutations(code).is_empty());
    }

    // -- Call extraction tests --

    #[test]
    fn extract_simple_method_call() {
        let expr: syn::Expr = syn::parse_str("app.deposits().record_deposit(sub, id)").unwrap();
        let info = extract_method_call_chain(&expr).unwrap();
        assert_eq!(info.method_name, "record_deposit");
        assert_eq!(info.accessors, vec!["deposits"]);
    }

    #[test]
    fn extract_chained_method_call() {
        let expr: syn::Expr =
            syn::parse_str("app.credit().collaterals().record_update(sub, id)").unwrap();
        let info = extract_method_call_chain(&expr).unwrap();
        assert_eq!(info.method_name, "record_update");
        assert_eq!(info.accessors, vec!["credit", "collaterals"]);
    }

    #[test]
    fn extract_direct_app_method() {
        let expr: syn::Expr =
            syn::parse_str("app.create_facility_proposal(sub, id, amount)").unwrap();
        let info = extract_method_call_chain(&expr).unwrap();
        assert_eq!(info.method_name, "create_facility_proposal");
        assert!(info.accessors.is_empty());
    }

    #[test]
    fn split_macro_args_four_args() {
        let tokens: proc_macro2::TokenStream =
            "Payload , Type , ctx , app.method(sub)".parse().unwrap();
        let args = split_macro_args(&tokens);
        assert_eq!(args.len(), 4);
    }

    #[test]
    fn split_macro_args_five_args() {
        let tokens: proc_macro2::TokenStream = "Payload , Type , IdType , ctx , app.method(sub)"
            .parse()
            .unwrap();
        let args = split_macro_args(&tokens);
        assert_eq!(args.len(), 5);
    }

    // -- enforce_permission detection tests --

    #[test]
    fn detect_enforce_permission_present() {
        let code = r#"{
            self.authz
                .enforce_permission(sub, CoreObject::all(), CoreAction::CREATE)
                .await?;
            let result = self.repo.find_by_id(id).await?;
            Ok(result)
        }"#;
        let block: syn::Block = syn::parse_str(code).unwrap();
        assert!(has_authz_call(&block));
    }

    #[test]
    fn detect_evaluate_permission_present() {
        let code = r#"{
            self.authz
                .evaluate_permission(sub, CoreObject::all(), CoreAction::CREATE, enforce)
                .await?;
            Ok(result)
        }"#;
        let block: syn::Block = syn::parse_str(code).unwrap();
        assert!(has_authz_call(&block));
    }

    #[test]
    fn detect_authz_call_missing() {
        let code = r#"{
            let result = self.repo.find_by_id(id).await?;
            Ok(result)
        }"#;
        let block: syn::Block = syn::parse_str(code).unwrap();
        assert!(!has_authz_call(&block));
    }

    #[test]
    fn find_self_calls() {
        let code = r#"{
            self.subject_can_create(sub, true).await?;
            self.repo.find_by_id(id).await?;
            Ok(result)
        }"#;
        let block: syn::Block = syn::parse_str(code).unwrap();
        let calls = find_self_method_calls(&block);
        assert!(calls.contains(&"subject_can_create".to_string()));
        // self.repo.find_by_id — receiver is self.repo (field access), not self
        assert!(!calls.contains(&"find_by_id".to_string()));
    }

    #[test]
    fn one_level_follow_delegation() {
        let code = r#"
            impl MyStruct {
                pub async fn public_method(&self, sub: &Subject) -> Result<(), Error> {
                    self.check_permission(sub).await?;
                    Ok(())
                }

                async fn check_permission(&self, sub: &Subject) -> Result<(), Error> {
                    self.authz.enforce_permission(sub, Obj::all(), Act::DO).await?;
                    Ok(())
                }
            }
        "#;
        let file = syn::parse_file(code).unwrap();
        // Find public_method
        if let syn::Item::Impl(impl_block) = &file.items[0] {
            if let syn::ImplItem::Fn(method) = &impl_block.items[0] {
                assert_eq!(method.sig.ident, "public_method");
                assert!(check_method_authz(method, &file));
            }
        }
    }

    #[test]
    fn one_level_follow_no_authz() {
        let code = r#"
            impl MyStruct {
                pub async fn public_method(&self, sub: &Subject) -> Result<(), Error> {
                    self.do_work(sub).await?;
                    Ok(())
                }

                async fn do_work(&self, sub: &Subject) -> Result<(), Error> {
                    self.repo.save(sub).await?;
                    Ok(())
                }
            }
        "#;
        let file = syn::parse_file(code).unwrap();
        if let syn::Item::Impl(impl_block) = &file.items[0] {
            if let syn::ImplItem::Fn(method) = &impl_block.items[0] {
                assert_eq!(method.sig.ident, "public_method");
                assert!(!check_method_authz(method, &file));
            }
        }
    }

    // -- Accessor mapping tests --

    #[test]
    fn accessor_mapping() {
        assert_eq!(
            search_dirs_for_accessor(Some(&"deposits".to_string())),
            vec!["core/deposit"]
        );
        assert_eq!(
            search_dirs_for_accessor(Some(&"credit".to_string())),
            vec!["core/credit"]
        );
        assert_eq!(
            search_dirs_for_accessor(Some(&"accounting".to_string())),
            vec!["core/accounting"]
        );
        assert_eq!(search_dirs_for_accessor(None), vec!["lana/app"]);
    }
}
