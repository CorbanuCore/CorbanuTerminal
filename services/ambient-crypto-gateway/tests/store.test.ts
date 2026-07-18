import assert from "node:assert/strict";
import { describe, test } from "node:test";

import { InMemoryGatewayStore, addUtcCalendarMonth } from "../src/store.js";
import { hashToken } from "../src/token.js";

const PEPPER = "test-pepper-that-is-at-least-thirty-two-characters";
const NOW = new Date("2026-01-31T12:00:00.000Z");

describe("subscription store", () => {
  test("adds calendar months without rolling across short months", () => {
    assert.equal(addUtcCalendarMonth(NOW).toISOString(), "2026-02-28T12:00:00.000Z");
  });

  test("binds a settled transaction to one period and makes retries idempotent", async () => {
    const store = new InMemoryGatewayStore();
    const settlement = {
      transaction: "solana-tx-1",
      walletAddress: "wallet-1",
      planId: "starter" as const,
      network: "solana:mainnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    };

    const first = await store.recordSettlement(settlement);
    const retry = await store.recordSettlement(settlement);

    assert.deepEqual(retry, first);
    assert.deepEqual(await store.listPeriods("wallet-1", NOW), [first]);
  });

  test("serializes purchases into non-overlapping monthly periods", async () => {
    const store = new InMemoryGatewayStore();
    const first = await store.recordSettlement({
      transaction: "solana-tx-1",
      walletAddress: "wallet-1",
      planId: "starter",
      network: "solana:mainnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });
    const second = await store.recordSettlement({
      transaction: "solana-tx-2",
      walletAddress: "wallet-1",
      planId: "basic",
      network: "solana:mainnet",
      amountAtomic: "20000000",
      settledAt: new Date("2026-02-01T12:00:00.000Z"),
    });

    assert.equal(second.startsAt.toISOString(), first.endsAt.toISOString());
    assert.equal(second.planId, "basic");
  });

  test("refuses a thirteenth stacked month", async () => {
    const store = new InMemoryGatewayStore();
    for (let index = 0; index < 12; index += 1) {
      await store.recordSettlement({
        transaction: `solana-tx-${index}`,
        walletAddress: "wallet-1",
        planId: "starter",
        network: "solana:mainnet",
        amountAtomic: "1000000",
        settledAt: NOW,
      });
    }

    await assert.rejects(
      store.recordSettlement({
        transaction: "solana-tx-13",
        walletAddress: "wallet-1",
        planId: "starter",
        network: "solana:mainnet",
        amountAtomic: "1000000",
        settledAt: NOW,
      }),
      /beyond 12 months/,
    );
  });

  test("issues revocable opaque keys only while a subscription is active", async () => {
    const store = new InMemoryGatewayStore();
    await assert.rejects(store.createApiKey("wallet-1", NOW, PEPPER), /active subscription/);
    await store.recordSettlement({
      transaction: "solana-tx-1",
      walletAddress: "wallet-1",
      planId: "starter",
      network: "solana:mainnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });

    const issued = await store.createApiKey("wallet-1", NOW, PEPPER);
    assert.match(issued.key, /^pft_amb_/);
    assert.ok(await store.authenticateApiKey(hashToken(issued.key, PEPPER), NOW));
    assert.equal(await store.revokeApiKey("wallet-1", issued.id, NOW), true);
    assert.equal(await store.authenticateApiKey(hashToken(issued.key, PEPPER), NOW), undefined);
  });

  test("rejects rebinding a transaction to another wallet or plan", async () => {
    const store = new InMemoryGatewayStore();
    await store.recordSettlement({
      transaction: "solana-tx-1",
      walletAddress: "wallet-1",
      planId: "starter",
      network: "solana:mainnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });

    await assert.rejects(
      store.recordSettlement({
        transaction: "solana-tx-1",
        walletAddress: "wallet-2",
        planId: "basic",
        network: "solana:mainnet",
        amountAtomic: "20000000",
        settledAt: NOW,
      }),
      /already bound/,
    );
  });

  test("tracks SIWX nonces to reject replay", () => {
    const store = new InMemoryGatewayStore();
    assert.equal(store.hasUsedNonce("nonce-1"), false);
    store.recordNonce("nonce-1");
    assert.equal(store.hasUsedNonce("nonce-1"), true);
  });
});
