import { Pool } from "pg";

import { createGatewayApp } from "./app.js";
import { readGatewayConfig } from "./config.js";
import { PostgresGatewayStore } from "./postgres-store.js";
import { createX402Middleware } from "./x402.js";

const config = readGatewayConfig();
const pool = new Pool({ connectionString: config.databaseUrl });
const store = new PostgresGatewayStore(pool);
await store.initialize();

const paymentMiddleware = createX402Middleware({
  store,
  network: config.network,
  payTo: config.payTo,
  publicBaseUrl: config.publicBaseUrl,
  facilitatorUrl: config.facilitatorUrl.toString(),
});
const app = createGatewayApp({
  store,
  tokenPepper: config.tokenPepper,
  ambientApiKey: config.ambientApiKey,
  ambientBaseUrl: config.ambientBaseUrl.toString(),
  paymentMiddleware,
  readiness: async () => {
    await pool.query("SELECT 1");
  },
});

const server = app.listen(config.port, config.host, () => {
  process.stdout.write(`Ambient crypto gateway listening on ${config.host}:${config.port}\n`);
});

async function shutdown(): Promise<void> {
  server.close();
  await pool.end();
}

process.once("SIGINT", shutdown);
process.once("SIGTERM", shutdown);
