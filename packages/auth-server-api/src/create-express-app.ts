import express from "express";

export function createExpressApp() {
  const app = express();
  app.disable("x-powered-by");
  return app;
}
