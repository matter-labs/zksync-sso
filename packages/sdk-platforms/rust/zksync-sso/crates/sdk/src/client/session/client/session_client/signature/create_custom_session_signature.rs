use crate::{
    client::session::{
        actions::session::sign::{
            CreateSessionTransactionSignatureParameters,
            create_session_transaction_signature,
        },
        client::session_client::timestamp::get_in_memory_node_timestamp,
    },
    config::Config,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::primitives::{Address, Bytes, ChainId, FixedBytes};
use log::debug;

#[derive(Debug, Clone)]
pub struct CreateCustomSessionSignatureParameters {
    pub chain: ChainId,
    pub session_key: FixedBytes<32>,
    pub config: Config,
    pub session_config: SessionSpec,
    pub hash: FixedBytes<32>,
    pub to: Address,
    pub call_data: Option<Bytes>,
}

pub async fn create_custom_session_signature(
    parameters: CreateCustomSessionSignatureParameters,
) -> eyre::Result<Bytes> {
    debug!("create_custom_session_signature");

    debug!("parameters: {parameters:?}");

    let timestamp = if is_in_memory_node(parameters.chain) {
        Some(get_in_memory_node_timestamp(parameters.config.clone()).await?)
    } else {
        None
    };

    debug!("timestamp: {timestamp:?}");

    let session_transaction_signature = create_session_transaction_signature(
        CreateSessionTransactionSignatureParameters {
            hash: parameters.hash,
            to: parameters.to,
            call_data: parameters.call_data,
            session_key: parameters.session_key,
            session_config: parameters.session_config,
            contracts: parameters.config.contracts,
            timestamp,
        },
    )?;

    debug!("session_transaction_signature: {session_transaction_signature:?}");

    Ok(session_transaction_signature)
}

fn is_in_memory_node(chain: ChainId) -> bool {
    chain == 260
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::contracts::SSOContracts,
        utils::session::session_lib::session_spec::{
            limit_type::LimitType, transfer_spec::TransferSpec,
            usage_limit::UsageLimit,
        },
    };
    use alloy::{
        hex,
        primitives::{U256, address, bytes, fixed_bytes},
    };

    #[tokio::test]
    async fn test_create_custom_session_signature() -> eyre::Result<()> {
        let chain = 260;
        let session_key = fixed_bytes!(
            "0x6954ddb21936036ccad688e2770846f15380a721bfab26c6e531e25b35cb5971"
        );
        let contracts = SSOContracts {
            account_factory: address!(
                "0x0000000000000000000000000000000000000000"
            ),
            passkey: address!("0x0000000000000000000000000000000000000000"),
            session: address!("0x027ba0517cfa4471457c6e74f201753d98e7431d"),
            account_paymaster: address!(
                "0x0000000000000000000000000000000000000000"
            ),
            recovery: address!("0x0000000000000000000000000000000000000000"),
        };
        let config = Config {
            contracts,
            node_url: "http://0.0.0.0:8011".parse().unwrap(),
            deploy_wallet: None,
        };
        let hash = fixed_bytes!(
            "0x438331d7eba6601df86b9ddc6b0ca3f5ec7ac0b395a3d7e2795fa2a855b4daad"
        );
        let to = address!("0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72");
        let call_data = None;
        let session_config = {
            let signer = address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");
            let expires_at = U256::from(1767225600u64);
            let fee_limit = UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64), // 0.1 ETH
                period: U256::from(0u64),
            };
            let call_policies = vec![];
            let transfer_policies = vec![TransferSpec {
                target: to,
                max_value_per_use: U256::from(10000000000000000u64), // 0.01 ETH
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0u64),
                    period: U256::from(0u64),
                },
            }];

            SessionSpec {
                signer,
                expires_at,
                fee_limit,
                call_policies,
                transfer_policies,
            }
        };

        let parameters = CreateCustomSessionSignatureParameters {
            chain,
            session_key,
            config,
            session_config,
            hash,
            to,
            call_data,
        };

        let custom_signature =
            create_custom_session_signature(parameters).await?;

        println!("custom_signature: {custom_signature:?}");

        let expected_custom_signature = bytes!(
            "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000027ba0517cfa4471457c6e74f201753d98e7431d00000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000419bdec54176e0c91080f44c066837875a3c9afda7095568f64951b8ed9b6f420d7f9e89aa5d0be73a8625c41267bc4a7d62ac3fde218b5392b56cee2c23237e271b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002000000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d972479000000000000000000000000000000000000000000000000000000006955b9000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000debbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        // Compare signatures byte by byte to find differences
        println!("\n=== SIGNATURE COMPARISON ===");
        println!("Custom signature length: {}", custom_signature.len());
        println!(
            "Expected signature length: {}",
            expected_custom_signature.len()
        );

        // Convert to hex strings for easier comparison
        let custom_hex = hex::encode(&custom_signature);
        let expected_hex = hex::encode(&expected_custom_signature);

        println!("\nCustom signature hex:\n{custom_hex}");
        println!("\nExpected signature hex:\n{expected_hex}");

        // Find differences
        println!("\n=== DIFFERENCES ===");
        let min_len = custom_hex.len().min(expected_hex.len());
        for i in (0..min_len).step_by(2) {
            let custom_byte = &custom_hex[i..i + 2];
            let expected_byte = &expected_hex[i..i + 2];
            if custom_byte != expected_byte {
                println!(
                    "Byte {} (offset 0x{:x}): custom={}, expected={}",
                    i / 2,
                    i / 2,
                    custom_byte,
                    expected_byte
                );
            }
        }

        eyre::ensure!(
            custom_signature == expected_custom_signature,
            "custom_signature does not match expected_custom_signature. \n\tcustom_signature: {:?}, \n\texpected_custom_signature: {:?}",
            custom_signature,
            expected_custom_signature,
        );

        Ok(())
    }
}
