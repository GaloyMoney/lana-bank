#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lana_cli::run().await
}
