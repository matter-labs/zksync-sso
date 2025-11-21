#!/usr/bin/env node

/**
 * Entry point for the Alto Bundler
 * Loads configuration and starts the Alto bundler with CORS proxy
 */

import "./config.js";

import { startBundler } from "./bundler-with-proxy.js";

// Import config to trigger validation and generate alto-config.json

console.log("Starting Alto Bundler...");

// Start the bundler with proxy
startBundler().catch((error) => {
  console.error(`Failed to start bundler: ${error.message}`);
  process.exit(1);
});
