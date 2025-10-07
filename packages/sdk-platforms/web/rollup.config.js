import resolve from "@rollup/plugin-node-resolve";
import typescript from "@rollup/plugin-typescript";
import { defineConfig } from "rollup";
import dts from "rollup-plugin-dts";

const external = [
  "fs",
  "path",
  "crypto",
  "stream",
  "util",
  "events",
  "buffer",
  "string_decoder",
  "url",
  "querystring",
  "http",
  "https",
  "zlib",
  "net",
  "tls",
  "os",
  // WASM files
  /\.wasm$/,
  // Generated WASM modules
  /pkg-(bundler|node)\/.*$/,
];

export default defineConfig([
  // ESM build
  {
    input: "src/index.ts",
    output: {
      file: "dist/index.js",
      format: "esm",
      sourcemap: true,
    },
    external,
    plugins: [
      resolve({ preferBuiltins: true }),
      typescript({
        tsconfig: "./tsconfig.json",
        declaration: false,
        declarationMap: false,
      }),
    ],
  },
  // CommonJS build
  {
    input: "src/index.ts",
    output: {
      file: "dist/index.cjs",
      format: "cjs",
      sourcemap: true,
    },
    external,
    plugins: [
      resolve({ preferBuiltins: true }),
      typescript({
        tsconfig: "./tsconfig.json",
        declaration: false,
        declarationMap: false,
      }),
    ],
  },
  // Bundler-specific build
  {
    input: "src/bundler.ts",
    output: {
      file: "dist/bundler.js",
      format: "esm",
      sourcemap: true,
    },
    external,
    plugins: [
      resolve({ preferBuiltins: false, browser: true }),
      typescript({
        tsconfig: "./tsconfig.json",
        declaration: false,
        declarationMap: false,
      }),
    ],
  },
  // Node-specific build
  {
    input: "src/node.ts",
    output: {
      file: "dist/node.js",
      format: "esm",
      sourcemap: true,
    },
    external,
    plugins: [
      resolve({ preferBuiltins: true }),
      typescript({
        tsconfig: "./tsconfig.json",
        declaration: false,
        declarationMap: false,
      }),
    ],
  },
  // Type definitions
  {
    input: "src/index.ts",
    output: {
      file: "dist/index.d.ts",
      format: "esm",
    },
    external,
    plugins: [dts()],
  },
  {
    input: "src/bundler.ts",
    output: {
      file: "dist/bundler.d.ts",
      format: "esm",
    },
    external,
    plugins: [dts()],
  },
  {
    input: "src/node.ts",
    output: {
      file: "dist/node.d.ts",
      format: "esm",
    },
    external,
    plugins: [dts()],
  },
]);
