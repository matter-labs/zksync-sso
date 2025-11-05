import { type Account, type Chain, type Transport, type WalletActions } from "viem";
import { deployContract, getAddresses, getCallsStatus, getCapabilities, getChainId, prepareAuthorization, sendCalls, sendRawTransaction, sendTransaction, showCallsStatus, signAuthorization, signMessage, signTransaction as signTransactionAction, signTypedData, waitForCallsStatus, writeContract } from "viem/actions";

import type { ClientWithZksyncSsoRecoveryData } from "../client.js";

export type ZksyncSsoWalletActions<chain extends Chain, account extends Account> = Omit<
  WalletActions<chain, account>, "addChain" | "getPermissions" | "requestAddresses" | "requestPermissions" | "switchChain" | "watchAsset" | "prepareTransactionRequest"
>;

export function zksyncSsoWalletActions<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: ClientWithZksyncSsoRecoveryData<transport, chain, account>): ZksyncSsoWalletActions<chain, account> {
  return {
    deployContract: (args) => deployContract(client, args),
    getAddresses: () => getAddresses(client),
    getChainId: () => getChainId(client),
    sendRawTransaction: (args) => sendRawTransaction(client, args),
    sendTransaction: (args) => sendTransaction(client, args),
    signMessage: (args) => signMessage(client, args),
    signTransaction: (args) => signTransactionAction(client, args),
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
