use anyhow::{Result, anyhow};
use cargo_metadata::{CargoOpt, MetadataCommand};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum Tier {
    Lib = 0,
    Core = 1,
    Lana = 2,
}

pub async fn check_dependency_dag() -> Result<()> {
    let metadata = MetadataCommand::new()
        .manifest_path("../../Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .exec()?;

    let mut violations = Vec::new();

    for package in &metadata.workspace_packages() {
        let package_path = package.manifest_path.parent().unwrap();
        let relative_path = package_path.strip_prefix(&metadata.workspace_root).unwrap();

        // Skip packages that don't follow our tier structure (like tools and scripts)
        let tier = match get_tier(relative_path.as_ref()) {
            Ok(tier) => tier,
            Err(_) => continue, // Skip packages not in our tier structure
        };

        for dependency in &package.dependencies {
            if let Some(dep_path) = &dependency.path {
                let dep_relative = dep_path.strip_prefix(&metadata.workspace_root).unwrap();
                let dep_tier = match get_tier(dep_relative.as_ref()) {
                    Ok(tier) => tier,
                    Err(_) => continue, // Skip dependencies not in our tier structure
                };

                if !is_valid_dependency(tier, dep_tier) {
                    violations.push(format!(
                        "{} ({:?}) cannot depend on {} ({:?})",
                        package.name, tier, dependency.name, dep_tier
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        println!("✅ All dependency rules are satisfied!");
        Ok(())
    } else {
        println!("❌ Dependency DAG violations found:");
        for violation in &violations {
            println!("  - {violation}");
        }
        std::process::exit(1);
    }
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
