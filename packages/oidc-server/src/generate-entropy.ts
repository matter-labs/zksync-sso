import "dotenv/config";

import { randomBytes } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import * as path from "node:path";

const dirName = import.meta.dirname;

async function main() {
  const provided = process.env.SALT_ENTROPY;
  if (provided === undefined || provided === "") {
    const newEntropy = randomBytes(48);
    const dotEnvPath = path.join(dirName, "..", ".env");
    const dotEnvFile = await readFile(dotEnvPath, "utf8")
      .catch(() => "");
    const filtered = dotEnvFile.replace(/.*SALT_ENTROPY=.*/g, "");

    await writeFile(dotEnvPath, `${filtered}\nSALT_ENTROPY="0x${newEntropy.toString("hex")}"\n`);
    console.log(`SALT ENTROPY: 0x${newEntropy.toString("hex")}`);
    process.exit(0);
  } else {
    if (/^0x[0-9a-fA-F]+$/.test(provided)) {
      console.log(`SALT ENTROPY: ${provided}`);
    } else {
      console.log(
        "There is an entropy already set, but it's wrongly generated. Entropy should be encoded as "
        + "a hex string. Please remove SALT_ENTROPY env variable and run again.",
      );
      process.exit(1);
    }
  }
}

await main();
