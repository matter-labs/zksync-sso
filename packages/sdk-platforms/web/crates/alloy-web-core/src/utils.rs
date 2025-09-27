use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rlp::{Decodable, Encodable};
use alloy::rpc::types::BlockNumberOrTag;
use alloy::sol_types::SolValue;

// Helper function to create errors
pub fn create_error(s: &str) -> String {
    s.to_string()
}

pub fn encode_rlp(data: Vec<u8>) -> Result<String, String> {
    let mut buf = Vec::new();
    data.encode(&mut buf);
    Ok(alloy::primitives::hex::encode(buf))
}

pub fn decode_rlp(hex_data: &str) -> Result<Vec<u8>, String> {
    let bytes =
        alloy::primitives::hex::decode(hex_data).map_err(|e| create_error(&e.to_string()))?;
    Vec::<u8>::decode(&mut bytes.as_slice()).map_err(|e| create_error(&e.to_string()))
}

pub fn create_address(address: &str) -> Result<String, String> {
    let addr: Address = address
        .parse()
        .map_err(|e| create_error(&format!("Invalid address: {e:?}")))?;
    Ok(format!(
        "0x{}",
        alloy::primitives::hex::encode(addr.as_slice())
    ))
}

pub fn parse_u256(value: &str) -> Result<String, String> {
    let u256_value: U256 = value
        .parse()
        .map_err(|e: alloy::primitives::ruint::ParseError| create_error(&e.to_string()))?;
    Ok(u256_value.to_string())
}

pub fn encode_function_data(signature: &str, params: Vec<String>) -> Result<String, String> {
    let selector = alloy::primitives::keccak256(signature.as_bytes());
    let selector_bytes = &selector[..4];

    let mut encoded = Vec::from(selector_bytes);

    for param in params {
        if let Some(stripped) = param.strip_prefix("0x") {
            let bytes = alloy::primitives::hex::decode(stripped)
                .map_err(|e| create_error(&e.to_string()))?;
            if bytes.len() == 20 {
                let addr = Address::from_slice(&bytes);
                encoded.extend_from_slice(&addr.abi_encode());
            } else if bytes.len() == 32 {
                encoded.extend_from_slice(&bytes);
            } else {
                let bytes_val = Bytes::from(bytes);
                encoded.extend_from_slice(&bytes_val.abi_encode());
            }
        } else if let Ok(num) = param.parse::<U256>() {
            encoded.extend_from_slice(&num.abi_encode());
        } else {
            return Err(create_error(&format!(
                "Unable to encode parameter: {param}"
            )));
        }
    }

    Ok(format!("0x{}", alloy::primitives::hex::encode(encoded)))
}

pub fn get_block_tag(tag: &str) -> Result<BlockNumberOrTag, String> {
    let block_tag = match tag {
        "latest" => BlockNumberOrTag::Latest,
        "earliest" => BlockNumberOrTag::Earliest,
        "pending" => BlockNumberOrTag::Pending,
        "safe" => BlockNumberOrTag::Safe,
        "finalized" => BlockNumberOrTag::Finalized,
        _ => {
            if let Ok(num) = tag.parse::<u64>() {
                BlockNumberOrTag::Number(num)
            } else {
                return Err(create_error("Invalid block tag"));
            }
        }
    };

    Ok(block_tag)
}

/// Check if a contract is deployed at the specified address
pub async fn is_contract_deployed(rpc_url: &str, address: &str) -> Result<bool, String> {
    let provider = ProviderBuilder::new()
        .connect(rpc_url)
        .await
        .map_err(|e| create_error(&format!("Failed to connect to provider: {e}")))?;

    let addr: Address = address
        .parse()
        .map_err(|e| create_error(&format!("Invalid address: {e:?}")))?;

    let code = provider
        .get_code_at(addr)
        .await
        .map_err(|e| create_error(&format!("Failed to get code at address: {e}")))?;

    Ok(!code.is_empty())
}
