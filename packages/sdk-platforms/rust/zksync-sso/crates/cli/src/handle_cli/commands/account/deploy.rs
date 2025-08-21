use super::constants::{
    EXPIRES_AT, RANDOM_SALT_STR, create_session_config_json, get_test_addresses,
};
use crate::config::ConfigLoader;
use sdk::api::{
    account::{
        fund::fund_account,
        modular_account::{
            DeployModularAccountArgs, SessionModuleArgs, deploy_modular_account,
        },
        session::session_lib::session_spec_from_json,
    },
    utils::parse_address,
};

pub async fn deploy_account() -> eyre::Result<()> {
    let config = ConfigLoader::load()?;
    let test_addresses = get_test_addresses();

    let session_config_json = create_session_config_json(
        test_addresses.session_owner,
        test_addresses.transfer_session_target,
        EXPIRES_AT,
    );

    let session_config = session_spec_from_json(&session_config_json)?;

    let deploy_args = DeployModularAccountArgs {
        install_no_data_modules: vec![],
        owners: vec![parse_address(test_addresses.owner)?],
        session_module: Some(SessionModuleArgs {
            location: config.contracts.session,
            initial_session: Some(session_config),
        }),
        paymaster: None,
        passkey_module: None,
        unique_account_id: Some(RANDOM_SALT_STR.to_string()),
    };

    let result = deploy_modular_account(deploy_args, &config).await?;

    println!("Account deployed successfully!");
    println!("  Address: {}", result.address);
    println!("  Unique ID: {}", result.unique_account_id);

    println!("\nFunding account with 1 ETH...");
    let funding_amount = sdk::api::utils::u256_from(1000000000000000000u64); // 1 ETH (10^18 wei)
    fund_account(result.address, funding_amount, &config).await?;
    println!("Account funded successfully!");

    Ok(())
}
