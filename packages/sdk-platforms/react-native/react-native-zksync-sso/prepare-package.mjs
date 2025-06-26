import { promises as fs } from "fs";
import path from "path";

const version = process.env.INPUT_VERSION;
const tag = process.env.INPUT_TAG;
const access = process.env.INPUT_ACCESS;

if (!version) {
  console.error("Error: INPUT_VERSION is required.");
  process.exit(1);
}

if (!tag) {
  console.error("Error: INPUT_TAG is required.");
  process.exit(1);
}

if (!access) {
  console.error("Error: INPUT_ACCESS is required.");
  process.exit(1);
}

const packageJsonPath = path.resolve("./package.json");

async function preparePackageJson() {
  try {
    const packageJsonData = await fs.readFile(packageJsonPath, "utf8");
    const packageJson = JSON.parse(packageJsonData);

    // Set the new version
    packageJson.version = version;

    // Update publishConfig if it exists
    if (!packageJson.publishConfig) {
      packageJson.publishConfig = {};
    }
    packageJson.publishConfig.tag = tag;

    // Write the updated package.json back to the file
    await fs.writeFile(packageJsonPath, JSON.stringify(packageJson, null, 2));

    console.log(`Updated package.json for version ${version} with tag ${tag} and access ${access}`);
    console.log(`Package will be published with: npm publish --access ${access} --tag ${tag}`);
  } catch (error) {
    console.error("Error updating package.json:", error);
    process.exit(1);
  }
}

preparePackageJson(); 