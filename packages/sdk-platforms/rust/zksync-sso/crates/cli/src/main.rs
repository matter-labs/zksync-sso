use cli::handle_cli::handle_cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    handle_cli().await?;

    Ok(())
}
