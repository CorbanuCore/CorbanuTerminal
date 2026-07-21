import assert from "node:assert/strict";
import { describe, test } from "node:test";

import request from "supertest";

import { createGatewayApp } from "../src/app.js";
import { SOLANA_MAINNET } from "../src/config.js";
import { InMemoryGatewayStore } from "../src/store.js";
import { createX402Middleware } from "../src/x402.js";

const RUN_LIVE = process.env.RUN_LIVE_X402_DISCOVERY === "1";

describe("live x402 facilitator discovery", { skip: !RUN_LIVE }, () => {
  test("advertises the starter purchase as Solana-mainnet USDC", async () => {
    const store = new InMemoryGatewayStore();
    const paymentMiddleware = createX402Middleware({
      store,
      network: SOLANA_MAINNET,
      payTo: "11111111111111111111111111111111",
      publicBaseUrl: new URL("https://gateway.example"),
      facilitatorUrl: "https://facilitator.payai.network",
    });
    const app = createGatewayApp({
      store,
      tokenPepper: "test-pepper-that-is-at-least-thirty-two-characters",
      ambientApiKey: "unused-upstream-key",
      paymentMiddleware,
    });

    const response = await request(app).post("/v1/subscriptions/starter").expect(402);
    const encoded = response.headers["payment-required"];
    if (typeof encoded !== "string") throw new Error("payment-required header was missing");
    const challenge = JSON.parse(Buffer.from(encoded, "base64").toString("utf8")) as {
      x402Version: number;
      accepts: Array<{ network: string; amount: string; payTo: string }>;
    };
    assert.equal(challenge.x402Version, 2);
    assert.equal(challenge.accepts[0]?.network, SOLANA_MAINNET);
    assert.equal(challenge.accepts[0]?.amount, "1000000");
    assert.equal(challenge.accepts[0]?.payTo, "11111111111111111111111111111111");
  });
});
