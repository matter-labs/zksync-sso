import { GOOGLE_ISS } from "./constants.js";
import { ContractUpdater } from "./contractUpdater.js";
import { env } from "./env.js";
import { GoogleFetcher } from "./fetchers/google.js";

const main = async () => {
  const fetcher = new GoogleFetcher();
  const contractUpdater = new ContractUpdater(env.KEY_REGISTRY_ADDRESS, env.RPC_URL, env.ADMIN_PRIVATE_KEY, env.NETWORK);

  try {
    const keys = await fetcher.fetchKeys();
    await contractUpdater.updateContract(GOOGLE_ISS, keys);
  } catch (error) {
    console.error("Error fetching keys:", error);
  }

  setTimeout(main, env.FETCH_INTERVAL);
};

main();
