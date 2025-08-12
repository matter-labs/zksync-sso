import { promises as fs } from "fs";
import path from "path";

const version = process.env.INPUT_VERSION;
if (!version) {
  console.error("Error: INPUT_VERSION is required.");
  process.exit(1);
}

const packageJsonPath = path.resolve("./package.json");

async function preparePackageJson() {
  try {
    const packageJsonData = await fs.readFile(packageJsonPath, "utf8");
    const packageJson = JSON.parse(packageJsonData);

    // Remove unnecessary properties for publishing
    delete packageJson.private;
    delete packageJson.publishConfig;

    // Set the new version
    packageJson.version = version;

    // Transform workspace dependencies to proper version ranges
    if (packageJson.peerDependencies) {
      for (const [dep, depVersion] of Object.entries(packageJson.peerDependencies)) {
        if (depVersion === "workspace:*") {
          // Replace workspace protocol with proper version range
          if (dep === "zksync-sso") {
            packageJson.peerDependencies[dep] = `^${version}`;
            console.log(`Replaced workspace dependency for ${dep} with ^${version}`);
          }
        }
      }
    }

    // Also check regular dependencies if they exist
    if (packageJson.dependencies) {
      for (const [dep, depVersion] of Object.entries(packageJson.dependencies)) {
        if (depVersion === "workspace:*") {
          if (dep === "zksync-sso") {
            packageJson.dependencies[dep] = `^${version}`;
            console.log(`Replaced workspace dependency for ${dep} with ^${version}`);
          }
        }
      }
    }

    // Write the updated package.json back to the file
    await fs.writeFile(packageJsonPath, JSON.stringify(packageJson, null, 2));

    console.log(`Updated connector-export package.json for version ${version}`);
  } catch (error) {
    console.error("Error updating connector-export package.json:", error);
    process.exit(1);
  }
}

preparePackageJson();
