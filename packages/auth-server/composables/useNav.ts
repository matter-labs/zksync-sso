export function useNav() {
  const mainNav = [
    {
      href: "/dashboard",
      name: "Dashboard",
      icon: "dashboard",
    },
    {
      href: "/dashboard/settings",
      name: "Settings",
      icon: "settings",
    },
    {
      href: "/dashboard/sessions",
      name: "Sessions",
      icon: "link",
    },
    {
      href: "/dashboard/wallet-connect",
      name: "WalletConnect",
      icon: "share",
    },
  ];

  return { mainNav };
}
