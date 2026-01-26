# NFT Quest

A demo of using the ZKsync SSO 4337 SDK for smart accounts with ERC-4337.

## Getting Started

### Prerequisites

The NFT Quest demo requires:

1. Anvil running on port 8545 (standard EVM node)
2. Alto bundler running on port 4337
3. ERC-4337 MSA (Modular Smart Account) factory and infrastructure deployed
4. NFT contracts deployed to Anvil

### Local Development

Run the following commands from the repository root:

```sh
# 1. Start Anvil (in a separate terminal)
cd packages/erc4337-contracts
pnpm run anvil

# 2. Deploy ERC-4337 MSA factory and infrastructure (in another terminal)
pnpm nx deploy-msa-factory demo-app

# 3. Start Alto bundler with CORS proxy (in another terminal)
cd packages/erc4337-contracts
pnpm run bundler:with-proxy

# 4. Deploy NFT Quest contracts
# This reads demo-app's contracts-anvil.json for shared infrastructure
pnpm nx deploy:local nft-quest-contracts

# 5. Start the website and Auth Server
pnpm nx dev nft-quest
```

The contract addresses will be written to:

- `.env.local` - For Nuxt runtime config
- `public/contracts.json` - For runtime access

The deployment script reuses the MSA infrastructure (session validator, webauthn
validator, mock paymaster) deployed by demo-app.

> **Note**: The current paymaster is MockPaymaster which allows all
> transactions. TODO: Implement a restricted paymaster that only sponsors NFT
> minting transactions.

## Running e2e tests

Run the tests locally with:

```sh
pnpm nx e2e nft-quest
```

Enable debug mode with:

```sh
pnpm nx e2e:debug nft-quest
```

## Production Deployment

> **TODO**: Deploy NFT Quest contracts to ZKsync OS Testnet and update contract
> addresses in `nuxt.config.ts`

The command to deploy to testnet (after contracts are deployed):

```sh
pnpm nx deploy nft-quest
```

Deploy to a preview channel:

```sh
pnpm nx deploy:preview nft-quest <CHANNEL_NAME>
```
