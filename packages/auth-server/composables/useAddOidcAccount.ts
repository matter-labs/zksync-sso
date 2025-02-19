export const useAddOidcAccount = () => {
  const { inProgress: isLoading, error, execute: addOidcAccount } = useAsync(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1500));
  });

  return {
    addOidcAccount,
    isLoading,
    error,
  };
};
