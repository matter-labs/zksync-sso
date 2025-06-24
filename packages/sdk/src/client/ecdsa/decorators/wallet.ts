import { type Account, bytesToHex, type Chain, type ExactPartial, formatTransaction, type RpcTransaction, type Transport, type WalletActions } from "viem";
import { deployContract, getAddresses, getCallsStatus, getCapabilities, getChainId, prepareAuthorization, sendCalls, sendRawTransaction, showCallsStatus, signAuthorization, signMessage, signTransaction, signTypedData, waitForCallsStatus, writeContract } from "viem/actions";
import { type TransactionRequestEIP712, type ZksyncEip712Meta } from "viem/zksync";

import { getTransactionWithPaymasterData } from "../../../paymaster/index.js";
import { sendEip712Transaction } from "../../session/actions/sendEip712Transaction.js";
import type { ClientWithZksyncSsoEcdsaData } from "../client.js";

export type ZksyncSsoEcdsaWalletActions<chain extends Chain, account extends Account> = Omit<
  WalletActions<chain, account>, "addChain" | "getPermissions" | "requestAddresses" | "requestPermissions" | "switchChain" | "watchAsset" | "prepareTransactionRequest"
>;

export function zksyncSsoEcdsaWalletActions<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: ClientWithZksyncSsoEcdsaData<transport, chain>): ZksyncSsoEcdsaWalletActions<chain, account> {
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

      /* eslint-disable @typescript-eslint/no-unused-vars */
      const { chainId: _, ...unformattedTxWithPaymaster } = await getTransactionWithPaymasterData(
        client.chain.id,
        client.account.address,
        unformattedTx,
        client.paymasterHandler,
      );

      const formatters = client.chain?.formatters;
      const format = formatters?.transaction?.format || formatTransaction;

      const tx = {
        ...format(unformattedTxWithPaymaster as ExactPartial<RpcTransaction>),
        type: "eip712",
      };

      return await sendEip712Transaction(client, tx);
    },
    signMessage: (args) => signMessage(client, args),

    signTransaction: async (args) => {
      /* eslint-disable @typescript-eslint/no-unused-vars */
      const { chainId: _, ...unformattedTxWithPaymaster } = await getTransactionWithPaymasterData(
        client.chain.id,
        client.account.address,
        args as TransactionRequestEIP712,
        client.paymasterHandler,
      );
      return signTransaction(client, {
        ...args,
        unformattedTxWithPaymaster,
      });
    },
    signTypedData: (args) => signTypedData(client, args),
    writeContract: (args) => writeContract(client, args),
    signAuthorization: (args) => signAuthorization(client, args),
    getCallsStatus: (args) => getCallsStatus(client, args),
    getCapabilities: (args) => getCapabilities(client, args),
    prepareAuthorization: (args) => prepareAuthorization(client, args),
    sendCalls: (args) => sendCalls(client, args),
    showCallsStatus: (args) => showCallsStatus(client, args),
    waitForCallsStatus: (args) => waitForCallsStatus(client, args),
  };
}
