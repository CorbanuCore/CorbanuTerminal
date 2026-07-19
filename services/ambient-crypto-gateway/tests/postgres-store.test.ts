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

  before(async () => {
    const identity = await pool.query<{ database: string }>("SELECT current_database() AS database");
    const database = identity.rows[0]?.database ?? "";
    if (!database.endsWith("_test")) {
      throw new Error(`refusing destructive PostgreSQL tests against non-test database: ${database}`);
    }
    await store.initialize();
  });
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

  test("refunds legacy full-reservation charges exactly once during initialization", async () => {
    await store.recordSettlement({
      transaction: "tx-legacy-refund", walletAddress: "wallet-refund", planId: "starter",
      network: "solana:devnet", amountAtomic: "1000000", settledAt: NOW,
    });
    const key = await store.createApiKey("wallet-refund", NOW, PEPPER);
    const authorization = await store.reserveApiKeyUsage(
      hashToken(key.key, PEPPER), "legacy-request", "z-ai/glm-5.2", 32_768, NOW,
    );
    assert.equal(authorization?.kind, "authorized");
    await pool.query(
      "UPDATE ambient_subscription_periods SET monthly_reserved_tokens=0,monthly_used_tokens=32768 WHERE transaction='tx-legacy-refund'",
    );
    await pool.query(
      "UPDATE ambient_weekly_windows SET reserved_tokens=0,used_tokens=32768 WHERE period_transaction='tx-legacy-refund'",
    );
    await pool.query(
      "UPDATE ambient_inference_ledger SET state='settled',charged_tokens=reserved_tokens,settled_at=$1,usage_source=NULL WHERE request_id='legacy-request'",
      [NOW],
    );

    await store.initialize();
    await store.initialize();

    const account = await store.accountForApiKey(hashToken(key.key, PEPPER), NOW);
    assert.equal(account?.period.monthlyUsedTokens, 0);
    assert.equal(account?.weekly.usedTokens, 0);
    const ledger = await pool.query(
      "SELECT state,charged_tokens,usage_source FROM ambient_inference_ledger WHERE request_id='legacy-request'",
    );
    assert.deepEqual(ledger.rows[0], {
      state: "released", charged_tokens: "0", usage_source: "legacy_unmetered",
    });
  });

  test("conservatively settles orphaned reservations during initialization", async () => {
    await store.recordSettlement({
      transaction: "tx-restart-recovery", walletAddress: "wallet-restart", planId: "starter",
      network: "solana:devnet", amountAtomic: "1000000", settledAt: NOW,
    });
    const key = await store.createApiKey("wallet-restart", NOW, PEPPER);
    const authorization = await store.reserveApiKeyUsage(
      hashToken(key.key, PEPPER), "interrupted-request", "z-ai/glm-5.2", 32_768, NOW,
    );
    assert.equal(authorization?.kind, "authorized");

    const before = await store.accountForApiKey(hashToken(key.key, PEPPER), NOW);
    assert.equal(before?.period.monthlyReservedTokens, 32_768);
    assert.equal(before?.weekly.reservedTokens, 32_768);

    await new PostgresGatewayStore(pool).initialize();

    const after = await store.accountForApiKey(hashToken(key.key, PEPPER), NOW);
    assert.equal(after?.period.monthlyReservedTokens, 0);
    assert.equal(after?.weekly.reservedTokens, 0);
    assert.equal(after?.period.monthlyUsedTokens, 32_768);
    assert.equal(after?.weekly.usedTokens, 32_768);
    const ledger = await pool.query(
      "SELECT state,charged_tokens,usage_source FROM ambient_inference_ledger WHERE request_id='interrupted-request'",
    );
    assert.deepEqual(ledger.rows[0], {
      state: "settled", charged_tokens: "32768", usage_source: "reservation",
    });
  });

  test("atomically rejects concurrent nonce replay", async () => {
    const results = await Promise.all(
      Array.from({ length: 10 }, () => store.hasUsedNonce("one-nonce")),
    );
    assert.equal(results.filter(value => value === false).length, 1);
    assert.equal(results.filter(value => value === true).length, 9);
  });
});
