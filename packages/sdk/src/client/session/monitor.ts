/**
 * Session monitoring utilities for tracking session state changes
 *
 * These utilities help monitor session expiration and state changes,
 * similar to the legacy SDK's onSessionStateChange callback functionality.
 */

import type { Chain, Client, PublicActions, Transport } from "viem";

import type {
  GetSessionStateParams,
  SessionEventType,
  SessionState,
  SessionStateEventCallback,
} from "../actions/sessions.js";
import { getSessionState, SessionStatus } from "../actions/sessions.js";
import type { SessionSpec } from "../session/types.js";

/**
 * Parameters for starting session monitoring
 */
export type StartSessionMonitoringParams = {
  /**
   * Callback to invoke when session state changes
   */
  onSessionStateChange: SessionStateEventCallback;

  /**
   * Interval in milliseconds to check session state (default: 60000 = 1 minute)
   */
  checkIntervalMs?: number;

  /**
   * Threshold in seconds for warning about session expiration (default: 3600 = 1 hour)
   */
  expirationWarningThresholdSeconds?: number;

  /**
   * Threshold percentage for warning about fee limit (default: 80 = 80%)
   */
  feeLimitWarningThresholdPercent?: number;
};

/**
 * Session monitor handle for stopping monitoring
 */
export type SessionMonitor = {
  /**
   * Stop monitoring the session
   */
  stop: () => void;

  /**
   * Force an immediate check of session state
   */
  checkNow: () => Promise<void>;
};

/**
 * Start monitoring a session for state changes
 *
 * This function periodically checks the session state and invokes the callback
 * when important events occur (expiration, revocation, fee limit warnings, etc.).
 *
 * @param client - A viem public client with chain and transport
 * @param sessionParams - Parameters for querying session state
 * @param monitoringParams - Monitoring configuration
 * @returns A SessionMonitor handle to stop monitoring
 *
 * @example
 * ```typescript
 * const monitor = startSessionMonitoring(
 *   publicClient,
 *   {
 *     account: smartAccountAddress,
 *     sessionSpec: mySessionSpec,
 *     contracts: {
 *       sessionValidator: sessionValidatorAddress,
 *     },
 *   },
 *   {
 *     onSessionStateChange: (event) => {
 *       if (event.type === SessionEventType.Expired) {
 *         console.error("Session expired!");
 *       } else if (event.type === SessionEventType.Warning) {
 *         console.warn(event.message);
 *       }
 *     },
 *     checkIntervalMs: 30000, // Check every 30 seconds
 *   }
 * );
 *
 * // Later, stop monitoring
 * monitor.stop();
 * ```
 */
export function startSessionMonitoring<
  TTransport extends Transport = Transport,
  TChain extends Chain | undefined = Chain | undefined,
>(
  client: Client<TTransport, TChain> & PublicActions,
  sessionParams: GetSessionStateParams,
  monitoringParams: StartSessionMonitoringParams,
): SessionMonitor {
  const {
    onSessionStateChange,
    checkIntervalMs = 60000, // Default: 1 minute
    expirationWarningThresholdSeconds = 3600, // Default: 1 hour
    feeLimitWarningThresholdPercent = 80, // Default: 80%
  } = monitoringParams;

  let intervalHandle: ReturnType<typeof setInterval> | null = null;
  let expirationTimeoutHandle: ReturnType<typeof setTimeout> | null = null;
  let hasExpired = false;
  let hasRevoked = false;
  let lastFeeWarning = false;

  const checkSessionState = async () => {
    try {
      const { sessionState } = await getSessionState(client, sessionParams);
      const now = BigInt(Math.floor(Date.now() / 1000));

      // Check if session has been revoked/closed
      if (sessionState.status === SessionStatus.Closed && !hasRevoked) {
        hasRevoked = true;
        onSessionStateChange({
          type: "session_revoked" as SessionEventType,
          message: "Session has been revoked",
          sessionState,
        });
        return; // Don't continue monitoring after revocation
      }

      // Check if session is inactive
      if (sessionState.status === SessionStatus.NotInitialized) {
        onSessionStateChange({
          type: "session_inactive" as SessionEventType,
          message: "Session is not initialized",
          sessionState,
        });
        return; // Don't continue monitoring if not initialized
      }

      // Check if session has expired
      if (sessionParams.sessionSpec.expiresAt <= now && !hasExpired) {
        hasExpired = true;
        onSessionStateChange({
          type: "session_expired" as SessionEventType,
          message: "Session has expired",
          sessionState,
        });
        return; // Don't continue monitoring after expiration
      }

      // Check for expiration warning
      const timeToExpiry = sessionParams.sessionSpec.expiresAt - now;
      if (
        timeToExpiry > 0n
        && timeToExpiry <= BigInt(expirationWarningThresholdSeconds)
        && !hasExpired
      ) {
        const minutes = Number(timeToExpiry) / 60;
        onSessionStateChange({
          type: "session_warning" as SessionEventType,
          message: `Session will expire in ${minutes.toFixed(1)} minutes`,
          sessionState,
        });
      }

      // Check for fee limit warning
      if (
        sessionState.status === SessionStatus.Active
        && sessionParams.sessionSpec.feeLimit.limit > 0n
      ) {
        const feeUsedPercent = Number(
          (sessionParams.sessionSpec.feeLimit.limit
            - sessionState.feesRemaining)
            * 100n
            / sessionParams.sessionSpec.feeLimit.limit,
        );

        if (
          feeUsedPercent >= feeLimitWarningThresholdPercent
          && !lastFeeWarning
        ) {
          lastFeeWarning = true;
          onSessionStateChange({
            type: "session_warning" as SessionEventType,
            message: `Session fee limit is ${feeUsedPercent.toFixed(0)}% exhausted (${sessionState.feesRemaining} remaining)`,
            sessionState,
          });
        } else if (feeUsedPercent < feeLimitWarningThresholdPercent) {
          // Reset warning flag if usage drops below threshold
          lastFeeWarning = false;
        }
      }
    } catch (error) {
      console.error("Error checking session state:", error);
    }
  };

  // Set up expiration timeout
  const now = BigInt(Math.floor(Date.now() / 1000));
  const timeToExpiry = sessionParams.sessionSpec.expiresAt - now;
  if (timeToExpiry > 0n) {
    const timeoutMs = Number(timeToExpiry) * 1000;
    expirationTimeoutHandle = setTimeout(() => {
      if (!hasExpired) {
        hasExpired = true;
        void getSessionState(client, sessionParams).then(({ sessionState }) => {
          onSessionStateChange({
            type: "session_expired" as SessionEventType,
            message: "Session has expired",
            sessionState,
          });
        });
      }
    }, timeoutMs);
  }

  // Start periodic monitoring
  intervalHandle = setInterval(() => {
    void checkSessionState();
  }, checkIntervalMs);

  // Do an immediate check
  void checkSessionState();

  return {
    stop: () => {
      if (intervalHandle) {
        clearInterval(intervalHandle);
        intervalHandle = null;
      }
      if (expirationTimeoutHandle) {
        clearTimeout(expirationTimeoutHandle);
        expirationTimeoutHandle = null;
      }
    },
    checkNow: checkSessionState,
  };
}

/**
 * Check if a session should show a warning based on its state
 */
export function shouldWarnAboutSession(
  sessionSpec: SessionSpec,
  sessionState: SessionState,
  options: {
    expirationWarningThresholdSeconds?: number;
    feeLimitWarningThresholdPercent?: number;
  } = {},
): { shouldWarn: boolean; reason?: string } {
  const {
    expirationWarningThresholdSeconds = 3600,
    feeLimitWarningThresholdPercent = 80,
  } = options;

  const now = BigInt(Math.floor(Date.now() / 1000));

  // Check expiration
  const timeToExpiry = sessionSpec.expiresAt - now;
  if (timeToExpiry > 0n && timeToExpiry <= BigInt(expirationWarningThresholdSeconds)) {
    const minutes = Number(timeToExpiry) / 60;
    return {
      shouldWarn: true,
      reason: `Session will expire in ${minutes.toFixed(1)} minutes`,
    };
  }

  // Check fee limit
  if (
    sessionState.status === SessionStatus.Active
    && sessionSpec.feeLimit.limit > 0n
  ) {
    const feeUsedPercent = Number(
      (sessionSpec.feeLimit.limit - sessionState.feesRemaining)
      * 100n
      / sessionSpec.feeLimit.limit,
    );

    if (feeUsedPercent >= feeLimitWarningThresholdPercent) {
      return {
        shouldWarn: true,
        reason: `Fee limit is ${feeUsedPercent.toFixed(0)}% exhausted`,
      };
    }
  }

  return { shouldWarn: false };
}
