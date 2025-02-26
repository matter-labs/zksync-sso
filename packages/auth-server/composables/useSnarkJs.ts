import type { CircuitSignals, Groth16Proof, PublicSignals } from "zksync-sso-circuits";

type SnarkJs = {
  groth16: {
    fullProve: (input: CircuitSignals, wasmPath: string, zkeyPath: string) => Promise<{
      proof: Groth16Proof;
      publicSignals: PublicSignals;
    }>;
  };
};

declare global {
  const snarkjs: SnarkJs;
}

export const useSnarkJs = () => {
  return { snarkJs: snarkjs };
};
