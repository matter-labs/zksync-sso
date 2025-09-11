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
    entry_point::{
        PackedUserOperation, nonce::get_nonce,
        sender_address::get_sender_address_v07,
    },
    paymaster::pimlico::{
        client::PaymasterClient,
        models::v08::UserOperationPreSponsorship as UserOperationPreSponsorshipV08,
    },
    transaction::Transaction,
    user_operation::signature::v08::sign_user_operation,
};
use alloy::{
    hex,
    primitives::{Address, Bytes, FixedBytes, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use std::str::FromStr;

pub mod config;
pub mod create;
pub mod ensure;
pub mod execute;

pub fn deploy_account() -> eyre::Result<Address> {
    todo!()
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

    let pimlico_client: PimlicoBundlerClient =
        PimlicoBundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

    let entry_point_address = Address::ZERO; // TODO: use the correct entrypoint address

    let alloy_signer = signer.clone();
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

    let gas_price = pimlico_client.estimate_user_operation_gas_price().await?;

    println!("Gas price: {:?}", gas_price);

    let nonce =
        get_nonce(&provider, sender_address, &entry_point_address).await?;

    let user_op = {
        let max_fee_per_gas = gas_price.fast.max_fee_per_gas;
        let max_priority_fee_per_gas = gas_price.fast.max_priority_fee_per_gas;
        // Create initCode by concatenating factory address and factory_data
        let init_code = if let (Some(factory), Some(factory_data)) =
            (factory, factory_data)
        {
            let mut code = factory.to_vec();
            code.extend_from_slice(&factory_data);
            Bytes::from(code)
        } else {
            Bytes::default()
        };

        // Pack gas limits into accountGasLimits (verification_gas_limit | call_gas_limit)
        let account_gas_limits_bytes = [0u8; 32];
        // For now, set to zeros - will be updated after estimation
        let account_gas_limits =
            FixedBytes::<32>::from(account_gas_limits_bytes);

        // Pack gas fees (max_priority_fee_per_gas | max_fee_per_gas)
        let mut gas_fees_bytes = [0u8; 32];
        let priority_bytes = max_priority_fee_per_gas.to_be_bytes_vec();
        let fee_bytes = max_fee_per_gas.to_be_bytes_vec();
        gas_fees_bytes[0..16].copy_from_slice(&priority_bytes[16..]);
        gas_fees_bytes[16..32].copy_from_slice(&fee_bytes[16..]);
        let gas_fees = FixedBytes::<32>::from(gas_fees_bytes);

        PackedUserOperation {
            sender: sender_address,
            nonce,
            initCode: init_code,
            callData: Bytes::from_str(&call_data_value_hex)?,
            accountGasLimits: account_gas_limits,
            preVerificationGas: U256::from(0),
            gasFees: gas_fees,
            paymasterAndData: Bytes::default(),
            signature: Bytes::from_str(
                signature::DUMMY_SIGNATURE_HEX.strip_prefix("0x").unwrap(),
            )?,
        }
    };

    let paymaster_client =
        PaymasterClient::new(BundlerConfig::new(paymaster_base_url.clone()));

    let user_op_v08 = UserOperationPreSponsorshipV08::from(user_op.clone());

    let sponsor_user_op_result = paymaster_client
        .sponsor_user_operation_v08(&user_op_v08, &entry_point_address, None)
        .await?;

    println!("sponsor_user_op_result: {:?}", sponsor_user_op_result);

    let sponsored_user_op = {
        let s = sponsor_user_op_result.clone();
        let mut op = user_op.clone();

        // Update accountGasLimits with new values
        let verification_gas = s.verification_gas_limit.to_be_bytes_vec();
        let call_gas = s.call_gas_limit.to_be_bytes_vec();
        let mut account_gas_limits_bytes = [0u8; 32];
        account_gas_limits_bytes[0..16]
            .copy_from_slice(&verification_gas[16..]);
        account_gas_limits_bytes[16..32].copy_from_slice(&call_gas[16..]);
        op.accountGasLimits = FixedBytes::<32>::from(account_gas_limits_bytes);

        op.preVerificationGas = s.pre_verification_gas;

        // Pack paymaster data
        let mut paymaster_and_data = s.paymaster.to_vec();
        // Add verification gas limit (16 bytes)
        paymaster_and_data.extend_from_slice(
            &s.paymaster_verification_gas_limit.to_be_bytes_vec()[16..],
        );
        // Add post op gas limit (16 bytes)
        paymaster_and_data.extend_from_slice(
            &s.paymaster_post_op_gas_limit.to_be_bytes_vec()[16..],
        );
        // Add paymaster data
        paymaster_and_data.extend_from_slice(&s.paymaster_data);
        op.paymasterAndData = Bytes::from(paymaster_and_data);

        op
    };

    println!("Received paymaster sponsor result: {:?}", sponsored_user_op);

    // Sign the UserOperation
    sign_user_operation(
        &sponsored_user_op.clone(),
        &signer,
        chain_id,
        entry_point_address,
    )?;
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

    todo!("finish implementing the deploy function")
}
