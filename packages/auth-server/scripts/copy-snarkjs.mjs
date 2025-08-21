#!/usr/bin/env node
import { copyFileSync, existsSync, mkdirSync, readdirSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { basename, dirname, join, parse, resolve } from "node:path";
import { fileURLToPath } from "node:url";

function main() {
  // Resolve the path to snarkjs browser bundle.
  // We cannot rely on snarkjs/package.json because it is not exported.
  let baseDir;
  try {
    const require = createRequire(import.meta.url);
    const mainEntry = require.resolve("snarkjs"); // e.g. node_modules/snarkjs/dist/main.cjs
    // ascend until we reach the package root (folder named 'snarkjs')
    let curr = dirname(mainEntry);
    while (curr && curr !== parse(curr).root && basename(curr) === "snarkjs") {
      if (curr.endsWith("snarkjs")) {
        baseDir = curr;
        break;
      }
      curr = dirname(curr);
    }
    if (!baseDir) {
      // fallback: one directory up from dist
      baseDir = dirname(dirname(mainEntry));
    }
  } catch (e) {
    console.warn("[copy-snarkjs] snarkjs not installed yet; skipping copy", e);
    return; // don't fail install, maybe another step will install it
  }
  const candidateRelPaths = ["dist/web/snarkjs.min.js", "dist/snarkjs.min.js", "build/snarkjs.min.js", "snarkjs.min.js"];
  let src;
  for (const rel of candidateRelPaths) {
    const cand = join(baseDir, rel);
    if (existsSync(cand)) {
      src = cand;
      break;
    }
  }
  if (!src) {
    // Last resort: scan first level dirs for snarkjs.min.js
    try {
      const entries = readdirSync(baseDir, { withFileTypes: true });
      for (const ent of entries) {
        if (ent.isDirectory()) {
          const cand = join(baseDir, ent.name, "snarkjs.min.js");
          if (existsSync(cand)) {
            src = cand;
            break;
          }
        }
      }
    } catch (e) {
      console.warn("[copy-snarkjs] Error scanning for snarkjs.min.js", e);
    }
  }
  if (!src) {
    console.warn(
      "[copy-snarkjs] Could not locate snarkjs.min.js inside package; looked in: " + candidateRelPaths.join(", "),
    );
    return;
  }
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);
  const destDir = resolve(__dirname, "../public");
  const dest = resolve(destDir, "snarkjs.min.js");

  try {
    statSync(destDir);
  } catch {
    mkdirSync(destDir, { recursive: true });
  }
  if (existsSync(dest)) {
    try {
      const srcStat = statSync(src);
      const destStat = statSync(dest);
      if (destStat.mtimeMs >= srcStat.mtimeMs) {
        console.log("[copy-snarkjs] Existing snarkjs.min.js is up to date");
        return;
      }
    } catch (e) {
      console.warn("[copy-snarkjs] Error checking existing snarkjs.min.js", e);
    }
  }
  copyFileSync(src, dest);
  console.log(`[copy-snarkjs] Copied ${src} -> ${dest}`);
}

main();
