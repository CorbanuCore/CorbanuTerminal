import assert from "node:assert/strict";
import { after, before, beforeEach, describe, test } from "node:test";

import { Pool } from "pg";

import { PostgresGatewayStore } from "../src/postgres-store.js";
import { hashToken } from "../src/token.js";

const DATABASE_URL = process.env.TEST_DATABASE_URL;
const NOW = new Date("2026-07-18T12:00:00.000Z");
const PEPPER = "test-pepper-that-is-at-least-thirty-two-characters";

describe("PostgreSQL gateway store", { skip: !DATABASE_URL }, () => {
  const pool = new Pool({ connectionString: DATABASE_URL });
  const store = new PostgresGatewayStore(pool);

  before(async () => store.initialize());
  beforeEach(async () => {
    await pool.query(`
      TRUNCATE ambient_used_siwx_nonces, ambient_inference_ledger,
        ambient_weekly_windows, ambient_api_keys, ambient_subscription_periods;
    `);
  });
  after(async () => pool.end());

  test("serializes concurrent purchases into distinct monthly periods", async () => {
    const periods = await Promise.all(
      ["tx-1", "tx-2", "tx-3", "tx-4"].map(transaction =>
        store.recordSettlement({
          transaction,
          walletAddress: "wallet-1",
          planId: "basic",
          network: "solana:devnet",
          amountAtomic: "20000000",
          settledAt: NOW,
        }),
      ),
    );
    periods.sort((left, right) => left.startsAt.getTime() - right.startsAt.getTime());
    assert.equal(periods.length, 4);
    for (let index = 1; index < periods.length; index += 1) {
      const current = periods[index];
      const previous = periods[index - 1];
      assert.ok(current && previous);
      assert.equal(current.startsAt.toISOString(), previous.endsAt.toISOString());
    }
  });

  test("keeps settlement retries idempotent and rejects transaction rebinding", async () => {
    const settlement = {
      transaction: "tx-idempotent",
      walletAddress: "wallet-1",
      planId: "starter" as const,
      network: "solana:devnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    };
    const first = await store.recordSettlement(settlement);
    const second = await store.recordSettlement(settlement);
    assert.deepEqual(second, first);
    await assert.rejects(
      store.recordSettlement({ ...settlement, walletAddress: "wallet-2" }),
      /already bound/,
    );
  });

  test("persists customer key authentication and revocation", async () => {
    await store.recordSettlement({
      transaction: "tx-key",
      walletAddress: "wallet-1",
      planId: "starter",
      network: "solana:devnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });
    const created = await store.createApiKey("wallet-1", NOW, PEPPER);
    const secondProcess = new PostgresGatewayStore(pool);
    assert.ok(await secondProcess.authenticateApiKey(hashToken(created.key, PEPPER), NOW));
    assert.equal(await secondProcess.revokeApiKey("wallet-1", created.id, NOW), true);
    assert.equal(await store.authenticateApiKey(hashToken(created.key, PEPPER), NOW), undefined);
  });

  test("atomically caps concurrent usage across multiple customer keys", async () => {
    await store.recordSettlement({
      transaction: "tx-usage",
      walletAddress: "wallet-usage",
      planId: "starter",
      network: "solana:devnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });
    const keys = await Promise.all([
      store.createApiKey("wallet-usage", NOW, PEPPER),
      store.createApiKey("wallet-usage", NOW, PEPPER),
    ]);
    const hashes = keys.map(key => hashToken(key.key, PEPPER));
    const authorizations = await Promise.all(
      Array.from({ length: 20 }, (_, index) =>
        store.reserveApiKeyUsage(
          hashes[index % hashes.length]!,
          `request-${index}`,
          "z-ai/glm-5.2",
          25_000,
          NOW,
        ),
      ),
    );
    assert.equal(authorizations.filter(value => value?.kind === "authorized").length, 10);
    const period = (await store.listPeriods("wallet-usage", NOW))[0];
    assert.equal(period?.monthlyReservedTokens, 250_000);
    assert.equal(period?.monthlyUsedTokens, 0);
  });

  test("atomically rejects concurrent nonce replay", async () => {
    const results = await Promise.all(
      Array.from({ length: 10 }, () => store.hasUsedNonce("one-nonce")),
    );
    assert.equal(results.filter(value => value === false).length, 1);
    assert.equal(results.filter(value => value === true).length, 9);
  });
});
