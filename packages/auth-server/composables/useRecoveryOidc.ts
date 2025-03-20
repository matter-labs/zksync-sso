import { type Address, encodeAbiParameters, encodeFunctionData, type Hex, keccak256 } from "viem";
import { OidcKeyRegistryAbi, OidcRecoveryModuleAbi } from "zksync-sso/abi";
import { type Groth16Proof, type JWT, JwtTxValidationInputs, OidcDigest } from "zksync-sso-circuits";

import type { OidcData } from "../../sdk/dist/_types/client/recovery/actions/oidc";

export const useRecoveryOidc = () => {
  const { getClient, getPublicClient, defaultChain } = useClientStore();
  const { snarkjs } = useSnarkJs();
  const {
    public: { saltServiceUrl },
  } = useRuntimeConfig();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  const getOidcAccountsInProgress = ref(false);
  const getOidcAccountsError = ref<Error | null>(null);
  const getOidcAccountsData = ref<readonly OidcData[] | null>(null);

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
      getOidcAccountsData.value = data;
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

  // TODO: improve this
  type KeyStruct = { issHash: Hex; kid: Hex };

  function recoveryStep1Calldata(proof: Groth16Proof, key: KeyStruct, passkey: [Hex, Hex], targetAccount: Address): Hex {
    const passkeyHash = keccak256(
      encodeAbiParameters([{ type: "bytes32" }, { type: "bytes32" }], passkey),
    );

    return encodeFunctionData({
      abi: OidcRecoveryModuleAbi,
      functionName: "startRecovery",
      args: [
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
          issHash: key.issHash,
          kid: key.kid,
          pendingPasskeyHash: passkeyHash,
        },
        targetAccount,
      ],
    });
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

    const groth16Result = await snarkjs.groth16.fullProve(
      inputs.toObject(),
      "/circuit/witness.wasm",
      "/circuit/circuit.zkey",
      console,
    );

    return groth16Result.proof;
  });

  async function hashIssuer(): Hex {
    const client = await getPublicClient({ chainId: defaultChain.id });
    return client.readContract({
      address: contractsByChain[defaultChain.id].oidcKeyRegistry,
      abi: OidcKeyRegistryAbi,
      functionName: "hashIssuer",
      args: ["https://accounts.google.com"],
    });
  }

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
    hashIssuer,
  };
};
