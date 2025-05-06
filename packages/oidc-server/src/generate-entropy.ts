import "dotenv/config";

import { randomBytes } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import * as path from "node:path";

import { bytesToHex } from "viem";

const dirName = import.meta.dirname;

function generateNewEntropy(): string {
  return bytesToHex(randomBytes(48));
}

function noEntropyProvided(): boolean {
  const provided = process.env.SALT_ENTROPY;
  return provided === undefined || provided === "";
}

async function readDotEnv(): Promise<string> {
  const dotEnvPath = path.join(dirName, "..", ".env");
  return readFile(dotEnvPath, "utf8")
    .catch(() => "").then((s) => s.trim());
}

async function writeDotEnv(content: string): Promise<void> {
  const dotEnvPath = path.join(dirName, "..", ".env");
  await writeFile(dotEnvPath, content);
}

async function main() {
  if (noEntropyProvided()) {
    const newEntropy = generateNewEntropy();
    const dotEnvFile = await readDotEnv();
    const filtered = dotEnvFile.replace(/.*SALT_ENTROPY=.*/g, "");
    await writeDotEnv(`${filtered}\nSALT_ENTROPY="${newEntropy}"\n`);
    console.log(`SALT ENTROPY: ${newEntropy}`);
    process.exit(0);
  }

  const provided = process.env.SALT_ENTROPY!;

  if (/^0x[0-9a-fA-F]+$/.test(provided)) {
    console.log(`Entropy was already present.\nSALT ENTROPY: ${provided}`);
    process.exit(0);
  }

  console.log(
    "There is an entropy already set, but it's wrongly generated. Entropy should be encoded as "
    + "a hex string. Please remove SALT_ENTROPY env variable and run again.",
  );
  process.exit(1);
}

await main();
