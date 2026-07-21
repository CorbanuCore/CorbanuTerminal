import assert from "node:assert/strict";
import { describe, test } from "node:test";

import { preflightSubscriptionPayment } from "../src/payment-preflight.js";

const challenge = {
  x402Version: 2,
  accepts: [{
    scheme: "exact",
    network: "solana:mainnet",
    amount: "1000000",
    asset: "usdc-mint",
    payTo: "receiver",
  }],
};

function fakeFetch(balance: string, receiverExists = true): typeof globalThis.fetch {
  return async (input, init) => {
    if (String(input).includes("gateway.test")) {
      return new Response("{}", {
        status: 402,
        headers: { "payment-required": Buffer.from(JSON.stringify(challenge)).toString("base64") },
      });
    }
    const request = JSON.parse(String(init?.body)) as { params: [string] };
    const owner = request.params[0];
    const value = owner === "receiver" && !receiverExists
      ? []
      : [{ account: { data: { parsed: { info: { tokenAmount: { amount: balance } } } } } }];
    return Response.json({
      jsonrpc: "2.0",
      id: 1,
      result: { value },
    });
  };
}

describe("payment preflight", () => {
  test("returns the offered payment only when the payer balance covers it", async () => {
    const requirement = await preflightSubscriptionPayment(
      new URL("https://gateway.test/v1/subscriptions/starter"),
      "payer",
      "https://rpc.test",
      fakeFetch("1000000"),
    );
    assert.equal(requirement.amount, 1000000n);
    assert.equal(requirement.asset, "usdc-mint");
  });

  test("stops before signing when the payer balance is insufficient", async () => {
    await assert.rejects(
      preflightSubscriptionPayment(
        new URL("https://gateway.test/v1/subscriptions/starter"),
        "payer",
        "https://rpc.test",
        fakeFetch("999999"),
      ),
      /requires 1000000/,
    );
  });

  test("stops before signing when the receiver token account is not initialized", async () => {
    await assert.rejects(
      preflightSubscriptionPayment(
        new URL("https://gateway.test/v1/subscriptions/starter"),
        "payer",
        "https://rpc.test",
        fakeFetch("1000000", false),
      ),
      /receiver .* has no token account/,
    );
  });

  test("rejects malformed challenges and RPC failures", async () => {
    const malformed: typeof globalThis.fetch = async () => new Response("{}", { status: 402 });
    await assert.rejects(
      preflightSubscriptionPayment(
        new URL("https://gateway.test/v1/subscriptions/starter"),
        "payer",
        "https://rpc.test",
        malformed,
      ),
      /omitted PAYMENT-REQUIRED/,
    );
  });
});
