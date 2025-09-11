use crate::erc4337::{
    account::{
        is_deployed::is_smart_account_deployed,
        simple_account::{
            contracts::SimpleAccountFactory,
            deploy::{
                config::Config, create::SimpleAccountCreate,
                execute::SimpleAccountExecute,
            },
            signature,
        },
    },
    bundler::{
        config::BundlerConfig,
        pimlico::client::BundlerClient as PimlicoBundlerClient,
    },
    entry_point::{nonce::get_nonce, sender_address::get_sender_address_v07},
    paymaster::pimlico::client::PaymasterClient,
};
use alloy::{
    hex,
    primitives::{Address, Bytes, U256},
    providers::ProviderBuilder,
    rpc::types::erc4337::PackedUserOperation,
    signers::local::PrivateKeySigner,
};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod config;
pub mod create;
pub mod ensure;
pub mod execute;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
}

pub fn deploy_account() -> eyre::Result<Address> {
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOperationEstimated(PackedUserOperation);

impl From<UserOperationEstimated> for PackedUserOperation {
    fn from(val: UserOperationEstimated) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub struct SignedUserOperation(PackedUserOperation);

impl From<SignedUserOperation> for PackedUserOperation {
    fn from(val: SignedUserOperation) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SentUserOperationHash(String);

impl From<SentUserOperationHash> for String {
    fn from(user_operation_hash: SentUserOperationHash) -> Self {
        user_operation_hash.0
    }
}

impl fmt::Display for SentUserOperationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn send_transaction_with_signer(
    transaction: Transaction,
    config: Config,
    chain_id: u64,
    signer: PrivateKeySigner,
) -> eyre::Result<String> {
    let bundler_base_url = config.clone().endpoints.bundler.base_url;
    let paymaster_base_url = config.clone().endpoints.paymaster.base_url;
    let rpc_base_url = config.clone().endpoints.rpc.base_url;

    // use crate::entry_point::EntryPointVersion;
    // let chain_id = ChainId::new_eip155(chain_id);
    // let chain = crate::chain::Chain::new(chain_id, EntryPointVersion::V07, "");
    // let entry_point_config = chain.entry_point_config();
    // let rpc_base_url: Url = rpc_base_url.parse()?;

    let pimlico_client: PimlicoBundlerClient =
        PimlicoBundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

    let entry_point_address = Address::ZERO; // TODO: use the correct entrypoint address

    // Create a provider
    let alloy_signer = signer;
    let ethereum_wallet =
        alloy::network::EthereumWallet::new(alloy_signer.clone());

    let owner = ethereum_wallet.clone().default_signer();
    let owner_address = owner.address();

    let rpc_url = rpc_base_url.parse()?;

    let provider = ProviderBuilder::new()
        .wallet(ethereum_wallet.clone())
        .connect_http(rpc_url);

    let simple_account_factory_address_primitives: Address =
        "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985".parse()?; // TODO: use the correct simple account factory address
    let simple_account_factory_address =
        simple_account_factory_address_primitives;

    let factory_data_call = SimpleAccountCreate::new_u64(owner_address, 2);
    let factory_data_value = factory_data_call.encode();
    let factory_data_value_hex = hex::encode(factory_data_value.clone());

    let factory_data_value_hex_prefixed =
        format!("0x{}", factory_data_value_hex);

    println!(
        "Generated factory_data: {:?}",
        factory_data_value_hex_prefixed.clone()
    );

    // 5. Calculate the sender address

    let sender_address = get_sender_address_v07(
        &provider,
        simple_account_factory_address,
        factory_data_value.clone().into(),
        entry_point_address,
    )
    .await?;

    println!("Calculated sender address: {:?}", sender_address);

    let to: Address = transaction.to;
    let value: alloy::primitives::Uint<256, 4> = transaction.value;
    let data = transaction.data;

    let call_data = SimpleAccountExecute::new(to, value, data);
    let call_data_encoded = call_data.encode();
    let call_data_value_hex = hex::encode(call_data_encoded.clone());
    let call_data_value_hex_prefixed = format!("0x{}", call_data_value_hex);

    println!("Generated callData: {:?}", call_data_value_hex_prefixed);

    let is_deployed =
        is_smart_account_deployed(&provider, sender_address).await?;

    println!("is_deployed: {:?}", is_deployed);

    let factory: Option<Address> =
        if !is_deployed { Some(simple_account_factory_address) } else { None };

    let factory_data: Option<Bytes> =
        if !is_deployed { Some(factory_data_value.into()) } else { None };

    // Fill out remaining UserOperation values

    let gas_price = pimlico_client.estimate_user_operation_gas_price().await?;

    println!("Gas price: {:?}", gas_price);

    let nonce =
        get_nonce(&provider, sender_address, &entry_point_address).await?;

    let user_op = {
        let max_fee_per_gas = gas_price.fast.max_fee_per_gas;
        let max_priority_fee_per_gas = gas_price.fast.max_priority_fee_per_gas;
        PackedUserOperation {
            sender: sender_address,
            nonce,
            factory,
            factory_data,
            call_data: Bytes::from_str(&call_data_value_hex)?,
            call_gas_limit: U256::from(0),
            verification_gas_limit: U256::from(0),
            pre_verification_gas: U256::from(0),
            max_fee_per_gas,
            max_priority_fee_per_gas,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
            paymaster_data: None,
            signature: Bytes::from_str(
                signature::DUMMY_SIGNATURE_HEX.strip_prefix("0x").unwrap(),
            )?,
        }
    };

    let paymaster_client =
        PaymasterClient::new(BundlerConfig::new(paymaster_base_url.clone()));

    let sponsor_user_op_result = paymaster_client
        .sponsor_user_operation_v07(
            &user_op.clone().into(),
            &entry_point_address,
            None,
        )
        .await?;

    println!("sponsor_user_op_result: {:?}", sponsor_user_op_result);

    let sponsored_user_op = {
        let s = sponsor_user_op_result.clone();
        let mut op = user_op.clone();

        op.call_gas_limit = s.call_gas_limit;
        op.verification_gas_limit = s.verification_gas_limit;
        op.pre_verification_gas = s.pre_verification_gas;
        op.paymaster = Some(s.paymaster);
        op.paymaster_verification_gas_limit =
            Some(s.paymaster_verification_gas_limit);
        op.paymaster_post_op_gas_limit = Some(s.paymaster_post_op_gas_limit);
        op.paymaster_data = Some(s.paymaster_data);

        op
    };

    println!("Received paymaster sponsor result: {:?}", sponsored_user_op);

    // Sign the UserOperation

    // let signed_user_op = sign_user_operation_v07_with_ecdsa(
    //     &sponsored_user_op.clone(),
    //     &entry_point_address,
    //     chain_id.eip155_chain_id(),
    //     alloy_signer,
    // )?;

    // println!("Generated signature: {:?}", signed_user_op.signature);

    // let user_operation_hash = bundler_client
    //     .send_user_operation(
    //         entry_point_address.to_address(),
    //         signed_user_op.clone(),
    //     )
    //     .await?;

    // println!("Received User Operation hash: {:?}", user_operation_hash);

    // Ok(user_operation_hash)

    todo!()
}
