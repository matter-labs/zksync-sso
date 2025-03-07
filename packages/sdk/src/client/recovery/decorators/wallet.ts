import { type Account, bytesToHex, type Chain, formatTransaction, type Transport, type WalletActions } from "viem";
import { deployContract, getAddresses, getChainId, sendRawTransaction, signMessage, signTypedData, writeContract } from "viem/actions";
import { signTransaction, type ZksyncEip712Meta } from "viem/zksync";

import { sendEip712Transaction } from "../actions/sendEip712Transaction.js";
import type { ClientWithZksyncSsoRecoveryData } from "../client.js";

export type ZKsyncSsoWalletActions<chain extends Chain, account extends Account> = Omit<
  WalletActions<chain, account>, "addChain" | "getPermissions" | "requestAddresses" | "requestPermissions" | "switchChain" | "watchAsset" | "prepareTransactionRequest"
>;

export function zksyncSsoWalletActions<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: ClientWithZksyncSsoRecoveryData<transport, chain, account>): ZKsyncSsoWalletActions<chain, account> {
  return {
    deployContract: (args) => deployContract(client, args),
    getAddresses: () => getAddresses(client),
    getChainId: () => getChainId(client),
    sendRawTransaction: (args) => sendRawTransaction(client, args),
    sendTransaction: async (args) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const unformattedTx: any = Object.assign({}, args);

      if ("eip712Meta" in unformattedTx) {
        const eip712Meta = unformattedTx.eip712Meta as ZksyncEip712Meta;
        unformattedTx.gasPerPubdata = eip712Meta.gasPerPubdata ? BigInt(eip712Meta.gasPerPubdata) : undefined;
        unformattedTx.factoryDeps = eip712Meta.factoryDeps;
        unformattedTx.customSignature = eip712Meta.customSignature;
        unformattedTx.paymaster = eip712Meta.paymasterParams?.paymaster;
        unformattedTx.paymasterInput = eip712Meta.paymasterParams?.paymasterInput ? bytesToHex(new Uint8Array(eip712Meta.paymasterParams?.paymasterInput)) : undefined;
        delete unformattedTx.eip712Meta;
      }

      const formatters = client.chain?.formatters;
      const format = formatters?.transaction?.format || formatTransaction;

      const tx = {
        ...format(unformattedTx),
        type: "eip712",
      };

      return await sendEip712Transaction(client, tx);
    },
    signMessage: (args) => signMessage(client, args),
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    signTransaction: (args) => signTransaction(client, args as any),
    signTypedData: (args) => signTypedData(client, args),
    writeContract: (args) => writeContract(client, args),
  };
}
