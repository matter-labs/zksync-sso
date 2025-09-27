use alloy_web_core::primitives::{Address, Bytes, U256, hex};
use alloy_web_core::sol_types::SolCall;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct SimpleAccountConfig {
    pub factory_address: String,
    pub owner_address: String,
    pub salt: Option<String>,
    pub entry_point: Option<String>,
}

#[derive(Serialize)]
pub struct DeploymentInfo {
    pub factory_address: String,
    pub predicted_address: String,
    pub owner: String,
    pub salt: String,
    pub init_code: String,
    pub needs_deployment: bool,
}

#[derive(Serialize)]
pub struct SimpleAccountDeploymentResult {
    pub account_address: String,
    pub transaction_hash: Option<String>,
    pub already_deployed: bool,
}

/// Calculate the counterfactual address for a SimpleAccount
#[wasm_bindgen]
pub fn get_account_address(config: JsValue) -> Result<JsValue, JsValue> {
    let config: SimpleAccountConfig = serde_wasm_bindgen::from_value(config)?;

    let factory = config
        .factory_address
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid factory address: {e}")))?;
    let owner = config
        .owner_address
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid owner address: {e}")))?;
    let salt = config.salt.unwrap_or_else(|| "0".to_string());
    let salt_value = salt.parse::<u64>().unwrap_or(0);

    let predicted_address = alloy_web_core::erc4337::account::compute_account_address(
        factory,
        owner,
        U256::from(salt_value),
    );

    let init_code =
        alloy_web_core::erc4337::account::get_init_code(factory, owner, U256::from(salt_value));

    let result = DeploymentInfo {
        factory_address: config.factory_address,
        predicted_address: format!("{predicted_address:?}"),
        owner: config.owner_address,
        salt,
        init_code: format!("0x{}", hex::encode(init_code)),
        needs_deployment: true, // Will be checked via JS provider
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

/// Generate the calldata for deploying a SimpleAccount
#[wasm_bindgen]
pub fn get_deploy_account_calldata(owner: &str, salt: &str) -> Result<String, JsValue> {
    use alloy_web_core::erc4337::contracts::SimpleAccountFactory;

    let owner = owner
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid owner address: {e}")))?;

    let salt_value = salt.parse::<u64>().unwrap_or(0);

    let call = SimpleAccountFactory::createAccountCall {
        owner,
        salt: U256::from(salt_value),
    };

    Ok(format!("0x{}", hex::encode(call.abi_encode())))
}

/// Get the init code for SimpleAccount deployment
#[wasm_bindgen]
pub fn get_account_init_code(factory: &str, owner: &str, salt: &str) -> Result<String, JsValue> {
    let factory = factory
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid factory address: {e}")))?;
    let owner = owner
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid owner address: {e}")))?;
    let salt_value = salt.parse::<u64>().unwrap_or(0);

    let init_code =
        alloy_web_core::erc4337::account::get_init_code(factory, owner, U256::from(salt_value));

    Ok(format!("0x{}", hex::encode(init_code)))
}

/// Build a UserOperation for account deployment
#[wasm_bindgen]
pub fn build_deploy_user_op(
    factory: &str,
    owner: &str,
    salt: &str,
    entry_point: &str,
) -> Result<JsValue, JsValue> {
    use alloy_web_core::erc4337::PackedUserOperation;

    let factory = factory
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid factory address: {e}")))?;
    let owner = owner
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid owner address: {e}")))?;
    let _entry_point = entry_point
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid entry point address: {e}")))?;
    let salt_value = salt.parse::<u64>().unwrap_or(0);

    let sender = alloy_web_core::erc4337::account::compute_account_address(
        factory,
        owner,
        U256::from(salt_value),
    );

    let init_code =
        alloy_web_core::erc4337::account::get_init_code(factory, owner, U256::from(salt_value));

    // Build a packed UserOperation for v0.7
    use alloy_web_core::primitives::{FixedBytes, U128};

    let call_gas_limit = U128::from(100_000);
    let verification_gas_limit = U128::from(200_000);
    let max_priority_fee_per_gas = U128::from(1_000_000_000);
    let max_fee_per_gas = U128::from(1_000_000_000);

    // Pack account gas limits
    let mut account_gas_limits = [0u8; 32];
    account_gas_limits[..16].copy_from_slice(&verification_gas_limit.to_be_bytes::<16>());
    account_gas_limits[16..].copy_from_slice(&call_gas_limit.to_be_bytes::<16>());

    // Pack gas fees
    let mut gas_fees = [0u8; 32];
    gas_fees[..16].copy_from_slice(&max_priority_fee_per_gas.to_be_bytes::<16>());
    gas_fees[16..].copy_from_slice(&max_fee_per_gas.to_be_bytes::<16>());

    let user_op = PackedUserOperation {
        sender,
        nonce: U256::ZERO,
        init_code: Bytes::from(init_code),
        call_data: Bytes::default(),
        account_gas_limits: FixedBytes::from(account_gas_limits),
        pre_verification_gas: U256::from(21_000),
        gas_fees: FixedBytes::from(gas_fees),
        paymaster_and_data: Bytes::default(),
        signature: Bytes::default(),
    };

    Ok(serde_wasm_bindgen::to_value(&user_op)?)
}
