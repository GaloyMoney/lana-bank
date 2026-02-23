#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

fn main() -> anyhow::Result<()> {
    // Install ring as the default rustls CryptoProvider before any TLS usage.
    // google-cloud-* crates pull in reqwest 0.13 which uses rustls 0.23 and
    // requires an explicit crypto provider to be installed.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // async-graphql 8's codegen produces deeper stack frames during field resolution.
    // The default 2MB tokio worker thread stack is insufficient for our schema size,
    // so we use 8MB worker stacks to avoid stack overflow.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()?
        .block_on(lana_cli::run())
}
