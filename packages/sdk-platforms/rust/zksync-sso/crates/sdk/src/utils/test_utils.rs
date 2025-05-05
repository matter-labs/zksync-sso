use crate::{
    config::{Config, deploy_wallet::DeployWallet},
    utils::deployment_utils::deploy_contracts,
};
use ::alloy::{
    providers::{
        Identity, RootProvider,
        fillers::{FillProvider, JoinFill, WalletFiller},
    },
    signers::{Signer, local::LocalSigner},
};
use alloy_zksync::{
    network::Zksync,
    node_bindings::{AnvilZKsync, AnvilZKsyncError, AnvilZKsyncInstance},
    provider::layers::anvil_zksync::AnvilZKsyncProvider,
    provider::{ProviderBuilderExt as _, zksync_provider},
    wallet::ZksyncWallet,
};
use k256::{Secp256k1, elliptic_curve::SecretKey};
use tokio::task;

pub fn zksync_wallet_from_anvil_zksync(
    anvil_zksync: &AnvilZKsyncInstance,
) -> eyre::Result<(ZksyncWallet, SecretKey<Secp256k1>, Vec<SecretKey<Secp256k1>>)>
{
    let default_keys: Vec<SecretKey<Secp256k1>> = anvil_zksync.keys().to_vec();
    let (default_key, remaining_keys) =
        default_keys.split_first().ok_or(AnvilZKsyncError::NoKeysAvailable)?;

    let default_signer = LocalSigner::from(default_key.clone())
        .with_chain_id(Some(anvil_zksync.chain_id()));
    let mut wallet = ZksyncWallet::from(default_signer);

    for key in remaining_keys {
        let signer = LocalSigner::from(key.clone());
        wallet.register_signer(signer)
    }

    Ok((wallet, default_key.clone(), remaining_keys.to_vec()))
}

pub async fn spawn_node() -> eyre::Result<(
    AnvilZKsyncInstance,
    FillProvider<
        JoinFill<Identity, WalletFiller<ZksyncWallet>>,
        AnvilZKsyncProvider<RootProvider<Zksync>>,
        Zksync,
    >,
    DeployWallet,
    url::Url,
)> {
    use alloy_zksync::provider::layers::anvil_zksync::AnvilZKsyncLayer;
    let anvil_zksync = AnvilZKsync::new().try_spawn()?;
    let node_url = anvil_zksync.endpoint_url();

    let (provider, private_key_hex) = {
        let f = |anvil_zksync: AnvilZKsync| anvil_zksync;
        let anvil_zksync_layer = AnvilZKsyncLayer::from(f(Default::default()));

        let (wallet, default_key, _) =
            zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let provider = zksync_provider()
            .wallet(wallet)
            .layer(anvil_zksync_layer)
            .on_http(node_url.clone());

        let private_key_hex = hex::encode(default_key.to_bytes());

        (provider, private_key_hex)
    };

    let deploy_wallet = DeployWallet::try_from(private_key_hex)?;

    Ok((anvil_zksync, provider, deploy_wallet, node_url))
}

pub async fn spawn_node_and_deploy_contracts() -> eyre::Result<(
    AnvilZKsyncInstance,
    Config,
    FillProvider<
        JoinFill<Identity, WalletFiller<ZksyncWallet>>,
        AnvilZKsyncProvider<RootProvider<Zksync>>,
        Zksync,
    >,
)> {
    let (anvil_zksync, provider, deploy_wallet, node_url) =
        spawn_node().await?;

    let contracts = deploy_contracts(node_url.clone()).await?;

    let config = Config { contracts, node_url, deploy_wallet };

    Ok((anvil_zksync, config, provider))
}

#[tokio::test]
#[ignore = "Disable this test, it is run as part of other tests"]
async fn test_spawn_node_and_deploy_contracts() -> eyre::Result<()> {
    let (anvil_zksync, config, _) = spawn_node_and_deploy_contracts().await?;

    println!("config: {:?}", config);

    drop(anvil_zksync);

    Ok(())
}

#[tokio::test]
#[ignore = "Disable this test for now"]
async fn test_parallel_contract_deployments() -> eyre::Result<()> {
    let num_deployments = 3;
    let mut handles = Vec::new();

    for _ in 0..num_deployments {
        let handle = task::spawn(async move {
            let (anvil_zksync, _, _) =
                spawn_node_and_deploy_contracts().await?;
            Ok::<_, eyre::Error>(anvil_zksync)
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await??;
        drop(result);
    }

    Ok(())
}
