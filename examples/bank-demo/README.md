# Bank demo

A Bank ZKsync demo illustrating how to create a ZKsync smart account, log in
with a Passkey, and stake some ETH with a fully embedded wallet.

## Running the demo locally

From the packages/contracts directory, deploy the contracts to a local node:

```bash
pnpm run deploy --file ../../examples/bank-demo/local-node.json
```

Run the following command from the root of the monorepo:

```bash
pnpm nx dev bank-demo
```

_Note: You will need `era_test_node` running with the latest contracts deployed_

## "Resetting" the demo

Account session and data is stored via the browser Local storage.

1. When you need to restart the demo, click the `Accounts` tab, click the
   ellipses (`...`), and click `Reset Demo`.

2. You should also delete the Passkey stored for the app. In the Chrome browser,
   navigate to `chrome://settings/passkeys`. Click the settings button for the
   entry for `localhost` and click "Delete".

## Deploying the Bank demo to Firebase

The Bank demo app uses Demo Node (`https://node.nvillanueva.com`). Add your
deployer private key to the .env file (packages/contracts/.env)

1. Deploy the latest contracts with

   `pnpm --dir packages/contracts run deploy --network demoNode`.

2. Update `nuxt.config.ts` contract addresses under `$production`.

3. Build the project with `pnpm nx build bank-demo`.

4. Deploy the project to Firebase.

   ```bash
   pnpm nx deploy bank-demo
   ```
