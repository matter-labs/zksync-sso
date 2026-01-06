/**
 * Simple CORS proxy that forwards requests to the configured RPC URL.
 * Used so browser-based apps can hit the zkSync OS RPC without CORS issues.
 */

import http from "http";
import https from "https";
import { URL } from "url";

import { env } from "./config.js";

const RPC_PROXY_PORT = 4339;
const targetUrl = new URL(env.RPC_URL);

const targetPort = targetUrl.port
  ? Number(targetUrl.port)
  : targetUrl.protocol === "https:"
    ? 443
    : 80;

export function startRpcProxy(): http.Server {
  const server = http.createServer((req, res) => {
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    res.setHeader("Access-Control-Allow-Headers", "Content-Type, Authorization");
    res.setHeader("Access-Control-Max-Age", "86400");

    if (req.method === "OPTIONS") {
      res.writeHead(200);
      res.end();
      return;
    }

    const chunks: Buffer[] = [];
    req.on("data", (chunk) => chunks.push(chunk));

    req.on("end", () => {
      const body = Buffer.concat(chunks).toString();
      const headers = {
        ...req.headers,
        host: targetUrl.host,
        origin: targetUrl.origin,
        "content-length": Buffer.byteLength(body).toString(),
      };

      const options: https.RequestOptions = {
        protocol: targetUrl.protocol,
        hostname: targetUrl.hostname,
        port: targetPort,
        path: targetUrl.pathname + targetUrl.search,
        method: req.method,
        headers,
      };

      const requestImpl = targetUrl.protocol === "https:" ? https : http;
      const proxyReq = requestImpl.request(options, (proxyRes) => {
        res.writeHead(proxyRes.statusCode ?? 500, proxyRes.headers);
        proxyRes.pipe(res);
      });

      proxyReq.on("error", (error) => {
        console.error("RPC proxy error:", error);
        res.setHeader("Content-Type", "application/json");
        res.writeHead(502);
        res.end(JSON.stringify({ error: "RPC Proxy Error", message: error.message }));
      });

      if (body) {
        proxyReq.write(body);
      }
      proxyReq.end();
    });
  });

  server.listen(RPC_PROXY_PORT, () => {
    console.log(`RPC CORS proxy listening on port ${RPC_PROXY_PORT}`);
    console.log(`Forwarding requests to ${targetUrl.toString()}`);
    console.log("CORS enabled for RPC requests");
  });

  server.on("error", (error: NodeJS.ErrnoException) => {
    if (error.code === "EADDRINUSE") {
      console.error(`Port ${RPC_PROXY_PORT} is already in use.`);
    } else {
      console.error("RPC proxy server error:", error);
    }
    process.exit(1);
  });

  return server;
}
