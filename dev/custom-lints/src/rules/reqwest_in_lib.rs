use std::path::Path;

use anyhow::Result;
use cargo_metadata::{CargoOpt, MetadataCommand};

use crate::{Violation, WorkspaceRule};

/// Rule that enforces reqwest dependency can only be used in lib/ tier.
///
/// HTTP clients should be infrastructure adapters in lib/, not in
/// core/ business logic or lana/ application layer. Core modules should
/// depend on lib/ adapters rather than using reqwest directly.
pub struct ReqwestInLibRule;

impl ReqwestInLibRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReqwestInLibRule {
    fn default() -> Self {
        Self::new()
    }
}

fn get_tier(path: &str) -> Option<&str> {
    if path.is_empty() {
        return None;
    }

    if path.starts_with("lib/") {
        Some("lib")
    } else if path.starts_with("core/") {
        Some("core")
    } else if path.starts_with("lana/") {
        Some("lana")
    } else {
        None
    }
}

impl WorkspaceRule for ReqwestInLibRule {
    fn name(&self) -> &'static str {
        "reqwest-in-lib"
    }

    fn description(&self) -> &'static str {
        "Enforces that reqwest dependency is only used in lib/ tier (HTTP clients must be infrastructure adapters)"
    }

    fn check_workspace(&self, workspace_root: &Path) -> Result<Vec<Violation>> {
        let manifest_path = workspace_root.join("Cargo.toml");
        let metadata = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec()?;

        let mut violations = Vec::new();

        for package in &metadata.workspace_packages() {
            let package_path = package.manifest_path.parent().unwrap();
            let relative_path = package_path.strip_prefix(&metadata.workspace_root).unwrap();

            // Only check core/ and lana/ tiers
            let tier: &str = match get_tier(relative_path.as_ref()) {
                Some("core") | Some("lana") => relative_path.as_ref(),
                _ => continue,
            };

            // Check if this package depends on reqwest
            for dependency in &package.dependencies {
                if dependency.name == "reqwest" {
                    violations.push(Violation::new(
                        self.name(),
                        package.manifest_path.as_str(),
                        format!(
                            "{} (in {}) cannot depend on reqwest directly - HTTP clients must be in lib/",
                            package.name, tier
                        ),
                    ));
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_detection() {
        assert_eq!(get_tier("lib/foo"), Some("lib"));
        assert_eq!(get_tier("core/bar"), Some("core"));
        assert_eq!(get_tier("lana/baz"), Some("lana"));
        assert_eq!(get_tier("dev/tool"), None);
        assert_eq!(get_tier(""), None);
    }
}
