use cli::{
    config::ConfigLoader,
    handle_cli::commands::{
        account::{
            deploy::deploy_account,
            session::{
                create_and_revoke::create_and_revoke_session,
                send::send_transaction,
            },
        },
        deploy_contracts::deploy_contracts_and_update_example_configs,
    },
};
use url::Url;

#[tokio::test]
#[ignore]
async fn test_complete_zksync_sso_integration() -> eyre::Result<()> {
    println!("\n{}", "=".repeat(100));
    println!("🚀 RUNNING COMPLETE ZKSYNC SSO INTEGRATION TEST 🚀");
    println!("{}", "=".repeat(100));

    // Step 0: Deploy contracts and create config
    println!("\n🚀 STEP 0: Deploy Contracts and Create Config");
    let node_url = Url::parse("http://localhost:8011/")?;
    deploy_contracts_and_update_example_configs(
        node_url,
        ConfigLoader::get_all_default_config_paths(),
    )
    .await?;
    println!("✅ Contracts deployed and config created");

    // Verify config is now available
    match ConfigLoader::load() {
        Ok(_) => println!("✅ Config loaded successfully"),
        Err(e) => {
            println!("❌ Config still not found after deployment: {e}");
            return Err(e);
        }
    }

    // Step 1: Deploy Account
    println!("\n📦 STEP 1: Deploy Account");
    deploy_account().await?;
    println!("✅ Deploy account completed");

    // Step 2: Create and Revoke Session
    // Uses the deterministic account address from constants
    println!("\n🔑 STEP 2: Create and Revoke Session");
    create_and_revoke_session().await?;
    println!("✅ Create and revoke session completed");

    // Step 3: Session Send Transaction
    // This deploys its own account with a different session owner
    println!("\n💸 STEP 3: Session Send Transaction");
    send_transaction().await?;
    println!("✅ Send transaction completed");

    println!("\n{}", "=".repeat(100));
    println!("✅ ALL INTEGRATION TESTS COMPLETED SUCCESSFULLY ✅");
    println!("{}", "=".repeat(100));
    println!("Integration Test Summary:");
    println!("0. ✅ Deployed contracts and created config files");
    println!("1. ✅ Deployed modular account with session module");
    println!("2. ✅ Created and revoked session successfully");
    println!("3. ✅ Deployed account with session and sent transaction");
    println!("4. ✅ All session states verified correctly");
    println!("5. ✅ All blockchain interactions successful");

    Ok(())
}

// Individual test cases for debugging
#[tokio::test]
#[ignore]
async fn test_individual_deploy_account() -> eyre::Result<()> {
    println!("\n📦 Testing Deploy Account individually");
    deploy_account().await
}

#[tokio::test]
#[ignore]
async fn test_individual_create_and_revoke() -> eyre::Result<()> {
    println!("\n🔑 Testing Create and Revoke Session individually");
    // First deploy an account
    deploy_account().await?;
    // Then test create and revoke
    create_and_revoke_session().await
}

#[tokio::test]
#[ignore]
async fn test_individual_send_transaction() -> eyre::Result<()> {
    println!("\n💸 Testing Send Transaction individually");
    send_transaction().await
}
