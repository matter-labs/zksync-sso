import { type Address, encodeAbiParameters, encodeFunctionData, type Hex, keccak256 } from "viem";
import { OidcKeyRegistryAbi, OidcRecoveryModuleAbi } from "zksync-sso/abi";
import type { OidcData } from "zksync-sso/client/oidc";
import { type Groth16Proof, type JWT, JwtTxValidationInputs, OidcDigest } from "zksync-sso-circuits";

export const useRecoveryOidc = () => {
  const { getClient, getPublicClient, defaultChain } = useClientStore();
  const { snarkjs } = useSnarkJs();
  const {
    public: { saltServiceUrl },
  } = useRuntimeConfig();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

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

  const {
    execute: getOidcAccounts,
    inProgress: getOidcAccountsInProgress,
    result: googleAccountData,
    error: getOidcAccountsError,
  } = useAsync(async (oidcAddress: Address) => {
    const client = getPublicClient({ chainId: defaultChain.id });
    try {
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].recoveryOidc,
        abi: OidcRecoveryModuleAbi,
        functionName: "oidcDataForAddress",
        args: [oidcAddress],
      });
      return data as OidcData;
    } catch (error) {
      console.warn(error);
      return undefined;
    }
  });

  const oidcAccounts = computed<OidcData[]>(() => {
    return googleAccountData.value == null ? [] : [googleAccountData];
  });

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
  } = useAsync(async (rawJwt: string, n: string, salt: Hex, valueInNonce: Hex, blindingFactor: bigint) => {
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

  async function hashIssuer(): Promise<Hex> {
    const client = await getPublicClient({ chainId: defaultChain.id });
    const res = await client.readContract({
      address: contractsByChain[defaultChain.id].oidcKeyRegistry,
      abi: OidcKeyRegistryAbi,
      functionName: "hashIssuer",
      args: ["https://accounts.google.com"],
    });
    return res as Hex;
  }

  return {
    getOidcAccounts,
    getOidcAccountsInProgress,
    getOidcAccountsError,
    googleAccountData,
    oidcAccounts,
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
