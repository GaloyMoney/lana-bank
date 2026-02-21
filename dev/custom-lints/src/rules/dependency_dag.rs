use std::path::Path;

use anyhow::{Result, anyhow};
use cargo_metadata::{CargoOpt, DependencyKind, MetadataCommand};

use crate::{Violation, WorkspaceRule};

/// Rule that enforces the dependency DAG between workspace tiers.
///
/// The workspace is organized into three tiers:
/// - `lib/` - Shared libraries (lowest tier)
/// - `core/` - Domain logic modules (middle tier)
/// - `lana/` - Application layer (highest tier)
///
/// Dependencies can only flow downward: lana -> core -> lib
pub struct DependencyDagRule;

impl DependencyDagRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DependencyDagRule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum Tier {
    Lib = 0,
    Core = 1,
    Lana = 2,
}

fn get_tier(path: &str) -> Result<Tier> {
    if path.is_empty() {
        return Err(anyhow!("Empty path provided"));
    }

    if path.starts_with("lib/") {
        Ok(Tier::Lib)
    } else if path.starts_with("core/") {
        Ok(Tier::Core)
    } else if path.starts_with("lana/") {
        Ok(Tier::Lana)
    } else {
        Err(anyhow!("Unknown tier for path: '{}'", path))
    }
}

fn is_valid_dependency(from_tier: Tier, to_tier: Tier) -> bool {
    // Can only depend on same tier or lower tier
    from_tier >= to_tier
}

impl WorkspaceRule for DependencyDagRule {
    fn name(&self) -> &'static str {
        "dependency-dag"
    }

    fn description(&self) -> &'static str {
        "Enforces layered architecture: lib <- core <- lana dependency order"
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

            // Skip packages that don't follow our tier structure (like tools and scripts)
            let tier = match get_tier(relative_path.as_ref()) {
                Ok(tier) => tier,
                Err(_) => continue,
            };

            for dependency in &package.dependencies {
                // Dev-dependencies are allowed to violate the DAG (e.g., core tests using lana adapters)
                if dependency.kind == DependencyKind::Development {
                    continue;
                }
                if let Some(dep_path) = &dependency.path {
                    let dep_relative = dep_path.strip_prefix(&metadata.workspace_root).unwrap();
                    let dep_tier = match get_tier(dep_relative.as_ref()) {
                        Ok(tier) => tier,
                        Err(_) => continue,
                    };

                    if !is_valid_dependency(tier, dep_tier) {
                        violations.push(Violation::new(
                            self.name(),
                            package.manifest_path.as_str(),
                            format!(
                                "{} ({:?}) cannot depend on {} ({:?})",
                                package.name, tier, dependency.name, dep_tier
                            ),
                        ));
                    }
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
    fn test_tier_parsing() {
        assert_eq!(get_tier("lib/foo").unwrap(), Tier::Lib);
        assert_eq!(get_tier("core/bar").unwrap(), Tier::Core);
        assert_eq!(get_tier("lana/baz").unwrap(), Tier::Lana);
        assert!(get_tier("dev/tool").is_err());
        assert!(get_tier("").is_err());
    }

    #[test]
    fn test_valid_dependencies() {
        // Same tier dependencies are valid
        assert!(is_valid_dependency(Tier::Lib, Tier::Lib));
        assert!(is_valid_dependency(Tier::Core, Tier::Core));
        assert!(is_valid_dependency(Tier::Lana, Tier::Lana));

        // Downward dependencies are valid
        assert!(is_valid_dependency(Tier::Core, Tier::Lib));
        assert!(is_valid_dependency(Tier::Lana, Tier::Core));
        assert!(is_valid_dependency(Tier::Lana, Tier::Lib));

        // Upward dependencies are invalid
        assert!(!is_valid_dependency(Tier::Lib, Tier::Core));
        assert!(!is_valid_dependency(Tier::Lib, Tier::Lana));
        assert!(!is_valid_dependency(Tier::Core, Tier::Lana));
    }
}
