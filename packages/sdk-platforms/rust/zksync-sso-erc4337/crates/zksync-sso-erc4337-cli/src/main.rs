use zksync_sso_erc4337_cli::run_cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    run_cli().await
}
