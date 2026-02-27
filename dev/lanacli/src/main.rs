#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lanacli::run_from_args(std::env::args_os()).await
}
