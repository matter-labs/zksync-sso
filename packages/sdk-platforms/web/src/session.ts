/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  add_session_to_account as wasm_add_session_to_account,
  deploy_account as wasm_deploy_account,
  send_transaction_session as wasm_send_transaction_session,
  SessionPayload as WasmSessionPayload,
  TransferPayload as WasmTransferPayload,
} from "./bundler";
import type { LimitType, SessionConfig, TransferSpec } from "./types";

function limitTypeToCode(t: LimitType | undefined): number {
  switch (t) {
    case "unlimited":
      return 0;
    case "allowance":
      return 2;
    case "lifetime":
    default:
      return 1;
  }
}

function toDecString(v: bigint | number): string {
  if (typeof v === "bigint") return v.toString(10);
  if (Number.isInteger(v)) return String(v);
  // Fallback: floor decimals (caller should pass wei as bigint ideally)
  return String(Math.trunc(v));
}

function buildTransfers(specs: TransferSpec[]): any[] {
  return specs.map((t) =>
    new (WasmTransferPayload as any)(
      t.to,
      toDecString(t.valueLimit),
      limitTypeToCode(t.limitType),
      String(t.period ?? 0),
    ),
  );
}

export function toSessionPayload(spec: SessionConfig): any {
  const transfers = buildTransfers(spec.transfers);
  const feeLimitType = limitTypeToCode(spec.feeLimit?.limitType);
  const feeLimitValue = toDecString(spec.feeLimit.limit);
  const feeLimitPeriod = String(spec.feeLimit.period ?? 0);
  const expiresAtStr = typeof spec.expiresAt === "bigint"
    ? spec.expiresAt.toString(10)
    : String(spec.expiresAt);

  return new (WasmSessionPayload as any)(
    spec.signer,
    expiresAtStr,
    feeLimitValue,
    feeLimitType,
    feeLimitPeriod,
    transfers,
  );
}

// Convenience wrapper: deploy account with optional session
export async function deployAccountWithSession(args: {
  userId: string;
  eoaSigners?: string[] | null;
  passkeyPayload?: any | null;
  session?: SessionConfig | null;
  deployConfig: any; // wasm DeployAccountConfig instance
}): Promise<any> {
  const sessionPayload = args.session ? toSessionPayload(args.session) : null;
  return wasm_deploy_account(
    args.userId,
    args.eoaSigners ?? null,
    args.passkeyPayload ?? null,
    sessionPayload,
    args.deployConfig,
  );
}

// Convenience wrapper: add session to existing account
export async function addSessionToAccount(args: {
  txConfig: any; // wasm SendTransactionConfig
  accountAddress: string;
  session: SessionConfig;
  sessionValidatorAddress: string;
  eoaValidatorAddress: string;
  eoaPrivateKey: string;
}): Promise<any> {
  const sessionPayload = toSessionPayload(args.session);
  return wasm_add_session_to_account(
    args.txConfig,
    args.accountAddress,
    sessionPayload,
    args.sessionValidatorAddress,
    args.eoaValidatorAddress,
    args.eoaPrivateKey,
  );
}

// Convenience wrapper: send transaction using session key
export async function sendTransactionWithSession(args: {
  txConfig: any; // wasm SendTransactionConfig
  sessionValidatorAddress: string;
  accountAddress: string;
  to: string;
  value: string;
  data?: string | null;
  sessionPrivateKey: string;
  session: SessionConfig;
}): Promise<any> {
  const sessionPayload = toSessionPayload(args.session);
  return wasm_send_transaction_session(
    args.txConfig,
    args.sessionValidatorAddress,
    args.accountAddress,
    args.to,
    args.value,
    args.data ?? null,
    args.sessionPrivateKey,
    sessionPayload,
  );
}
