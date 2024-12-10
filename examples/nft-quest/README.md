# NFT Quest

A demo of using the ZKsync SDK for smart accounts.

## Getting Started

Run the following (be sure a local node is running, e.g.
`era_test_node`[https://docs.zksync.io/build/zksync-cli/running-a-node]):

```sh
# Deploy the ZKsync SSO contracts
pnpm nx deploy contracts

# Deploy the contracts for the NFT Quest Demo
pnpm nx deploy:local nft-quest-contracts
```

Contract addresses are defined in the `/packages/auth-server/abi` directory. For
local development using In Memory Node, edit the contracts' `addressByChain` for
`260`.

```sh
# Start the website and Auth Server
pnpm nx dev nft-quest
```

## Running e2e tests

Run the tests locally with:

```sh
pnpm nx e2e nft-quest
```

and you can enable debug mode with:

```sh
pnpm nx e2e:debug nft-quest
```

## Deploy the NFT Quest app to Firebase (WIP)

The command to deploy for testnet:

```sh
pnpm nx deploy nft-quest
```

The command to deploy to a preview channel:

```sh
pnpm nx deploy:preview nft-quest <CHANNEL_NAME>
```
