use crate::erc4337::{
    account::{
        erc7579::{Execution, calls::encode_calls},
        modular_smart_account::{
            WebAuthnValidator,
            add_passkey::PasskeyPayload,
            send::{SendParams, send_transaction},
        },
    },
    bundler::pimlico::client::BundlerClient,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
    sol_types::SolCall,
};

pub async fn add_passkey<P>(
    account_address: Address,
    passkey: PasskeyPayload,
    entry_point_address: Address,
    webauthn_validator_address: Address,
    bundler_client: BundlerClient,
    provider: P,
    signer: Signer,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let call_data = add_passkey_call_data(passkey, webauthn_validator_address);

    send_transaction(SendParams {
        account: account_address,
        entry_point: entry_point_address,
        factory_payload: None,
        call_data,
        nonce_key: None,
        paymaster: None,
        bundler_client,
        provider,
        signer,
    })
    .await?;

    Ok(())
}

fn add_passkey_call_data(
    passkey: PasskeyPayload,
    webauthn_validator_address: Address,
) -> Bytes {
    let credential_id = passkey.credential_id;
    let origin_domain = passkey.origin_domain;
    let new_key = passkey.passkey;

    let add_validation_key_calldata = WebAuthnValidator::addValidationKeyCall {
        credentialId: credential_id,
        newKey: new_key,
        domain: origin_domain,
    }
    .abi_encode()
    .into();

    let call = {
        let target = webauthn_validator_address;
        let value = U256::from(0);
        let data = add_validation_key_calldata;
        Execution { target, value, data }
    };

    let calls = vec![call];
    encode_calls(calls).into()
}
