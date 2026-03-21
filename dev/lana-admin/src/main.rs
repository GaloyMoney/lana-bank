#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // reqwest+rustls on rustls 0.23 requires explicitly setting a default
    // crypto provider before any TLS client is used.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    lana_admin::run_from_args(std::env::args_os()).await
}
