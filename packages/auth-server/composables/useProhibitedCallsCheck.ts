import type { Address } from "viem";

export const useProhibitedCallsCheck = (
  _chainId: MaybeRef<SupportedChainId>,
) => {
  const chainId = toRef(_chainId);
  const { address } = storeToRefs(useAccountStore());

  const checkTargetAddress = (target: Address): boolean => {
    const hasCallToAccountAddress = address.value === target;
    const ssoSystemAddresses = Object.values(contractsByChain[chainId.value] || {});
    const hasCallToSystemAddress = ssoSystemAddresses.includes(target);
    return hasCallToAccountAddress || hasCallToSystemAddress;
  };

  return {
    checkTargetAddress,
  };
};
