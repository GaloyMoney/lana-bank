use std::env;

fn main() {
    // Tell cargo to rerun this script if the source files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Capture enabled features
    let mut features: Vec<&str> = Vec::new();

    #[cfg(feature = "sim-time")]
    features.push("sim-time");

    #[cfg(feature = "sim-bootstrap")]
    features.push("sim-bootstrap");

    #[cfg(feature = "fail-on-warnings")]
    features.push("fail-on-warnings");

    // Convert features to a comma-separated string
    let features_str = features.join(",");
    println!("cargo:rustc-env=ENABLED_FEATURES={}", features_str);

    // Capture build profile
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    // Capture target triple
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);

    // Capture host triple
    let host = env::var("HOST").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_HOST={}", host);

    // Capture build timestamp
    let build_time = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_time);
}
