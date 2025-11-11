import * as eslintBaseConfig from "../../eslint.config.js";

export default [
  { ignores: ["**/node_modules", "**/dist", "**/temp", "**/tmp", "**/tests", "**/test"] },
  ...eslintBaseConfig.default,
];
