import assert from "node:assert/strict";
import type { Server } from "node:http";
import { after, before, describe, it } from "node:test";

import { createExpressApp } from "../src/create-express-app.ts";

const app = createExpressApp();
let server: Server | undefined;

before(async () => {
  app.get("/health", (_req, res) => {
    res.json({ status: "ok" });
  });

  await new Promise<void>((resolve) => {
    server = app.listen(0, resolve);
  });
});

after(() => {
  server?.close();
});

describe("createExpressApp", () => {
  it("does not expose Express fingerprinting headers", async () => {
    assert(server);
    const address = server.address();
    assert(address && typeof address === "object");

    const response = await fetch(`http://127.0.0.1:${address.port}/health`);

    assert.equal(response.status, 200);
    assert.equal(response.headers.has("x-powered-by"), false);
  });
});
