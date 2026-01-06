/* eslint-disable prefer-const */
/**
 * Runs Alto bundler on port 4338 and CORS proxy on port 4337
 * This allows browser-based applications to interact with Alto without CORS issues
 */

import { type ChildProcess, spawn } from "child_process";
import type http from "http";
import net from "net";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

import { startProxy } from "./bundler-proxy.js";
import { startRpcProxy } from "./rpc-cors-proxy.js";

const __dirname = dirname(fileURLToPath(import.meta.url));

const ALTO_CONFIG = join(__dirname, "..", "alto-config.json");

// ANSI color codes
const colors = {
  reset: "\x1b[0m",
  cyan: "\x1b[36m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  red: "\x1b[31m",
};

function log(prefix: string, message: string, color = colors.reset) {
  console.log(`${color}[${prefix}]${colors.reset} ${message}`);
}

/**
 * Check if a port is listening
 */
async function waitForPort(port: number, maxAttempts = 30, delayMs = 1000): Promise<boolean> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const isListening = await new Promise<boolean>((resolve) => {
      const client = new net.Socket();

      client.once("connect", () => {
        client.destroy();
        resolve(true);
      });

      client.once("error", () => {
        client.destroy();
        resolve(false);
      });

      client.connect(port, "localhost");
    });

    if (isListening) {
      return true;
    }

    if (attempt < maxAttempts) {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }

  return false;
}

export async function startBundler(): Promise<void> {
  let alto: ChildProcess | undefined;
  let proxyServer: http.Server | undefined;
  let rpcProxyServer: http.Server | undefined;

  // Cleanup handler
  const cleanup = () => {
    log("SETUP", "Shutting down...", colors.yellow);
    if (alto) {
      alto.kill();
    }
    if (rpcProxyServer) {
      rpcProxyServer.close();
    }
    if (proxyServer) {
      proxyServer.close();
    }
    process.exit(0);
  };

  // Register cleanup handlers
  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);

  // Start Alto bundler
  log("SETUP", "Starting Alto bundler on port 4338...", colors.cyan);
  alto = spawn("npx", ["alto", "--config", ALTO_CONFIG, "--port", "4338"], {
    stdio: "inherit",
    shell: true,
  });

  alto.on("error", (error) => {
    log("ALTO", `Failed to start: ${error.message}`, colors.red);
    process.exit(1);
  });

  alto.on("exit", (code) => {
    log("ALTO", `Exited with code ${code}`, colors.yellow);
    if (proxyServer) {
      proxyServer.close();
    }
    if (rpcProxyServer) {
      rpcProxyServer.close();
    }
    process.exit(code || 0);
  });

  // Wait for Alto to be ready
  log("SETUP", "Waiting for Alto bundler to be ready on port 4338...", colors.cyan);

  const altoReady = await waitForPort(4338);
  if (!altoReady) {
    log("ALTO", "Failed to start - port 4338 not listening after 30 seconds", colors.red);
    alto.kill();
    process.exit(1);
  }

  log("ALTO", "Ready and listening on port 4338", colors.green);
  log("SETUP", "Starting CORS proxy on port 4337...", colors.cyan);

  // Start proxy server
  proxyServer = startProxy();

  // Wait for proxy to be ready
  log("SETUP", "Waiting for CORS proxy to be ready on port 4337...", colors.cyan);
  const proxyReady = await waitForPort(4337);
  if (!proxyReady) {
    log("PROXY", "Failed to start - port 4337 not listening after 30 seconds", colors.red);
    alto.kill();
    proxyServer.close();
    process.exit(1);
  }

  log("PROXY", "Ready and listening on port 4337", colors.green);

  log("SETUP", "Starting RPC CORS proxy on port 4339...", colors.cyan);
  rpcProxyServer = startRpcProxy();

  const rpcProxyReady = await waitForPort(4339);
  if (!rpcProxyReady) {
    log("RPC", "Failed to start - port 4339 not listening after 30 seconds", colors.red);
    alto.kill();
    proxyServer.close();
    rpcProxyServer.close();
    process.exit(1);
  }

  log("RPC", "Ready and listening on port 4339", colors.green);
  log("SETUP", "All services started successfully!", colors.green);
  log("SETUP", "Alto bundler: http://localhost:4338", colors.green);
  log("SETUP", "Bundler CORS proxy: http://localhost:4337", colors.green);
  log("SETUP", "RPC CORS proxy: http://localhost:4339", colors.green);
}
