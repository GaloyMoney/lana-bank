use std::env;

fn main() {
    // Tell cargo to rerun this script if the source files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=RELEASE_BUILD_VERSION");

    // Capture version - use RELEASE_BUILD_VERSION if set, otherwise use CARGO_PKG_VERSION
    let version = env::var("RELEASE_BUILD_VERSION").unwrap_or_else(|_| {
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string())
    });
    println!("cargo:rustc-env=BUILD_VERSION={version}");

    // Capture enabled features (deterministic based on build configuration)
    let features: Vec<&str> = vec![
        #[cfg(feature = "sim-bootstrap")]
        "sim-bootstrap",
        #[cfg(feature = "sumsub-testing")]
        "sumsub-testing",
        #[cfg(feature = "mock-custodian")]
        "mock-custodian",
    ];
    let features_str = features.join(",");
    println!("cargo:rustc-env=ENABLED_FEATURES={features_str}");

    // Capture build profile (deterministic based on build type)
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={profile}");

    // Capture target architecture (deterministic and functionally relevant)
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={target}");
}
