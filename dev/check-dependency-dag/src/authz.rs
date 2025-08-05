use anyhow::{Result, anyhow};
use cargo_metadata::{CargoOpt, MetadataCommand};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
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
            println!("  - {}", violation);
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
                        // We found the app root, now extract module
                        if let Some(module) = chain.last() {
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
        // Remove spaces to handle tokenization issues
        let clean_tokens = tokens.replace(" ", "");

        // Simple parsing of exec_mutation! macro content
        // Look for app.module().function() patterns in the tokens
        if let Some(start) = clean_tokens.find("app.") {
            let remaining = &clean_tokens[start..];
            if let Some(app_call) = self.parse_simple_app_call(remaining) {
                app_calls.push(app_call);
            }
        }
    }

    fn parse_simple_app_call(&self, text: &str) -> Option<AppCall> {
        // More careful parsing - look for the pattern app.module().function(
        if let Some(app_start) = text.find("app.") {
            let remaining = &text[app_start..];

            // Find the module name (between first dot and next ())
            if let Some(first_dot) = remaining.find('.') {
                let after_app = &remaining[first_dot + 1..];
                if let Some(paren_pos) = after_app.find("()") {
                    let module = &after_app[..paren_pos];

                    // Now find the function after the next dot
                    let after_module = &after_app[paren_pos + 2..];
                    if let Some(dot_pos) = after_module.find('.') {
                        let after_dot = &after_module[dot_pos + 1..];
                        if let Some(func_paren) = after_dot.find('(') {
                            let function = &after_dot[..func_paren];
                            return Some(AppCall {
                                module: module.to_string(),
                                function: function.to_string(),
                            });
                        }
                    }
                }
            }
        }

        None
    }
}

fn check_function_authorization(app_call: &AppCall) -> Result<bool> {
    // Use Rust's own resolution by asking rust-analyzer or using metadata
    if let Some(target_file) = resolve_function_location(app_call)? {
        let content = fs::read_to_string(&target_file)
            .map_err(|e| anyhow!("Failed to read resolved file {}: {}", target_file, e))?;
        return check_function_in_content(&content, &app_call.function);
    }

    // Fallback to directory scanning if resolution fails
    let possible_core_modules = [
        app_call.module.as_str(), // exact match (e.g., "access" -> "access")
        &app_call.module.trim_end_matches('s'), // remove trailing 's' (e.g., "customers" -> "customer")
        &format!("{}s", app_call.module), // add trailing 's' (e.g., "applicant" -> "applicants")
    ];

    for core_module in possible_core_modules {
        let core_dir = format!("../../core/{}", core_module);

        // Check if the core directory exists
        if !std::path::Path::new(&core_dir).exists() {
            continue;
        }

        // Auto-discover all Rust files in the core module directory
        if check_function_in_directory(&core_dir, &app_call.function)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn resolve_function_location(app_call: &AppCall) -> Result<Option<String>> {
    // FUTURE IMPROVEMENT: Use rust-analyzer LSP for precise resolution
    // This would be the most accurate approach - query rust-analyzer directly:
    //
    // 1. Start rust-analyzer LSP server
    // 2. Send "textDocument/definition" request for the app.module().function() call
    // 3. Get back exact file path and line number
    // 4. This is exactly what IDEs like VSCode do for "Go to Definition"
    //
    // For now, use cargo metadata as a good approximation:

    // Try the LSP approach first (if rust-analyzer is available)
    if let Ok(location) = resolve_using_rust_analyzer(app_call) {
        return Ok(Some(location));
    }

    // Use the actual dependency graph from cargo metadata
    // This leverages Cargo's own resolution logic!
    let metadata = MetadataCommand::new()
        .manifest_path("../../Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .exec()?;

    // Find the admin-server package
    let workspace_packages = metadata.workspace_packages();
    let admin_server_pkg = workspace_packages
        .iter()
        .find(|pkg| pkg.name == "admin-server")
        .ok_or_else(|| anyhow!("Could not find admin-server package"))?;

    // Look through its actual dependencies for core modules
    for dependency in &admin_server_pkg.dependencies {
        if let Some(dep_path) = &dependency.path {
            // Check if this dependency matches our app module
            let dep_name = &dependency.name;

            // Convert dependency name to module name (e.g., "lana-customer" -> "customer")
            let module_candidates = [
                app_call.module.as_str(),
                &app_call.module.trim_end_matches('s'),
                &format!("{}s", app_call.module),
            ];

            for candidate in module_candidates {
                if dep_name.contains(candidate) || candidate.contains(dep_name) {
                    // Found the actual dependency! Use its real path
                    let src_dir = dep_path.join("src");

                    // Check lib.rs first
                    let lib_file = src_dir.join("lib.rs");
                    if lib_file.exists() {
                        return Ok(Some(lib_file.to_string()));
                    }

                    // Check mod.rs
                    let mod_file = src_dir.join("mod.rs");
                    if mod_file.exists() {
                        return Ok(Some(mod_file.to_string()));
                    }
                }
            }
        }
    }

    // Fallback: Look through all workspace packages
    let possible_package_names = [
        format!("lana-{}", app_call.module),
        format!("core-{}", app_call.module),
        format!("core_{}", app_call.module),
        app_call.module.trim_end_matches('s').to_string(),
        app_call.module.clone(),
    ];

    for package in metadata.workspace_packages() {
        if possible_package_names
            .iter()
            .any(|name| package.name.contains(name) || name.contains(&package.name))
        {
            let src_dir = package.manifest_path.parent().unwrap().join("src");

            let lib_file = src_dir.join("lib.rs");
            if lib_file.exists() {
                return Ok(Some(lib_file.to_string()));
            }

            let mod_file = src_dir.join("mod.rs");
            if mod_file.exists() {
                return Ok(Some(mod_file.to_string()));
            }
        }
    }

    Ok(None)
}

fn resolve_using_rust_analyzer(_app_call: &AppCall) -> Result<String> {
    // OPTION 1: Use rust-analyzer via LSP (Most Practical)
    // This would involve:
    // 1. Starting rust-analyzer as a subprocess
    // 2. Sending LSP "textDocument/definition" requests
    // 3. Getting back exact file:line locations
    //
    // Example implementation:
    // let lsp_response = send_lsp_request(GotoDefinitionParams {
    //     text_document_position_params: TextDocumentPositionParams {
    //         text_document: TextDocumentIdentifier {
    //             uri: "file:///.../schema.rs".parse()?
    //         },
    //         position: Position { line: 123, character: 45 },
    //     },
    //     work_done_progress_params: Default::default(),
    //     partial_result_params: Default::default(),
    // })?;

    // OPTION 2: Use cargo check with custom rustc flags
    // We can leverage rustc's --emit=metadata flag to get resolution info
    let output = Command::new("cargo")
        .args(&[
            "check",
            "--manifest-path",
            "../../Cargo.toml",
            "--package",
            "lana-admin-server",
            "--message-format=json",
        ])
        .current_dir("../../")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            // Parse cargo check JSON output for diagnostic information
            // This can give us information about where symbols are defined
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                    // Look for compiler messages that contain resolution info
                    if message["reason"] == "compiler-message" {
                        // This approach would need more development but is feasible
                    }
                }
            }
        }
    }

    // OPTION 3: Use the actual compiler resolution (Future)
    // For the ultimate solution, we could:
    // 1. Parse the source file with syn to find the exact position
    // 2. Use rustc_interface to run compiler analysis
    // 3. Query the resolver for exact symbol locations
    //
    // This would give us 100% accurate resolution but requires more complex setup

    Err(anyhow!(
        "Exact rust-analyzer resolution not yet implemented - using fallback"
    ))
}

fn check_function_in_directory(dir_path: &str, function_name: &str) -> Result<bool> {
    // Recursively find all .rs files in the directory
    let rust_files = find_rust_files(Path::new(dir_path))?;

    for file_path in rust_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if check_function_in_content(&content, function_name)? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

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

fn check_function_in_content(content: &str, function_name: &str) -> Result<bool> {
    let syntax_tree: File =
        syn::parse_str(content).map_err(|e| anyhow!("Failed to parse Rust file: {}", e))?;
    let mut visitor = AuthzVisitor::new(function_name);
    visitor.visit_file(&syntax_tree);
    Ok(visitor.has_authorization)
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

            // Check for subject_can_* patterns
            if method_name.starts_with("subject_can_") {
                self.has_authorization = true;
                return;
            }

            // Check for delegated authorization (functions that take 'sub' as first parameter)
            if self.is_delegation_call(node) {
                self.has_authorization = true;
                return;
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.in_target_function && !self.has_authorization {
            // Check for function calls that might be authorization related
            if let syn::Expr::Path(path) = &*node.func {
                if let Some(segment) = path.path.segments.last() {
                    let func_name = segment.ident.to_string();
                    if func_name.starts_with("subject_can_") {
                        self.has_authorization = true;
                        return;
                    }
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

impl AuthzVisitor {
    fn is_delegation_call(&self, method_call: &syn::ExprMethodCall) -> bool {
        let method_name = method_call.method.to_string();

        // Known delegation methods that pass authorization down
        let delegation_methods = [
            "import_from_csv",
            "add_root_node",
            "add_child_node",
            "create_user",
            "update_role_of_user",
            "create_role",
        ];

        if delegation_methods.contains(&method_name.as_str()) {
            // Check if 'sub' is passed as first argument
            if let Some(first_arg) = method_call.args.first() {
                if let syn::Expr::Path(path) = first_arg {
                    if path.path.segments.len() == 1 && path.path.segments[0].ident == "sub" {
                        return true;
                    }
                }
            }
        }

        false
    }
}
