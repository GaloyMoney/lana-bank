use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use rayon::prelude::*;
use walkdir::WalkDir;

use custom_lints::rules::{
    DbOpConventionRule, DependencyDagRule, TransactionCommitRule, UnwrapUsageRule,
};
use custom_lints::{LintRule, Violation, WorkspaceRule};

fn main() -> ExitCode {
    match run() {
        Ok((violations, summary)) => {
            println!("\n{}", summary);
            if violations.is_empty() {
                println!("\nAll custom lint checks passed!");
                ExitCode::SUCCESS
            } else {
                println!("\nCustom lint violations found:\n");
                for violation in &violations {
                    println!("  - {}", violation);
                }
                println!("\nTotal: {} violation(s)", violations.len());
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("Error running custom lints: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(Vec<Violation>, String)> {
    let workspace_root = find_workspace_root()?;
    let mut all_violations = Vec::new();
    let mut summary_lines = vec!["Lint Rules Executed:".to_string()];

    // Run workspace-level rules
    let workspace_rules: Vec<Box<dyn WorkspaceRule>> = vec![Box::new(DependencyDagRule::new())];

    summary_lines.push(format!("\n  Workspace Rules ({}):", workspace_rules.len()));
    for rule in &workspace_rules {
        summary_lines.push(format!("    [{}] {}", rule.name(), rule.description()));
        let violations = rule.check_workspace(&workspace_root)?;
        all_violations.extend(violations);
    }

    // Run file-level rules
    let file_rules: Vec<Box<dyn LintRule>> = vec![
        Box::new(TransactionCommitRule::new()),
        Box::new(DbOpConventionRule::new()),
        Box::new(UnwrapUsageRule::new()),
    ];

    if !file_rules.is_empty() {
        let dirs_to_check = vec!["core", "lana", "lib"];
        let rust_files = collect_rust_files(&workspace_root, &dirs_to_check);

        summary_lines.push(format!(
            "\n  File Rules ({}) - checked {} files:",
            file_rules.len(),
            rust_files.len()
        ));
        for rule in &file_rules {
            summary_lines.push(format!("    [{}] {}", rule.name(), rule.description()));
        }

        let file_violations: Vec<Violation> = rust_files
            .par_iter()
            .flat_map(|file_path| check_file(&file_rules, file_path, &workspace_root))
            .collect();

        all_violations.extend(file_violations);
    }

    Ok((all_violations, summary_lines.join("\n")))
}

fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(current);
            }
        }
        if !current.pop() {
            anyhow::bail!("Could not find workspace root (no Cargo.toml with [workspace] found)");
        }
    }
}

fn collect_rust_files(workspace_root: &Path, dirs: &[&str]) -> Vec<PathBuf> {
    dirs.iter()
        .flat_map(|dir| {
            let dir_path = workspace_root.join(dir);
            WalkDir::new(&dir_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "rs") && e.file_type().is_file()
                })
                .map(|e| e.path().to_path_buf())
        })
        .collect()
}

fn check_file(
    rules: &[Box<dyn LintRule>],
    file_path: &Path,
    workspace_root: &Path,
) -> Vec<Violation> {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not read {}: {}", file_path.display(), e);
            return vec![];
        }
    };

    let parsed = match syn::parse_file(&content) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Warning: Could not parse {}: {}", file_path.display(), e);
            return vec![];
        }
    };

    let relative_path = file_path.strip_prefix(workspace_root).unwrap_or(file_path);

    rules
        .iter()
        .flat_map(|rule| rule.check_file(&parsed, relative_path))
        .collect()
}
