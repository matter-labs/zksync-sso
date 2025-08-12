use crate::{
    api::account::Account, config::Config, contracts::WebAuthValidator,
    utils::passkey::passkey_signature_from_public_key::get_passkey_signature_from_public_key_bytes,
};
use alloy::primitives::{Address, Bytes, address};
use alloy_zksync::provider::zksync_provider;
use log::debug;

#[derive(Debug, Clone)]
pub struct FetchedAccount {
    pub address: Address,
    pub unique_account_id: String,
    pub passkey_public_key: Vec<u8>,
}

pub async fn fetch_account(
    unique_account_id: String,
    expected_origin: String,
    config: &Config,
) -> eyre::Result<FetchedAccount> {
    let origin = expected_origin;

    let username = unique_account_id.clone();

    let provider = zksync_provider()
        .with_recommended_fillers()
        .on_http(config.clone().node_url);

    let credential_id: Bytes = {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(username)?;
        Bytes::from(encoded)
    };

    debug!("XDB fetch_account - credential_id: {credential_id:?}");

    let validator = {
        let validator_address = config.contracts.passkey;
        WebAuthValidator::new(validator_address, provider.clone())
    };

    let account_address = validator
        .registeredAddress(origin.clone(), credential_id.clone())
        .call()
        .await?
        .accountAddress;

    eyre::ensure!(
        account_address != address!("0000000000000000000000000000000000000000")
    );

    debug!("XDB fetch_account - account_address: {account_address:?}");

    let passkey_public_key = {
        let public_key = validator
            .getAccountKey(
                origin.clone(),
                credential_id.clone(),
                account_address,
            )
            .call()
            .await?
            ._0;

        let lower_key_half = public_key[0];
        let upper_key_half = public_key[1];

        get_passkey_signature_from_public_key_bytes((
            *lower_key_half,
            *upper_key_half,
        ))?
    };

    Ok(FetchedAccount {
        address: account_address,
        unique_account_id,
        passkey_public_key,
    })
}

pub async fn get_account_by_user_id(
    user_id: String,
    config: &Config,
) -> eyre::Result<super::Account> {
    crate::client::passkey::account_factory::get_smart_account_address_by_user_id(
        user_id.clone(),
        config,
    )
    .await
    .map_err(|e| eyre::eyre!(e))
    .map(|address| Account {
        address,
        unique_account_id: user_id,
    })
}
