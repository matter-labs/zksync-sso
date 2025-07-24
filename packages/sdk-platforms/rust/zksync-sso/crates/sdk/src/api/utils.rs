use crate::{
    client::session::actions::session::send::{SignFn, sign_fn_from_signer},
    config::contracts::SSOContracts,
    utils::deployment_utils,
};
use alloy::{
    hex::FromHex,
    primitives::{Address, Bytes, FixedBytes},
    signers::local::PrivateKeySigner,
};
use alloy_zksync::network::unsigned_tx::eip712::PaymasterParams;
use eyre::eyre;
use std::str::FromStr;

pub fn parse_address(address: &str) -> eyre::Result<Address> {
    address.parse::<Address>().map_err(|e| eyre!("Invalid address: {}", e))
}

pub fn parse_paymaster_params(
    address: String,
    input_data: Option<String>,
) -> eyre::Result<PaymasterParams> {
    let paymaster = parse_address(&address)?;
    let paymaster_input: Bytes = input_data
        .map(|input| decode_hex(&input))
        .transpose()?
        .unwrap_or(Bytes::new());
    Ok(PaymasterParams { paymaster, paymaster_input })
}

pub fn decode_fixed_bytes_hex<const N: usize>(
    bytes: &str,
) -> eyre::Result<FixedBytes<N>> {
    FixedBytes::from_hex(bytes).map_err(|e| eyre!("Invalid fixed bytes: {}", e))
}

pub fn decode_hex<T: FromHex>(hex: &str) -> eyre::Result<T> {
    T::from_hex(hex).map_err(|_| eyre!("Invalid hex"))
}

/// Create a SignFn from a private key hex string  
///
/// # Arguments
/// * `private_key_hex` - Private key as a hex string
///
/// # Returns
/// A SignFn that can be used with session management functions
pub fn sign_fn_from_private_key_hex(
    private_key_hex: &str,
) -> eyre::Result<SignFn> {
    let signer = PrivateKeySigner::from_str(private_key_hex)?;
    Ok(sign_fn_from_signer(signer))
}

/// Get the address from a private key hex string
pub fn private_key_to_address(private_key_hex: &str) -> eyre::Result<Address> {
    let signer = PrivateKeySigner::from_str(private_key_hex)?;
    Ok(signer.address())
}

pub async fn deploy_contracts(
    node_url: url::Url,
) -> eyre::Result<SSOContracts> {
    let contracts = deployment_utils::deploy_contracts(node_url).await?;
    Ok(contracts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = "0x7e1da98cb433d253a6faf30895fa4a389ef6b182";
        assert!(parse_address(addr).is_ok());

        let invalid = "0x1234";
        assert!(parse_address(invalid).is_err());

        let invalid = "0xghijk";
        assert!(parse_address(invalid).is_err());
    }
}
