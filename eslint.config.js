import pluginJs from "@eslint/js";
import stylistic from "@stylistic/eslint-plugin";
import { defineConfig, globalIgnores } from "eslint/config";
import simpleImportSort from "eslint-plugin-simple-import-sort";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores([
    "**/node_modules/",
    "**/dist/",
    "**/temp/",
    "**/tmp/",
    "**/.nuxt/",
    "**/.nx/",
    "**/.output/",
    "**/artifacts-zk/",
    "**/deployments-zk/",
    "**/cache-zk/",
    "**/typechain-types/",
    "**/react-native-zksync-sso/",
    "examples/",
    "packages/",
  ], "Ignore global"),
  { files: ["**/*.{js,mjs,cjs,ts}"] },
  stylistic.configs.customize({
    globalIgnores: ["packages/", "examples/"],
    indent: 2,
    quotes: "double",
    semi: true,
    arrowParens: "always",
    quoteProps: "as-needed",
    braceStyle: "1tbs",
  }),
  { languageOptions: { globals: { ...globals.browser, ...globals.node } } },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": ["warn"],
    },
  },
  {
    plugins: {
      "simple-import-sort": simpleImportSort,
    },
    rules: {
      "simple-import-sort/imports": "error",
      "simple-import-sort/exports": "error",
      "sort-imports": "off",
    },
  },
]);
