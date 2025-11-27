// TODO: Figure out why this causes error:
// `TypeError: Cannot set properties of undefined (setting 'type')`

// import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
// import { WagmiPlugin } from "@wagmi/vue";

// const queryClient = new QueryClient();

export default defineNuxtPlugin((_nuxtApp) => {
  // const { wagmiConfig } = useAppKit();

  // nuxtApp.vueApp
  //   .use(WagmiPlugin, { config: wagmiConfig })
  //   .use(VueQueryPlugin, { queryClient });
});
