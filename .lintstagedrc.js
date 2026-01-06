export default {
  "*.{js,ts,vue}": (filenames) => {
    const filtered = filenames.filter((f) => !f.includes("passkey-wallet-app/main.js"));
    return filtered.length > 0 ? `eslint --no-ignore ${filtered.join(" ")}` : [];
  },
  "*.md": ["markdownlint-cli2"],
  "*.{json,yml}": ["prettier --list-different"],
  "*.sol": ["prettier --list-different"],
};
