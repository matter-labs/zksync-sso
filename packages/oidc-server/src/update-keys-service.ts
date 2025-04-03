import { ContractUpdater } from "./contractUpdater.js";
import { env } from "./env.js";
import { GoogleFetcher } from "./fetchers/google.js";

const main = async () => {
  const fetcher = new GoogleFetcher();
  const contractUpdater = new ContractUpdater();

  try {
    const keys = await fetcher.fetchKeys();
    await contractUpdater.updateContract("https://accounts.google.com", keys);
  } catch (error) {
    console.error("Error fetching keys:", error);
  }

  setTimeout(main, env.FETCH_INTERVAL);
};

main();
