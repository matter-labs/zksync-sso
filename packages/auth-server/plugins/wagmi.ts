import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { WagmiPlugin } from "@wagmi/vue";

const queryClient = new QueryClient();

export default defineNuxtPlugin((nuxtApp) => {
  const { wagmiConfig } = useAppKit();

  nuxtApp.vueApp
    .use(WagmiPlugin, { config: wagmiConfig })
    .use(VueQueryPlugin, { queryClient });
});
