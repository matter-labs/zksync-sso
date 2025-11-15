use crate::erc4337::{
    account::{
        erc7579::calls::encoded_call_with_target_and_data,
        modular_smart_account::{
            send::{SendUserOpParams, send_user_op},
            signers::eoa::EOAKeyValidator,
        },
    },
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    sol_types::SolCall,
};

pub struct AddOwnerParams<P: Provider + Send + Sync + Clone> {
    pub account_address: Address,
    pub new_owner: Address,
    pub entry_point_address: Address,
    pub eoa_validator_address: Address,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub signer: Signer,
}

pub async fn add_owner<P>(params: AddOwnerParams<P>) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let AddOwnerParams {
        account_address,
        new_owner,
        entry_point_address,
        eoa_validator_address,
        paymaster,
        bundler_client,
        provider,
        signer,
    } = params;

    let call_data = add_owner_call_data(new_owner, eoa_validator_address);

    send_user_op(SendUserOpParams {
        account: account_address,
        entry_point: entry_point_address,
        factory_payload: None,
        call_data,
        nonce_key: None,
        paymaster,
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

    encoded_call_with_target_and_data(eoa_validator_address, add_owner_calldata)
}
