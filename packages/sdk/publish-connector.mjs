import { execSync } from "child_process";
import { promises as fs } from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const version = process.env.INPUT_VERSION;

if (!version) {
  console.error("Error: INPUT_VERSION is required.");
  process.exit(1);
}

async function publishConnector() {
  const tempDir = "temp-publish";
  try {
    // Create a temporary directory
    await fs.mkdir(tempDir, { recursive: true });

    // Copy build artifacts
    await fs.cp(path.join(__dirname, "dist"), path.join(tempDir, "dist"), { recursive: true });

    // Read the connector-specific package.json
    const connectorPackageJsonData = await fs.readFile(path.join(__dirname, "wagmi-connector.package.json"), "utf8");
    const connectorPackageJson = JSON.parse(connectorPackageJsonData);

    // Set the version and other properties
    connectorPackageJson.version = version;
    const originalPackageJsonData = await fs.readFile(path.join(__dirname, "package.json"), "utf8");
    const originalPackageJson = JSON.parse(originalPackageJsonData);
    connectorPackageJson.author = originalPackageJson.author;
    connectorPackageJson.license = originalPackageJson.license;
    connectorPackageJson.repository = originalPackageJson.repository;

    // Write the new package.json to the temp directory
    await fs.writeFile(path.join(tempDir, "package.json"), JSON.stringify(connectorPackageJson, null, 2));

    // Copy the connector-specific README
    await fs.copyFile(path.join(__dirname, "README-wagmi-connector.md"), path.join(tempDir, "README.md"));

    // Publish from the temp directory
    execSync("npm publish --access public", { cwd: tempDir, stdio: "inherit" });

    console.log(`Successfully published zksync-sso-wagmi-connector version ${version}`);
  } catch (error) {
    console.error("Error publishing wagmi connector:", error);
    process.exit(1);
  } finally {
    // Clean up the temporary directory
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

publishConnector();
