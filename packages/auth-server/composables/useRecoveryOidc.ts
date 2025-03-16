import { type Address, encodeAbiParameters, type Hex } from "viem";
import { OidcRecoveryModuleAbi } from "zksync-sso/abi";
import {
  type BigintTuple,
  type OidcData,
  type OidcKey,
  type ParsedOidcData,
  parseOidcData,
} from "zksync-sso/client/oidc";
import { type Groth16Proof, type JWT, JwtTxValidationInputs, OidcDigest } from "zksync-sso-circuits";

export const useRecoveryOidc = () => {
  const { getClient, getPublicClient, defaultChain } = useClientStore();
  const { snarkjs } = useSnarkJs();
  const {
    public: { saltServiceUrl },
  } = useRuntimeConfig();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  const getOidcAccountsInProgress = ref(false);
  const getOidcAccountsError = ref<Error | null>(null);
  const getOidcAccountsData = ref<readonly ParsedOidcData[] | null>(null);

  async function buildOidcDigest(jwt: JWT): Promise<OidcDigest> {
    const response = await fetch(saltServiceUrl, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${jwt.raw}`,
      },
    })
      .then((res) => res.json());

    const salt = response.salt;
    return new OidcDigest(jwt.iss, jwt.aud, jwt.sub, salt);
  }

  async function getOidcAccounts(oidcAddress: Address) {
    getOidcAccountsInProgress.value = true;
    getOidcAccountsError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].recoveryOidc,
        abi: OidcRecoveryModuleAbi,
        functionName: "oidcDataForAddress",
        args: [oidcAddress],
      });
      getOidcAccountsData.value = data.map(parseOidcData);
      return;
    } catch (err) {
      getOidcAccountsError.value = err as Error;
      return;
    } finally {
      getOidcAccountsInProgress.value = false;
    }
  }

  const {
    inProgress: addOidcAccountIsLoading,
    error: addOidcAccountError,
    execute: addOidcAccount,
  } = useAsync(async (oidcData: OidcData) => {
    const client = getClient({ chainId: defaultChain.id });

    return await client.addOidcAccount({
      paymaster: {
        address: paymasterAddress,
      },
      oidcData,
    });
  });

  function recoveryStep1Calldata(proof: Groth16Proof, key: OidcKey): Hex {
    return encodeAbiParameters(
      [
        {
          type: "tuple",
          components: [
            {
              name: "zkProof",
              type: "tuple",
              components: [
                { name: "pA", type: "uint256[2]" },
                { name: "pB", type: "uint256[2][2]" },
                { name: "pC", type: "uint256[2]" },
              ],
            },
            {
              name: "key",
              type: "tuple",
              components: [
                { name: "issHash", type: "bytes32" },
                { name: "kid", type: "bytes32" },
                { name: "n", type: "uint256[17]" },
                { name: "e", type: "bytes" },
              ],
            },
          ],
        },
      ],
      [
        {
          zkProof: {
            pA: [BigInt(proof.pi_a[0]), BigInt(proof.pi_a[1])],
            pB: [
              // The verifier expects these parameters in this order.
              // It's easier to perform this inversion here than in solidity.
              [BigInt(proof.pi_b[0][1]), BigInt(proof.pi_b[0][0])],
              [BigInt(proof.pi_b[1][1]), BigInt(proof.pi_b[1][0])],
            ],
            pC: [BigInt(proof.pi_c[0]), BigInt(proof.pi_c[1])],
          },
          key: {
            issHash: key.issHash,
            kid: key.kid,
            n: key.n as BigintTuple<17>,
            e: key.e,
          },
        },
      ]);
  }

  const {
    inProgress: zkProofInProgress,
    execute: generateZkProof,
    result: zkProof,
    error: zkProofError,
  } = useAsync(async (rawJwt: string, n: string, salt: Hex, valueInNonce: string, blindingFactor: bigint) => {
    const inputs = new JwtTxValidationInputs(
      rawJwt,
      n,
      salt,
      valueInNonce,
      blindingFactor,
    );

    const res = await snarkjs.groth16.fullProve(
      inputs.toObject(),
      "/circuit/witness.wasm",
      "/circuit/circuit.zkey",
      console,
    );

    return res;
  });

  return {
    getOidcAccounts,
    getOidcAccountsInProgress,
    getOidcAccountsError,
    getOidcAccountsData,
    buildOidcDigest,
    addOidcAccount,
    addOidcAccountIsLoading,
    addOidcAccountError,
    recoveryStep1Calldata,
    zkProofInProgress,
    generateZkProof,
    zkProof,
    zkProofError,
  };
};
