# Q&A

A demo of using the ZKsync SDK for smart accounts.

## Getting Started

Run the following (be sure a local node is running, e.g.
`era_test_node`[https://docs.zksync.io/build/zksync-cli/running-a-node]):

```sh
# Deploy the ZKsync SSO contracts
pnpm nx deploy contracts

# Deploy the contracts for the Q&A Demo
pnpm nx deploy:local q-and-a-contracts
```

The contract addresses for the Q&A app will be set in `.env.local`. This
.env file will override the values set in the `runtimeConfig` in
`nuxt.config.ts`.

You may also need to update the contract addresses for the Auth Server in
`/packages/auth-server/stores/client.ts` under the
`contractsByChain[zksyncInMemoryNode.id]`

```sh
# Start the website and Auth Server
pnpm nx dev q-and-a
```

## Running e2e tests

Run the tests locally with:

```sh
pnpm nx e2e q-and-a
```

and you can enable debug mode with:

```sh
pnpm nx e2e:debug q-and-a
```

## Deploy the NFT Quest app to Firebase (WIP)

The command to deploy for testnet:

```sh
pnpm nx deploy q-and-a
```

The command to deploy to a preview channel:

```sh
pnpm nx deploy:preview q-and-a <CHANNEL_NAME>
```
