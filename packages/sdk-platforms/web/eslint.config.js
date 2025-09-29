import { defineConfig } from "eslint-define-config";

export default defineConfig({
  extends: ["../../../eslint.config.js"],
  ignorePatterns: ["dist/", "pkg-bundler/", "pkg-node/", "node_modules/"],
  rules: {
    // Allow any types for WASM bindings since they're dynamically loaded
    "@typescript-eslint/no-explicit-any": "off",
    // Allow unused vars for WASM declarations that may not be used immediately
    "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],

  },
});
