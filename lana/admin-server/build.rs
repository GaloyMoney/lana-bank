use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=RELEASE_BUILD_VERSION");

    let version = env::var("RELEASE_BUILD_VERSION").unwrap_or_else(|_| {
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string())
    });
    println!("cargo:rustc-env=BUILD_VERSION={version}");

    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={profile}");

    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={target}");
}
