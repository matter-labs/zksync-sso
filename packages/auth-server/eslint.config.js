// @ts-check
import simpleImportSort from "eslint-plugin-simple-import-sort";

import withNuxt from "./.nuxt/eslint.config.mjs";

export default withNuxt(
  {
    rules: {
      "no-console": "warn",
      semi: ["error", "always"], // Require semicolons
      quotes: ["error", "double"], // Require double quotes
      "vue/multi-word-component-names": "off", // Allow multi-word component names
      "vue/require-default-prop": "off", // Allow props without default values
      // Prevent Node.js Buffer usage in browser code
      "no-restricted-globals": [
        "error",
        {
          name: "Buffer",
          message: "Use browser-compatible alternatives instead of Node.js Buffer in client-side code",
        },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: "CallExpression[callee.object.name='Buffer']",
          message: "Use browser-compatible alternatives instead of Node.js Buffer in client-side code",
        },
        {
          selector: "NewExpression[callee.name='Buffer']",
          message: "Use browser-compatible alternatives instead of Node.js Buffer in client-side code",
        },
      ],
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
);
