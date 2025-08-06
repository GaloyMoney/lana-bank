use anyhow::{Result, anyhow};
use cargo_metadata::{CargoOpt, MetadataCommand};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{File, ImplItemFn, ItemImpl, visit::Visit};

#[derive(Debug, Clone)]
pub struct AppCall {
    pub module: String,
    pub function: String,
}

pub async fn check_authorization() -> Result<()> {
    // Read the schema.rs file
    let schema_path = "../../lana/admin-server/src/graphql/schema.rs";
    let schema_content =
        fs::read_to_string(schema_path).map_err(|e| anyhow!("Failed to read schema.rs: {}", e))?;

    // Extract mutations and their app method calls
    let mutations = extract_mutations(&schema_content)?;

    let mut violations = Vec::new();
    let mut checked_functions = HashMap::new();

    for (mutation_name, app_calls) in mutations {
        for app_call in app_calls {
            let function_key = format!("{}::{}", app_call.module, app_call.function);

            // Skip if we've already checked this function
            if checked_functions.contains_key(&function_key) {
                continue;
            }

            // Find the corresponding implementation file
            let has_authz = check_function_authorization(&app_call)?;
            checked_functions.insert(function_key.clone(), has_authz);

            if !has_authz {
                violations.push(format!(
                    "Mutation '{}' calls '{}::{}' which does not perform authorization check",
                    mutation_name, app_call.module, app_call.function
                ));
            }
        }
    }

    if violations.is_empty() {
        println!("✅ All mutations have proper authorization!");
        Ok(())
    } else {
        println!("❌ Authorization violations found:");
        for violation in &violations {
            println!("  - {violation}");
        }
        std::process::exit(1);
    }
}

fn extract_mutations(content: &str) -> Result<Vec<(String, Vec<AppCall>)>> {
    let syntax_tree: File = syn::parse_str(content)?;
    let mut visitor = MutationVisitor::new();
    visitor.visit_file(&syntax_tree);
    Ok(visitor.mutations)
}

struct MutationVisitor {
    mutations: Vec<(String, Vec<AppCall>)>,
    current_function: Option<String>,
    in_mutation_impl: bool,
}

impl MutationVisitor {
    fn new() -> Self {
        Self {
            mutations: Vec::new(),
            current_function: None,
            in_mutation_impl: false,
        }
    }
}

impl<'ast> Visit<'ast> for MutationVisitor {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        // Check if this is an impl block for Mutation
        if let syn::Type::Path(type_path) = &*node.self_ty {
            if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Mutation"
            {
                self.in_mutation_impl = true;
                syn::visit::visit_item_impl(self, node);
                self.in_mutation_impl = false;
                return;
            }
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if self.in_mutation_impl {
            let has_ctx = node.sig.inputs.iter().any(|input| {
                if let syn::FnArg::Typed(typed) = input {
                    if let syn::Pat::Ident(ident) = &*typed.pat {
                        return ident.ident == "ctx";
                    }
                }
                false
            });

            // Check if this is an async function with ctx parameter
            if node.sig.asyncness.is_some() && has_ctx {
                self.current_function = Some(node.sig.ident.to_string());
                let mut app_calls = Vec::new();
                self.extract_app_calls(&node.block, &mut app_calls);

                if !app_calls.is_empty() {
                    self.mutations
                        .push((self.current_function.clone().unwrap(), app_calls));
                }
                self.current_function = None;
            }
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

impl MutationVisitor {
    fn extract_app_calls(&self, block: &syn::Block, app_calls: &mut Vec<AppCall>) {
        for stmt in &block.stmts {
            self.visit_stmt_for_app_calls(stmt, app_calls);
        }
    }

    fn visit_stmt_for_app_calls(&self, stmt: &syn::Stmt, app_calls: &mut Vec<AppCall>) {
        match stmt {
            syn::Stmt::Expr(expr, _) => {
                self.visit_expr_for_app_calls(expr, app_calls);
            }
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    self.visit_expr_for_app_calls(&init.expr, app_calls);
                }
            }
            syn::Stmt::Item(_) => {}
            syn::Stmt::Macro(macro_stmt) => {
                // Handle macro statements that might contain app calls
                if let Some(ident) = macro_stmt.mac.path.get_ident() {
                    if ident == "exec_mutation" {
                        let tokens_str = macro_stmt.mac.tokens.to_string();
                        self.parse_macro_tokens_for_app_calls(&tokens_str, app_calls);
                    }
                }
            }
        }
    }

    fn visit_expr_for_app_calls(&self, expr: &syn::Expr, app_calls: &mut Vec<AppCall>) {
        match expr {
            syn::Expr::MethodCall(method_call) => {
                self.check_method_call_for_app(method_call, app_calls);
                self.visit_expr_for_app_calls(&method_call.receiver, app_calls);
                for arg in &method_call.args {
                    self.visit_expr_for_app_calls(arg, app_calls);
                }
            }
            syn::Expr::Call(call) => {
                self.visit_expr_for_app_calls(&call.func, app_calls);
                for arg in &call.args {
                    self.visit_expr_for_app_calls(arg, app_calls);
                }
            }
            syn::Expr::Block(block) => {
                self.extract_app_calls(&block.block, app_calls);
            }
            syn::Expr::Match(match_expr) => {
                self.visit_expr_for_app_calls(&match_expr.expr, app_calls);
                for arm in &match_expr.arms {
                    self.visit_expr_for_app_calls(&arm.body, app_calls);
                }
            }
            syn::Expr::If(if_expr) => {
                self.visit_expr_for_app_calls(&if_expr.cond, app_calls);
                self.extract_app_calls(&if_expr.then_branch, app_calls);
                if let Some((_, else_branch)) = &if_expr.else_branch {
                    self.visit_expr_for_app_calls(else_branch, app_calls);
                }
            }
            syn::Expr::Macro(macro_expr) => {
                // Handle exec_mutation! macro which contains app calls
                if let Some(ident) = macro_expr.mac.path.get_ident() {
                    if ident == "exec_mutation" {
                        // Parse the macro tokens to extract app calls
                        let tokens_str = macro_expr.mac.tokens.to_string();
                        self.parse_macro_tokens_for_app_calls(&tokens_str, app_calls);
                    }
                }
            }
            _ => {
                // Handle other expression types if needed
            }
        }
    }

    fn check_method_call_for_app(
        &self,
        method_call: &syn::ExprMethodCall,
        app_calls: &mut Vec<AppCall>,
    ) {
        // Check for patterns like app.module().function() or app.module().submodule().function()
        if let Some((module, function)) = self.extract_app_call_from_method_chain(method_call) {
            app_calls.push(AppCall { module, function });
        }
    }

    fn extract_app_call_from_method_chain(
        &self,
        method_call: &syn::ExprMethodCall,
    ) -> Option<(String, String)> {
        let function = method_call.method.to_string();

        // Walk back through the method chain to find app.module()
        let mut current_expr = &*method_call.receiver;
        let mut chain = Vec::new();

        loop {
            match current_expr {
                syn::Expr::MethodCall(inner_call) => {
                    chain.push(inner_call.method.to_string());
                    current_expr = &*inner_call.receiver;
                }
                syn::Expr::Path(path) => {
                    if path.path.segments.len() == 1 && path.path.segments[0].ident == "app" {
                        // We found the app root, now extract the correct module
                        // For multi-level calls like app.credit().terms_templates().create_terms_template()
                        // chain will be ["terms_templates", "credit"] (reverse order)
                        // We want the second-to-last item as the module (terms_templates)
                        // or if there's only one item, use that (for simple app.module().function() calls)

                        if chain.len() >= 2 {
                            // Multi-level call: use second-to-last as module
                            // e.g., app.credit().terms_templates().create_terms_template()
                            // chain = ["terms_templates", "credit"], so chain[chain.len()-2] = "terms_templates"
                            let module = &chain[chain.len() - 2];
                            return Some((module.clone(), function));
                        } else if chain.len() == 1 {
                            // Simple call: app.module().function()
                            let module = &chain[0];
                            return Some((module.clone(), function));
                        }
                    }
                    break;
                }
                _ => break,
            }
        }

        None
    }

    fn parse_macro_tokens_for_app_calls(&self, tokens: &str, app_calls: &mut Vec<AppCall>) {
        let clean_tokens = tokens
            .replace(' ', "")
            .replace('\n', "")
            .replace('\t', "")
            .replace('\r', "");

        if let Some(start) = clean_tokens.find("app.") {
            let remaining = &clean_tokens[start..];
            if let Some(app_call) = self.parse_simple_app_call(remaining) {
                app_calls.push(app_call);
            }
        }
    }

    fn parse_simple_app_call(&self, text: &str) -> Option<AppCall> {
        if !text.starts_with("app.") {
            return None;
        }

        let after_app = &text[4..]; // skip "app."
        let parts: Vec<&str> = after_app.split('.').collect();

        if parts.len() < 2 {
            return None;
        }

        // The last part should contain the function name followed by (
        let last_part = parts.last()?;
        if let Some(paren_pos) = last_part.find('(') {
            let function_name = &last_part[..paren_pos];

            // Clean function name - only alphanumeric and underscore
            let clean_function = function_name
                .chars()
                .take_while(|&c| c.is_alphanumeric() || c == '_')
                .collect::<String>();

            if clean_function.is_empty() {
                return None;
            }

            // The module is the second-to-last part (remove () if present)
            if parts.len() >= 2 {
                let module_part = parts[parts.len() - 2];
                let module_name = module_part.trim_end_matches("()");

                if !module_name.is_empty() {
                    return Some(AppCall {
                        module: module_name.to_string(),
                        function: clean_function,
                    });
                }
            }
        }

        None
    }
}

fn check_function_authorization(app_call: &AppCall) -> Result<bool> {
    if let Some(target_file) = resolve_function_with_metadata(app_call)? {
        let content = fs::read_to_string(&target_file)
            .map_err(|e| anyhow!("Failed to read resolved file {}: {}", target_file, e))?;
        return check_specific_function_in_content(&content, &app_call.function);
    }

    Ok(false)
}

fn resolve_function_with_metadata(app_call: &AppCall) -> Result<Option<String>> {
    let metadata = MetadataCommand::new()
        .manifest_path("../../Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .exec()?;

    let target_package = find_package_for_module(&metadata, &app_call.module)?;

    if let Some(package) = target_package {
        if let Some(file_path) =
            find_function_in_package(package, &app_call.module, &app_call.function)?
        {
            return Ok(Some(file_path));
        }
    }

    Ok(None)
}

fn find_package_for_module<'a>(
    metadata: &'a cargo_metadata::Metadata,
    module_name: &str,
) -> Result<Option<&'a cargo_metadata::Package>> {
    // Look through workspace packages for one that might contain our module
    for package in metadata.workspace_packages() {
        // Check if package name matches module patterns
        if is_package_match(&package.name, module_name) {
            return Ok(Some(package));
        }

        // Also check if this is a multi-module package (like core-credit containing terms_templates)
        if package.name.starts_with("core-") || package.name.starts_with("lana-") {
            // Check if the package's source directory contains the module
            let src_dir = package.manifest_path.parent().unwrap().join("src");
            if module_exists_in_package_src(src_dir.as_std_path(), module_name)? {
                return Ok(Some(package));
            }
        }
    }

    Ok(None)
}

fn is_package_match(package_name: &str, module_name: &str) -> bool {
    // Direct matches
    if package_name == module_name {
        return true;
    }

    // Handle common patterns:
    // - "core-customer" matches "customers"
    // - "lana-access" matches "access"
    // - "terms_templates" matches "core-credit" (special case)

    let package_parts: Vec<&str> = package_name.split('-').collect();
    let module_parts: Vec<&str> = module_name.split('_').collect();

    // Check if any part of the package name matches any part of the module name
    for package_part in &package_parts {
        for module_part in &module_parts {
            if package_part == module_part
                || package_part.trim_end_matches('s') == module_part.trim_end_matches('s')
                || package_part == &format!("{module_part}s")
                || &format!("{package_part}s") == module_part
            {
                return true;
            }
        }
    }

    false
}

fn module_exists_in_package_src(src_dir: &std::path::Path, module_name: &str) -> Result<bool> {
    if !src_dir.exists() {
        return Ok(false);
    }

    // Check for direct module directory
    let module_dir = src_dir.join(module_name);
    if module_dir.exists() {
        return Ok(true);
    }

    // Check for singular/plural variations
    let module_singular = module_name.trim_end_matches('s');
    let module_dir_singular = src_dir.join(module_singular);
    if module_dir_singular.exists() {
        return Ok(true);
    }

    // For compound module names, check if any part exists
    if module_name.contains('_') {
        for part in module_name.split('_') {
            let part_dir = src_dir.join(part);
            if part_dir.exists() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn find_function_in_package(
    package: &cargo_metadata::Package,
    module_name: &str,
    function_name: &str,
) -> Result<Option<String>> {
    let src_dir = package.manifest_path.parent().unwrap().join("src");

    // Search strategy:
    // 1. Look in lib.rs first (main entry point)
    // 2. Look in module-specific directories
    // 3. Recursively search all .rs files

    let search_files = collect_rust_files_for_module(src_dir.as_std_path(), module_name)?;

    for file_path in search_files {
        if find_function_in_file(&file_path, function_name)? {
            return Ok(Some(file_path.to_string_lossy().to_string()));
        }
    }

    Ok(None)
}

fn collect_rust_files_for_module(
    src_dir: &std::path::Path,
    module_name: &str,
) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    // Always check lib.rs first
    let lib_file = src_dir.join("lib.rs");
    if lib_file.exists() {
        files.push(lib_file);
    }

    // Check main.rs
    let main_file = src_dir.join("main.rs");
    if main_file.exists() {
        files.push(main_file);
    }

    // Check module-specific directories
    let possible_module_dirs = [
        module_name,
        module_name.trim_end_matches('s'),
        &format!("{}s", module_name.trim_end_matches('s')),
    ];

    for dir_name in possible_module_dirs {
        let module_dir = src_dir.join(dir_name);
        if module_dir.exists() {
            files.extend(find_rust_files(&module_dir)?);
        }
    }

    // If we still haven't found anything, search all .rs files
    if files.len() <= 2 {
        // Only lib.rs and/or main.rs
        files.extend(find_rust_files(src_dir)?);
    }

    Ok(files)
}

fn find_function_in_file(file_path: &std::path::Path, function_name: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)?;
    let syntax_tree: syn::File = syn::parse_str(&content)
        .map_err(|e| anyhow!("Failed to parse {}: {}", file_path.display(), e))?;

    let mut visitor = FunctionFinder::new(function_name);
    visitor.visit_file(&syntax_tree);

    Ok(visitor.function_location.is_some())
}

struct FunctionFinder {
    target_function: String,
    function_location: Option<()>,
}

impl FunctionFinder {
    fn new(function_name: &str) -> Self {
        Self {
            target_function: function_name.to_string(),
            function_location: None,
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for FunctionFinder {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if node.sig.ident == self.target_function {
            // Found the function!
            self.function_location = Some(());
            return;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.ident == self.target_function {
            self.function_location = Some(());
            return;
        }
        syn::visit::visit_item_fn(self, node);
    }
}

fn check_specific_function_in_content(content: &str, function_name: &str) -> Result<bool> {
    // Parse the file and find the specific function
    let syntax_tree: syn::File =
        syn::parse_str(content).map_err(|e| anyhow!("Failed to parse Rust file: {}", e))?;

    let mut visitor = AuthzVisitor::new(function_name);
    visitor.visit_file(&syntax_tree);

    Ok(visitor.has_authorization)
}

// Note: find_rust_files is used by collect_rust_files_for_module
fn find_rust_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut rust_files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively search subdirectories
                rust_files.extend(find_rust_files(&path)?);
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    rust_files.push(path);
                }
            }
        }
    }

    Ok(rust_files)
}

struct AuthzVisitor {
    target_function: String,
    has_authorization: bool,
    in_target_function: bool,
}

impl AuthzVisitor {
    fn new(function_name: &str) -> Self {
        Self {
            target_function: function_name.to_string(),
            has_authorization: false,
            in_target_function: false,
        }
    }
}

impl<'ast> Visit<'ast> for AuthzVisitor {
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if node.sig.ident == self.target_function
            && node.sig.asyncness.is_some()
            && matches!(node.vis, syn::Visibility::Public(_))
        {
            // Check if this function has 'sub' as the first parameter
            if !self.function_has_sub_parameter(&node.sig) {
                // This function doesn't have 'sub' parameter - it should be flagged
                // We don't set has_authorization = true because this is a violation
                self.in_target_function = false;
                return;
            }

            self.in_target_function = true;
            self.visit_block(&node.block);
            self.in_target_function = false;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.ident == self.target_function
            && node.sig.asyncness.is_some()
            && matches!(node.vis, syn::Visibility::Public(_))
        {
            // Check if this function has 'sub' as the first parameter
            if !self.function_has_sub_parameter(&node.sig) {
                // This function doesn't have 'sub' parameter - it should be flagged
                self.in_target_function = false;
                return;
            }

            self.in_target_function = true;
            self.visit_block(&node.block);
            self.in_target_function = false;
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_target_function && !self.has_authorization {
            let method_name = node.method.to_string();

            // Check for direct authz calls
            if method_name == "enforce_permission" {
                self.has_authorization = true;
                return;
            }

            // Note: subject_can_* methods are for permission checking, not authorization enforcement
            if method_name.starts_with("subject_can_") {
                syn::visit::visit_expr_method_call(self, node);
                return;
            }

            // Check if this is a function call with 'sub' as first argument
            // All functions that take 'sub' should either:
            // 1. Have authorization enforcement, or
            // 2. Be delegation calls to other functions that enforce authorization
            if self.has_sub_as_first_argument(node) {
                self.has_authorization = true;
                return;
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Note: subject_can_* functions are for permission checking, not authorization enforcement
        syn::visit::visit_expr_call(self, node);
    }
}

impl AuthzVisitor {
    fn has_sub_as_first_argument(&self, method_call: &syn::ExprMethodCall) -> bool {
        // Check if 'sub' is passed as first argument
        // Any function that takes 'sub' as first parameter should have authorization
        if let Some(syn::Expr::Path(path)) = method_call.args.first() {
            if path.path.segments.len() == 1 && path.path.segments[0].ident == "sub" {
                return true;
            }
        }
        false
    }

    fn function_has_sub_parameter(&self, sig: &syn::Signature) -> bool {
        // Check if the function signature has 'sub' as a parameter
        // Look for patterns like:
        // - sub: &SomeType
        // - sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject
        for input in &sig.inputs {
            if let syn::FnArg::Typed(typed) = input {
                if let syn::Pat::Ident(ident) = &*typed.pat {
                    if ident.ident == "sub" {
                        return true;
                    }
                }
            }
        }
        false
    }
}
