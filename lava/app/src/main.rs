use lava_app::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
