use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    check_dependency_dag::run().await
}
