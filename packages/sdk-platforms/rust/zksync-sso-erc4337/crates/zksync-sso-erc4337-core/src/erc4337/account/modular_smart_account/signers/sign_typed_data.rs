use crate::erc4337::signer::SignatureProvider;
use alloy::{
    dyn_abi::TypedData,
    primitives::{Address, Bytes},
    providers::Provider,
    sol,
};

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    EIP712,
    "../../../../../../packages/erc4337-contracts/out/EIP712.sol/EIP712.json"
);

pub async fn sign_typed_data<P>(
    message_types: serde_json::Map<String, serde_json::Value>,
    message: serde_json::Value,
    primary_type: String,
    account: Address,
    contract_address: Address,
    provider: P,
    sign: SignatureProvider,
) -> eyre::Result<Bytes>
where
    P: Provider + Send + Sync + Clone,
{
    let caller_domain: EIP712::eip712DomainReturn = {
        let contract_instance = EIP712::new(contract_address, provider.clone());
        contract_instance.eip712Domain().call().await?
    };

    let account_domain: EIP712::eip712DomainReturn = {
        let account_instance = EIP712::new(account, provider.clone());
        account_instance.eip712Domain().call().await?
    };

    sign_typed_data_with_domains(
        message_types,
        message,
        primary_type,
        account_domain,
        caller_domain,
        sign,
    )
}

fn sign_typed_data_with_domains(
    message_types: serde_json::Map<String, serde_json::Value>,
    message: serde_json::Value,
    primary_type: String,
    account_domain: EIP712::eip712DomainReturn,
    caller_domain: EIP712::eip712DomainReturn,
    sign: SignatureProvider,
) -> eyre::Result<Bytes> {
    let inner_typed_data_json = serde_json::json!({
        "message": message,
        "types": message_types,
        "primaryType": primary_type,
        "domain": {}
    });

    let inner_typed_data: TypedData =
        serde_json::from_value(inner_typed_data_json)?;
    let inner_typed_data_struct_hash = inner_typed_data.hash_struct()?;

    let typed_data_sign_value = serde_json::json!({
        "contents": message,
        "name": account_domain.name,
        "version": account_domain.version,
        "chainId": account_domain.chainId,
        "verifyingContract": account_domain.verifyingContract,
        "salt": account_domain.salt,
    });

    let mut types_json = message_types.clone();
    types_json.insert(
        "TypedDataSign".to_string(),
        serde_json::json!([{
            "name": "contents",
            "type": primary_type
        },
        {
            "name": "name",
            "type": "string"
        },
        {
            "name": "version",
            "type": "string"
        },
        {
            "name": "chainId",
            "type": "uint256"
        },
        {
            "name": "verifyingContract",
            "type": "address"
        },
        {
            "name": "salt",
            "type": "bytes32"
        }]),
    );
    types_json.insert(
        "EIP712Domain".to_string(),
        serde_json::json!([
        {
            "name": "name",
            "type": "string"
        },
        {
            "name": "version",
            "type": "string"
        },
        {
            "name": "chainId",
            "type": "uint256"
        },
        {
            "name": "verifyingContract",
            "type": "address"
        }]),
    );

    let final_typed_data_json = serde_json::json!({
        "types": types_json,
        "primaryType": "TypedDataSign",
        "domain": {
            "name": caller_domain.name,
            "version": caller_domain.version,
            "chainId": caller_domain.chainId,
            "verifyingContract": caller_domain.verifyingContract,
        },
        "message": typed_data_sign_value,
    });

    let typed_data: TypedData = serde_json::from_value(final_typed_data_json)?;

    let final_hash = typed_data.eip712_signing_hash()?;

    let original_signature = sign(final_hash)?;

    let final_signature: Vec<u8> = {
        let domain_seperator = typed_data.domain().separator();

        let content_type = inner_typed_data.clone().encode_type()?;

        let content_len_bytes = {
            let content_len = content_type.len() as u16;
            content_len.to_be_bytes().to_vec()
        };

        [
            original_signature.to_vec(),
            domain_seperator.to_vec(),
            inner_typed_data_struct_hash.to_vec(),
            content_type.as_bytes().to_vec(),
            content_len_bytes,
        ]
        .concat()
    };

    Ok(final_signature.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::modular_smart_account::signers::eoa::eoa_signature;
    use alloy::primitives::{FixedBytes, U256, address, bytes};
    use std::sync::Arc;

    #[test]
    fn test_sign_typed_data() -> eyre::Result<()> {
        let eoa_validator_address =
            address!("0xF62849F9A0B5Bf2913b396098F7c7019b51A820a");
        let sign = {
            let signer_private_key: String = "0x02016836a56b71f0d02689e69e326f4f4c1b9057164ef592671cf0d37c8040c0".to_string();
            Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            })
        };

        let contract_address =
            address!("0xA4AD4f68d0b91CFD19687c881e50f3A00242828c");

        let caller_domain = EIP712::eip712DomainReturn {
            name: "ERC1271Caller".to_string(),
            version: "1.0.0".to_string(),
            fields: FixedBytes::default(),
            chainId: U256::from(31337),
            verifyingContract: contract_address,
            extensions: Vec::new(),
            salt: FixedBytes::default(),
        };

        let account_domain = EIP712::eip712DomainReturn {
            name: "zksync-sso-1271".to_string(),
            version: "1.0.0".to_string(),
            fields: FixedBytes::default(),
            chainId: U256::from(31337),
            verifyingContract: address!(
                "0xe8BF1ec183070262A223eB0dCb233FC51768E031"
            ),
            extensions: Vec::new(),
            salt: FixedBytes::default(),
        };

        let mut message_types = serde_json::Map::new();
        message_types.insert(
            "MockMessage".to_string(),
            serde_json::json!([
              { "type": "string", "name": "message" },
              { "type": "uint256", "name": "value"  }
            ]),
        );

        let message = serde_json::json!({
            "message": "Hello, world!",
            "value": 42
        });

        let primary_type = "MockMessage".to_string();

        let signature = sign_typed_data_with_domains(
            message_types,
            message,
            primary_type,
            account_domain,
            caller_domain,
            sign,
        )?;

        let expected_signature = bytes!(
            "0xf62849f9a0b5bf2913b396098f7c7019b51a820a3348056b01d55830973942a1a8c01359ecb6aef2e1d08cfa1b470d835a310aae204c66c0dcee0e53c4c37a2e6906ad3d36ea8188588ae6448a151a829ac74e491baa335e38a793ec8c04d12c9496bc1ee26e40779883a74d6dd1781f4b7a537b968f3e8c20dbe17618cf006de0a778efbb171a55529fb1c077086c27d407c8e5504d6f636b4d65737361676528737472696e67206d6573736167652c75696e743235362076616c7565290029"
        );

        eyre::ensure!(
            signature == expected_signature,
            "Signature mismatch, received: {signature:?}, expected: {expected_signature:?}"
        );

        Ok(())
    }
}
