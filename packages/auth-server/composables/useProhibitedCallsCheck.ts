import type { Address } from "viem";

export const useProhibitedCallsCheck = () => {
  const { contracts } = useClientStore();
  const { address } = storeToRefs(useAccountStore());

  const checkTargetAddress = (target: Address): boolean => {
    const hasCallToAccountAddress = address.value === target;
    const ssoSystemAddresses = Object.values(contracts || {});
    const hasCallToSystemAddress = ssoSystemAddresses.includes(target);
    return hasCallToAccountAddress || hasCallToSystemAddress;
  };

  return {
    checkTargetAddress,
  };
};
