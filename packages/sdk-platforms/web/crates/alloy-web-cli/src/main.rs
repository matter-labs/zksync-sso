#[tokio::main]
async fn main() -> eyre::Result<()> {
    alloy_web_cli::run().await
}
