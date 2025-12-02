import { startRegistration } from "@simplewebauthn/browser";
import type { Address, Hex } from "viem";
import { concat, decodeErrorResult, encodeFunctionData } from "viem";
import { entryPoint07Address } from "viem/account-abstraction";
import { prepareDeploySmartAccount } from "zksync-sso-4337/client";

import contractsConfig from "../contracts-anvil.json";
import { useAccountStore } from "./account";
import { useClientStore } from "./client";

export const useConnectorStore = defineStore("connector", () => {
  const accountStore = useAccountStore();
  const { address, isConnected } = storeToRefs(accountStore);

  const connectAccount = async () => {
    try {
      // Generate random challenge as base64url string
      const challengeBytes = crypto.getRandomValues(new Uint8Array(32));
      const challengeBase64 = btoa(String.fromCharCode(...challengeBytes))
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=/g, "");

      // Generate random user ID
      const userIdBytes = crypto.getRandomValues(new Uint8Array(16));
      const userIdBase64 = btoa(String.fromCharCode(...userIdBytes))
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=/g, "");

      // Start passkey registration
      const registrationResponse = await startRegistration({
        rp: {
          name: "NFT Quest",
          id: window.location.hostname,
        },
        user: {
          id: userIdBase64,
          name: "nft-quest-user",
          displayName: "NFT Quest User",
        },
        challenge: challengeBase64,
        pubKeyCredParams: [
          { alg: -7, type: "public-key" },
          { alg: -257, type: "public-key" },
        ],
        authenticatorSelection: {
          residentKey: "preferred",
          userVerification: "preferred",
        },
      });

      // Extract COSE key & credentialId from authenticatorData
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const authenticatorDataBase64url = (registrationResponse.response as any).authenticatorData;
      if (!authenticatorDataBase64url) {
        throw new Error("No authenticatorData in registration response");
      }
      const authenticatorDataBinary = atob(authenticatorDataBase64url.replace(/-/g, "+").replace(/_/g, "/"));
      const authenticatorData = new Uint8Array(authenticatorDataBinary.length);
      for (let i = 0; i < authenticatorDataBinary.length; i++) {
        authenticatorData[i] = authenticatorDataBinary.charCodeAt(i);
      }
      // Offsets per WebAuthn spec
      const credIdLengthOffset = 32 + 1 + 4 + 16; // RP hash + flags + counter + AAGUID
      const credIdLength = (authenticatorData[credIdLengthOffset] << 8) | authenticatorData[credIdLengthOffset + 1];
      const credIdOffset = credIdLengthOffset + 2;
      const coseKeyOffset = credIdOffset + credIdLength;
      const coseKey = authenticatorData.slice(coseKeyOffset);
      // Derive credentialId from authenticatorData (correct hex bytes)
      const rawCredId = authenticatorData.slice(credIdOffset, credIdOffset + credIdLength);
      const credIdHex = `0x${Array.from(rawCredId).map((b) => b.toString(16).padStart(2, "0")).join("")}` as Hex;

      // Minimal CBOR parser for COSE key (expecting map of small ints)
      function decodeBytes(buf: Uint8Array, off: number): [Uint8Array, number] {
        const b = buf[off];
        if (b >= 0x40 && b <= 0x57) {
          const len = b - 0x40;
          return [buf.slice(off + 1, off + 1 + len), len + 1];
        } else if (b === 0x58) {
          const len = buf[off + 1];
          return [buf.slice(off + 2, off + 2 + len), len + 2];
        }
        throw new Error("Unsupported byte string format");
      }
      function decodeInt(buf: Uint8Array, off: number): [number, number] {
        const b = buf[off];
        if (b < 24) return [b, 1];
        if (b === 0x18) return [buf[off + 1], 2];
        if (b === 0x19) return [(buf[off + 1] << 8) | buf[off + 2], 3];
        if (b >= 0x20 && b <= 0x37) return [-(b - 0x20) - 1, 1];
        if (b === 0x38) return [-1 - buf[off + 1], 2];
        if (b === 0x39) return [-1 - ((buf[off + 1] << 8) | buf[off + 2]), 3];
        throw new Error("Unsupported integer format");
      }
      function decodeValue(buf: Uint8Array, off: number): [number | Uint8Array, number] {
        const t = buf[off];
        if (t >= 0x40 && t <= 0x5F) return decodeBytes(buf, off);
        return decodeInt(buf, off);
      }
      function decodeMap(buf: Uint8Array): Map<number, Uint8Array | number> {
        const header = buf[0];
        if (header < 0xA0 || header > 0xB7) throw new Error("Unsupported map header");
        const pairs = header - 0xA0;
        let offset = 1;
        const map = new Map<number, Uint8Array | number>();
        for (let i = 0; i < pairs; i++) {
          const [key, keyLen] = decodeInt(buf, offset);
          offset += keyLen;
          const [val, valLen] = decodeValue(buf, offset);
          offset += valLen;
          map.set(key as number, val);
        }
        return map;
      }
      const coseMap = decodeMap(coseKey);
      const xBytes = coseMap.get(-2) as Uint8Array;
      const yBytes = coseMap.get(-3) as Uint8Array;
      if (!xBytes || !yBytes) throw new Error("Missing x/y in COSE key");
      const toHex = (arr: Uint8Array) => `0x${Array.from(arr).map((b) => b.toString(16).padStart(2, "0")).join("")}` as Hex;
      const x = toHex(xBytes);
      const y = toHex(yBytes);
      const clientStore = useClientStore();
      const publicClient = clientStore.getPublicClient();

      const { transaction } = prepareDeploySmartAccount({
        contracts: {
          factory: contractsConfig.factory as Address,
          webauthnValidator: contractsConfig.webauthnValidator as Address,
        },
        passkeySigners: [{
          credentialId: credIdHex,
          publicKey: { x, y },
          originDomain: window.location.origin,
        }],
      });

      const initCode = concat([transaction.to, transaction.data]);
      const entryPoint = entryPoint07Address;

      let senderAddress: Address;
      try {
        await publicClient.call({
          to: entryPoint,
          data: encodeFunctionData({
            abi: [{
              name: "getSenderAddress",
              type: "function",
              stateMutability: "view",
              inputs: [{ name: "initCode", type: "bytes" }],
              outputs: [],
            }],
            functionName: "getSenderAddress",
            args: [initCode],
          }),
        });
        throw new Error("getSenderAddress did not revert");
      } catch (e) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const data = (e as any).data || (e as any).cause?.data || (e as any).cause?.cause?.data;
        if (!data) throw e;

        const error = decodeErrorResult({
          abi: [{
            name: "SenderAddressResult",
            type: "error",
            inputs: [{ name: "sender", type: "address" }],
          }],
          data,
        });
        senderAddress = error.args[0];
      }

      accountStore.setAccount(senderAddress, credIdHex);
      return { address: senderAddress, credentialId: credIdHex };
    } catch (error) {
      // eslint-disable-next-line no-console
      console.error("Failed to connect account:", error);
      throw error;
    }
  };

  const disconnectAccount = () => {
    accountStore.clearAccount();
  };

  const shortAddress = computed(() => {
    if (!address.value) return null;
    return useTruncateAddress(address.value);
  });

  const { subscribe: address$, notify: notifyOnAccountChange } = useObservable<Address | undefined>();
  watch(address, (newAddress) => {
    notifyOnAccountChange(newAddress);
  });

  return {
    isConnected,
    address,
    connectAccount,
    disconnectAccount,
    address$,
    shortAddress,
  };
});
