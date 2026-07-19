import assert from "node:assert/strict";
import { after, before, beforeEach, describe, test } from "node:test";

import { Pool } from "pg";

import {
  loadAccountingReport,
  renderAccountingReport,
  setWalletClassification,
} from "../src/accounting-store.js";
import { PostgresGatewayStore } from "../src/postgres-store.js";

const DATABASE_URL = process.env.TEST_DATABASE_URL;
const NOW = new Date("2026-07-19T12:00:00.000Z");
const RECEIVER = "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd";
const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

describe("accounting store", { skip: !DATABASE_URL }, () => {
  const pool = new Pool({ connectionString: DATABASE_URL });
  const store = new PostgresGatewayStore(pool);

  before(async () => {
    const identity = await pool.query<{ database: string }>(
      "SELECT current_database() AS database",
    );
    const database = identity.rows[0]?.database ?? "";
    if (!database.endsWith("_test")) {
      throw new Error(
        `refusing destructive accounting tests against non-test database: ${database}`,
      );
    }
    await store.initialize();
  });
  beforeEach(async () => {
    await pool.query(`
      TRUNCATE ambient_accounting_wallets, ambient_used_siwx_nonces, ambient_inference_ledger,
        ambient_weekly_windows, ambient_api_keys, ambient_subscription_periods;
    `);
  });
  after(async () => pool.end());

  test("keeps unclassified volume out of recognized revenue", async () => {
    await store.recordSettlement({
      transaction: "tx-unclassified",
      walletAddress: "3speRmSn2J3fhpxpY2B8eQQB8h9i569TajGifjJRwV5r",
      planId: "starter",
      network: "solana:mainnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });
    const report = await loadAccountingReport({
      pool,
      receiverAddress: RECEIVER,
      usdcMint: USDC_MINT,
      solanaRpcUrl: "https://rpc.example.test",
      now: NOW,
      fetch: rpcBalance("394000000"),
    });
    assert.deepEqual(
      {
        receiverUsdc: report.receiverUsdc,
        recordedGrossUsdc: report.recordedGrossUsdc,
        recognizedCustomerRevenueUsdc: report.recognizedCustomerRevenueUsdc,
        internalVolumeUsdc: report.internalVolumeUsdc,
        unclassifiedVolumeUsdc: report.unclassifiedVolumeUsdc,
        unclassifiedWallets: report.unclassifiedWallets,
      },
      {
        receiverUsdc: "394",
        recordedGrossUsdc: "1",
        recognizedCustomerRevenueUsdc: "0",
        internalVolumeUsdc: "0",
        unclassifiedVolumeUsdc: "1",
        unclassifiedWallets: [
          {
            walletAddress: "3speRmSn2J3fhpxpY2B8eQQB8h9i569TajGifjJRwV5r",
            periods: 1,
            grossUsdc: "1",
          },
        ],
      },
    );
  });

  test("reclassifies wallets without changing settlement records", async () => {
    const wallet = "3speRmSn2J3fhpxpY2B8eQQB8h9i569TajGifjJRwV5r";
    await store.recordSettlement({
      transaction: "tx-customer",
      walletAddress: wallet,
      planId: "basic",
      network: "solana:mainnet",
      amountAtomic: "20000000",
      settledAt: NOW,
    });
    await setWalletClassification(
      pool,
      wallet,
      "customer",
      "first production customer",
      NOW,
    );
    const customer = await report(pool);
    assert.equal(customer.recognizedCustomerRevenueUsdc, "20");
    assert.equal(customer.internalVolumeUsdc, "0");
    assert.deepEqual(customer.unclassifiedWallets, []);

    await setWalletClassification(
      pool,
      wallet,
      "internal",
      "qualification wallet",
      NOW,
    );
    const internal = await report(pool);
    assert.equal(internal.recognizedCustomerRevenueUsdc, "0");
    assert.equal(internal.internalVolumeUsdc, "20");

    await setWalletClassification(pool, wallet, "unclassified", undefined, NOW);
    const unclassified = await report(pool);
    assert.equal(unclassified.unclassifiedVolumeUsdc, "20");
  });

  test("rejects malformed classifications and renders an operator-readable report", async () => {
    await assert.rejects(
      setWalletClassification(
        pool,
        "not-a-wallet",
        "customer",
        "customer",
        NOW,
      ),
      /valid base58/,
    );
    await assert.rejects(
      setWalletClassification(pool, RECEIVER, "customer", "", NOW),
      /require a label/,
    );
    const rendered = renderAccountingReport(await report(pool));
    assert.match(rendered, /Recognized customer revenue: 0 USDC/);
    assert.match(rendered, /Receiver balance: 394 USDC/);
  });
});

function report(pool: Pool) {
  return loadAccountingReport({
    pool,
    receiverAddress: RECEIVER,
    usdcMint: USDC_MINT,
    solanaRpcUrl: "https://rpc.example.test",
    now: NOW,
    fetch: rpcBalance("394000000"),
  });
}

function rpcBalance(amount: string): typeof globalThis.fetch {
  return async () =>
    new Response(
      JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        result: {
          value: [
            {
              account: {
                data: {
                  parsed: { info: { tokenAmount: { amount, decimals: 6 } } },
                },
              },
            },
          ],
        },
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
}
