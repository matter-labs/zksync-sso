use alloy::primitives::Address;
use wasm_bindgen::prelude::*;
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::{
    propose::propose_guardian_call_data,
    remove::remove_guardian_call_data,
};

/// Encode the call data for proposing a guardian (no signing, just encoding)
///
/// Returns the complete encoded call data ready to be sent via account.execute()
/// This includes both the mode and execution calldata in the format expected by ERC-7579.
///
/// # Parameters
/// * `guardian_executor` - Address of the GuardianExecutor contract
/// * `new_guardian` - Address of the guardian to propose
///
/// # Returns
/// Hex-encoded call data (0x-prefixed) for account.execute(mode, executionCalldata)
/// The returned data is a complete executeCall that can be sent directly to the account
#[wasm_bindgen]
pub fn encode_propose_guardian_call_data(
    guardian_executor: String,
    new_guardian: String,
) -> Result<String, JsValue> {
    // Parse addresses
    let guardian_executor_addr =
        guardian_executor.parse::<Address>().map_err(|e| {
            JsValue::from_str(&format!(
                "Invalid guardian executor address: {}",
                e
            ))
        })?;

    let new_guardian_addr = new_guardian.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid new guardian address: {}", e))
    })?;

    // Get the encoded call data
    let call_data =
        propose_guardian_call_data(new_guardian_addr, guardian_executor_addr);

    // Return as hex string
    Ok(format!("0x{}", hex::encode(call_data)))
}

/// Encode the call data for removing a guardian (no signing, just encoding)
///
/// Returns the complete encoded call data ready to be sent via account.execute()
/// This includes both the mode and execution calldata in the format expected by ERC-7579.
///
/// # Parameters
/// * `guardian_executor` - Address of the GuardianExecutor contract
/// * `guardian_to_remove` - Address of the guardian to remove
///
/// # Returns
/// Hex-encoded call data (0x-prefixed) for account.execute(mode, executionCalldata)
/// The returned data is a complete executeCall that can be sent directly to the account
#[wasm_bindgen]
pub fn encode_remove_guardian_call_data(
    guardian_executor: String,
    guardian_to_remove: String,
) -> Result<String, JsValue> {
    // Parse addresses
    let guardian_executor_addr =
        guardian_executor.parse::<Address>().map_err(|e| {
            JsValue::from_str(&format!(
                "Invalid guardian executor address: {}",
                e
            ))
        })?;

    let guardian_to_remove_addr =
        guardian_to_remove.parse::<Address>().map_err(|e| {
            JsValue::from_str(&format!(
                "Invalid guardian to remove address: {}",
                e
            ))
        })?;

    // Get the encoded call data
    let call_data = remove_guardian_call_data(
        guardian_to_remove_addr,
        guardian_executor_addr,
    );

    // Return as hex string
    Ok(format!("0x{}", hex::encode(call_data)))
}
