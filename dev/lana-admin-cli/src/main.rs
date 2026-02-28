#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lana_admin_cli::run_from_args(std::env::args_os()).await
}
