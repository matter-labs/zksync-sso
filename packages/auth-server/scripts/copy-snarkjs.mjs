#!/usr/bin/env node
import { mkdirSync, copyFileSync, statSync, existsSync, readdirSync } from 'node:fs';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

function main() {
  // Resolve the path to snarkjs browser bundle
  // Resolve snarkjs package base
  let baseDir;
  try {
    const pkgJson = require.resolve('snarkjs/package.json');
    baseDir = dirname(pkgJson);
  } catch (e) {
    console.warn('[copy-snarkjs] snarkjs not installed yet; skipping copy');
    return; // don't fail install, maybe another step will install it
  }
  const candidateRelPaths = [
    'dist/web/snarkjs.min.js',
    'dist/snarkjs.min.js',
    'build/snarkjs.min.js'
  ];
  let src;
  for (const rel of candidateRelPaths) {
    const cand = join(baseDir, rel);
    if (existsSync(cand)) { src = cand; break; }
  }
  if (!src) {
    // Last resort: scan first level dirs for snarkjs.min.js
    try {
      const entries = readdirSync(baseDir, { withFileTypes: true });
      for (const ent of entries) {
        if (ent.isDirectory()) {
          const cand = join(baseDir, ent.name, 'snarkjs.min.js');
          if (existsSync(cand)) { src = cand; break; }
        }
      }
    } catch {}
  }
  if (!src) {
    console.warn('[copy-snarkjs] Could not locate snarkjs.min.js inside package; looked in: ' + candidateRelPaths.join(', '));
    return;
  }
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);
  const destDir = resolve(__dirname, '../public');
  const dest = resolve(destDir, 'snarkjs.min.js');

  try { statSync(destDir); } catch { mkdirSync(destDir, { recursive: true }); }
  if (existsSync(dest)) {
    try {
      const srcStat = statSync(src);
      const destStat = statSync(dest);
      if (destStat.mtimeMs >= srcStat.mtimeMs) {
        console.log('[copy-snarkjs] Existing snarkjs.min.js is up to date');
        return;
      }
    } catch {}
  }
  copyFileSync(src, dest);
  console.log(`[copy-snarkjs] Copied ${src} -> ${dest}`);
}

main();
