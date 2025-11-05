import {
  type Account,
  type Chain,
  type Transport, type WalletActions } from "viem";
import {
  deployContract, getAddresses, getCallsStatus, getCapabilities, getChainId, prepareAuthorization, sendCalls, sendRawTransaction,
  sendTransaction, showCallsStatus, signAuthorization, signMessage, signTransaction as signTransactionAction, signTypedData, waitForCallsStatus, writeContract,
} from "viem/actions";

import { SessionErrorType, SessionEventType, type SessionState, validateSessionTransaction } from "../../../utils/session.js";
import { getSessionState, sessionStateNotify } from "../actions/session.js";
import type { ClientWithZksyncSsoSessionData } from "../client.js";

export type ZksyncSsoWalletActions<chain extends Chain, account extends Account> = Omit<
  WalletActions<chain, account>, "addChain" | "getPermissions" | "requestAddresses" | "requestPermissions" | "switchChain" | "watchAsset" | "prepareTransactionRequest"
>;

const sessionErrorToSessionEventType = {
  [SessionErrorType.SessionInactive]: SessionEventType.Inactive,
  [SessionErrorType.SessionExpired]: SessionEventType.Expired,
};

/**
 * Helper function to check session state and notify via callback
 */
async function getSessionStateAndNotify<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: ClientWithZksyncSsoSessionData<transport, chain, account>): Promise<SessionState> {
  const { sessionState } = await getSessionState(client, {
    account: client.account.address,
    sessionConfig: client.sessionConfig,
    contracts: client.contracts,
  });

  if (client.onSessionStateChange) {
    sessionStateNotify({
      sessionConfig: client.sessionConfig,
      sessionState,
      onSessionStateChange: client.onSessionStateChange,
      sessionNotifyTimeout: client._sessionNotifyTimeout,
    });
  }

  return sessionState;
}

export function zksyncSsoWalletActions<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: ClientWithZksyncSsoSessionData<transport, chain, account>): ZksyncSsoWalletActions<chain, account> {
  return {
    deployContract: (args) => deployContract(client, args),
    getAddresses: () => getAddresses(client),
    getChainId: () => getChainId(client),
    sendRawTransaction: (args) => sendRawTransaction(client, args),
    sendTransaction: async (args) => {
      if (client.skipPreTransactionStateValidation !== true) {
        // Get current session state and trigger callback if needed
        const sessionState = await getSessionStateAndNotify(client);

        // Validate transaction against session constraints
        const validationResult = validateSessionTransaction({
          sessionState,
          sessionConfig: client.sessionConfig,
          transaction: args as any,
        });

        // Throw error if validation fails
        if (validationResult.error) {
          // If validation fails due to session issues, notify via callback
          if (client.onSessionStateChange && Object.keys(sessionErrorToSessionEventType).includes(validationResult.error.type)) {
            client.onSessionStateChange({
              type: sessionErrorToSessionEventType[validationResult.error.type as keyof typeof sessionErrorToSessionEventType],
              message: validationResult.error.message,
            });
          }
          throw new Error(`Session validation failed: ${validationResult.error.message} (${validationResult.error.type})`);
        }
      }

      return sendTransaction(client, args);
    },
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
