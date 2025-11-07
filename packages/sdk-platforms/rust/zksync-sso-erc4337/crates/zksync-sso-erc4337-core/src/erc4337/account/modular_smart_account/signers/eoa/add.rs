use crate::erc4337::{
    account::{
        erc7579::{Execution, calls::encode_calls},
        modular_smart_account::{
            send::{SendParams, send_transaction},
            signers::eoa::EOAKeyValidator,
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

pub async fn add_owner<P>(
    account_address: Address,
    new_owner: Address,
    entry_point_address: Address,
    eoa_validator_address: Address,
    bundler_client: BundlerClient,
    provider: P,
    signer: Signer,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let call_data = add_owner_call_data(new_owner, eoa_validator_address);

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

fn add_owner_call_data(
    new_owner: Address,
    eoa_validator_address: Address,
) -> Bytes {
    let add_owner_calldata =
        EOAKeyValidator::addOwnerCall { owner: new_owner }.abi_encode().into();

    let call = {
        let target = eoa_validator_address;
        let value = U256::from(0);
        let data = add_owner_calldata;
        Execution { target, value, data }
    };

    let calls = vec![call];
    encode_calls(calls).into()
}
