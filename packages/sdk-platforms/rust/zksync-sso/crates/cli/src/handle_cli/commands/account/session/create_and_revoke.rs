use super::super::constants::{
    DEPLOYED_ACCOUNT_ADDRESS, EXPECTED_SECOND_SESSION_HASH,
    SECOND_SESSION_OWNER_PRIVATE_KEY, create_second_session_config_json,
};
use crate::config::ConfigLoader;
use sdk::api::{
    account::session::{
        create::{CreateSessionArgs, create_session as sdk_create_session},
        hash::get_session_hash,
        revoke::{RevokeSessionArgs, revoke_session as sdk_revoke_session},
        session_lib::session_spec_from_json,
        state::{GetSessionStateArgs, get_session_state},
    },
    utils::{
        parse_address, parse_paymaster_params, sign_fn_from_private_key_hex,
    },
};

pub async fn create_and_revoke_session_with_account(
    account_address: &str,
) -> eyre::Result<()> {
    let config = ConfigLoader::load()?;

    println!(
        "Creating session using hardcoded test configuration (second session from test_api_create_session)..."
    );

    let deployed_account_address = parse_address(account_address)?;

    println!("Account address: {deployed_account_address}");

    let session_config_json = create_second_session_config_json();

    let session_config = session_spec_from_json(&session_config_json)?;

    let session_hash = get_session_hash(session_config.clone())?;
    let session_hash_str = format!(
        "0x{}",
        session_hash
            .bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );

    println!("Session hash to create: {session_hash_str}");

    let expected_hash = EXPECTED_SECOND_SESSION_HASH;
    eyre::ensure!(
        session_hash_str == expected_hash,
        "Session hash does not match expected value. Expected: {}, Got: {}",
        expected_hash,
        session_hash_str
    );

    println!(
        "✓ Session hash matches expected value from test_api_create_session"
    );

    let sign_fn =
        sign_fn_from_private_key_hex(SECOND_SESSION_OWNER_PRIVATE_KEY)?;

    let paymaster_params = parse_paymaster_params(
        config.contracts.account_paymaster.to_string(),
        None,
    )?;

    let create_args = CreateSessionArgs {
        account: deployed_account_address,
        session_config: session_config.clone(),
        paymaster: Some(paymaster_params),
    };

    println!("Attempting to create session...");

    let create_result =
        sdk_create_session(create_args, sign_fn, &config).await?;

    println!("Session creation successful!");
    println!("  Session ID: {session_hash_str}");
    println!("  Account: {deployed_account_address}");
    println!(
        "  Transaction receipt: {}",
        create_result.transaction_receipt_json
    );

    println!("\n--- Verifying session state ---");
    let session_state = get_session_state(
        GetSessionStateArgs {
            account: deployed_account_address,
            session_config: session_config.clone(),
        },
        &config,
    )
    .await?;

    println!("Session state after creation:");
    println!("  Status: {:?} (1 = Active)", session_state.session_state.status);
    println!(
        "  Fees remaining: {:?}",
        session_state.session_state.fees_remaining
    );
    println!(
        "  Transfer value entries: {}",
        session_state.session_state.transfer_value.len()
    );

    eyre::ensure!(
        session_state.session_state.status.is_active(),
        "Session should be active (status=1)"
    );

    let expected_fee_limit = session_config.fee_limit.limit;
    eyre::ensure!(
        session_state.session_state.fees_remaining == expected_fee_limit,
        "Session fee limit should match configuration"
    );

    println!("✓ Session successfully created and verified");

    println!("\n--- Revoking session ---");

    let revoke_sign_fn =
        sign_fn_from_private_key_hex(SECOND_SESSION_OWNER_PRIVATE_KEY)?;

    let revoke_args =
        RevokeSessionArgs { session_id: session_hash_str.clone() };

    println!("Attempting to revoke session...");

    let revoke_result = sdk_revoke_session(
        revoke_args,
        deployed_account_address,
        revoke_sign_fn,
        &config,
    )
    .await?;

    println!("Session revocation successful!");
    println!("  Session ID: {session_hash_str}");
    println!("  Account: {deployed_account_address}");
    println!(
        "  Transaction receipt: {}",
        revoke_result.transaction_receipt_json
    );

    println!("\n--- Verifying session state after revocation ---");
    let session_state_after_revoke = get_session_state(
        GetSessionStateArgs {
            account: deployed_account_address,
            session_config: session_config.clone(),
        },
        &config,
    )
    .await?;

    println!("Session state after revocation:");
    println!(
        "  Status: {:?} (0 = Revoked)",
        session_state_after_revoke.session_state.status
    );
    println!(
        "  Fees remaining: {:?}",
        session_state_after_revoke.session_state.fees_remaining
    );

    eyre::ensure!(
        !session_state_after_revoke.session_state.status.is_active(),
        "Session should be revoked (status=0)"
    );

    println!("✓ Session successfully revoked and verified");

    Ok(())
}

pub async fn create_and_revoke_session() -> eyre::Result<()> {
    create_and_revoke_session_with_account(DEPLOYED_ACCOUNT_ADDRESS).await
}
