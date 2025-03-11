import type { CircuitSignals, Groth16Proof, PublicSignals } from "zksync-sso-circuits";

type Logger = typeof console;

type SnarkJs = {
  groth16: {
    fullProve: (input: CircuitSignals, wasmPath: string, zkeyPath: string, logger?: Logger) => Promise<{
      proof: Groth16Proof;
      publicSignals: PublicSignals;
    }>;
  };
};

declare const snarkjs: SnarkJs;

export const useSnarkJs = () => {
  return { snarkjs };
};
