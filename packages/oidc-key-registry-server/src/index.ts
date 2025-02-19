import { ContractUpdater } from "./contractUpdater";
import { env } from "./env.ts";
import { GoogleFetcher } from "./fetchers/google";

const main = async () => {
  const fetcher = new GoogleFetcher();
  const contractUpdater = new ContractUpdater();

  try {
    const keys = await fetcher.fetchKeys();
    await contractUpdater.updateContract("accounts.google.com", keys);
  } catch (error) {
    console.error("Error fetching keys:", error);
  }

  setTimeout(main, env.FETCH_INTERVAL);
};

main();
