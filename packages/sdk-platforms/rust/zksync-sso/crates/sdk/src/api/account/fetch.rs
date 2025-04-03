use crate::{
    api::account::Account, config::Config,
    utils::passkey::passkey_signature_from_public_key::get_passkey_signature_from_public_key_bytes,
};
use alloy::primitives::Address;
use alloy_zksync::provider::zksync_provider;

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

    let address = {
        let factory_address = config.contracts.account_factory;
        let factory = crate::client::contracts::aa_factory::AAFactory::new(
            factory_address,
            provider.clone(),
        );
        factory.accountMappings(username.clone()).call().await?._0
    };

    let passkey_public_key = {
        let passkey =
            crate::client::contracts::webauthvalidator::WebAuthValidator::new(
                config.contracts.passkey,
                provider.clone(),
            );

        let lower_key_half =
            passkey.lowerKeyHalf(origin.clone(), address).call().await?._0;

        let upper_key_half =
            passkey.upperKeyHalf(origin.clone(), address).call().await?._0;
        get_passkey_signature_from_public_key_bytes((
            *lower_key_half,
            *upper_key_half,
        ))?
    };

    Ok(FetchedAccount { address, unique_account_id, passkey_public_key })
}

pub async fn get_account_by_user_id(
    user_id: String,
    secret_account_salt: String,
    config: &Config,
) -> eyre::Result<super::Account> {
    crate::client::passkey::account_factory::get_smart_account_address_by_user_id(
        &user_id, &secret_account_salt, config,
    )
    .await
    .map_err(|e| eyre::eyre!(e))
    .map(|address| Account {
        address,
        unique_account_id: user_id,
    })
}
