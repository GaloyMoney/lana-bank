use std::path::Path;

pub mod rules;

/// A lint violation found during analysis
#[derive(Debug, Clone)]
pub struct Violation {
    /// Name of the rule that found this violation
    pub rule: &'static str,
    /// File path where the violation was found
    pub file: String,
    /// Line number (1-indexed), if applicable
    pub line: Option<usize>,
    /// Description of the violation
    pub message: String,
}

impl Violation {
    pub fn new(rule: &'static str, file: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rule,
            file: file.into(),
            line: None,
            message: message.into(),
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(
                f,
                "[{}] {}:{}: {}",
                self.rule, self.file, line, self.message
            ),
            None => write!(f, "[{}] {}: {}", self.rule, self.file, self.message),
        }
    }
}

/// Trait for implementing custom lint rules
pub trait LintRule: Send + Sync {
    /// Returns the name of this rule
    fn name(&self) -> &'static str;

    /// Returns a brief description of what this rule checks
    fn description(&self) -> &'static str;

    /// Check a parsed Rust file for violations
    fn check_file(&self, file: &syn::File, path: &Path) -> Vec<Violation>;
}

/// Trait for rules that operate on the workspace level (not individual files)
pub trait WorkspaceRule: Send + Sync {
    /// Returns the name of this rule
    fn name(&self) -> &'static str;

    /// Returns a brief description of what this rule checks
    fn description(&self) -> &'static str;

    /// Check the workspace for violations
    fn check_workspace(&self, workspace_root: &Path) -> anyhow::Result<Vec<Violation>>;
}
