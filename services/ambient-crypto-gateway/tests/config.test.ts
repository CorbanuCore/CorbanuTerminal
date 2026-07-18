import assert from "node:assert/strict";
import { describe, test } from "node:test";

import { readGatewayConfig, SOLANA_DEVNET, SOLANA_MAINNET } from "../src/config.js";

const VALID_ENV: NodeJS.ProcessEnv = {
  DATABASE_URL: "postgresql://localhost/test",
  AMBIENT_API_KEY: "upstream-key",
  PFT_AMBIENT_TOKEN_PEPPER: "a-secure-pepper-with-at-least-thirty-two-characters",
  PFT_AMBIENT_PUBLIC_BASE_URL: "https://ambient-gateway.example",
  PFT_X402_NETWORK: SOLANA_DEVNET,
  PFT_X402_PAY_TO: "11111111111111111111111111111111",
};

describe("gateway configuration", () => {
  test("defaults devnet to the test facilitator and loopback binding", () => {
    const config = readGatewayConfig(VALID_ENV);
    assert.equal(config.host, "127.0.0.1");
    assert.equal(config.facilitatorUrl.toString(), "https://x402.org/facilitator");
  });

  test("defaults mainnet to the live Solana facilitator", () => {
    const config = readGatewayConfig({ ...VALID_ENV, PFT_X402_NETWORK: SOLANA_MAINNET });
    assert.equal(config.facilitatorUrl.toString(), "https://facilitator.payai.network/");
  });

  test("fails closed on missing secrets, weak token pepper, and malformed receiver", () => {
    assert.throws(() => readGatewayConfig({ ...VALID_ENV, AMBIENT_API_KEY: "" }), /required/);
    assert.throws(
      () => readGatewayConfig({ ...VALID_ENV, PFT_AMBIENT_TOKEN_PEPPER: "short" }),
      /32 characters/,
    );
    assert.throws(
      () => readGatewayConfig({ ...VALID_ENV, PFT_X402_PAY_TO: "not-a-wallet" }),
      /Solana address/,
    );
  });

  test("rejects insecure remote endpoints", () => {
    assert.throws(
      () => readGatewayConfig({ ...VALID_ENV, AMBIENT_BASE_URL: "http://ambient.example" }),
      /HTTPS/,
    );
  });
});
