use crate::erc4337::account::modular_smart_account::passkey::contract::WebAuthnValidator;
use alloy::{primitives::Bytes, sol_types::SolCall};

pub fn get_account_list_call_data(
    domain: String,
    credential_id: Vec<u8>,
) -> Bytes {
    WebAuthnValidator::getAccountListCall {
        domain,
        credentialId: credential_id.into(),
    }
    .abi_encode()
    .into()
}
