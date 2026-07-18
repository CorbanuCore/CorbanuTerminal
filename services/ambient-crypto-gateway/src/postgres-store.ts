import type { Pool, PoolClient } from "pg";

import { parsePlanId } from "./plans.js";
import {
  addUtcCalendarMonth,
  type CreatedApiKey,
  type GatewayStore,
  type Settlement,
  type SubscriptionPeriod,
} from "./store.js";
import { issueToken } from "./token.js";

const MAX_STACKED_MONTHS = 12;

interface PeriodRow {
  transaction: string;
  wallet_address: string;
  plan_id: string;
  starts_at: Date;
  ends_at: Date;
}

export class PostgresGatewayStore implements GatewayStore {
  constructor(private readonly pool: Pool) {}

  async initialize(): Promise<void> {
    await this.pool.query(`
      CREATE TABLE IF NOT EXISTS ambient_subscription_periods (
        transaction TEXT PRIMARY KEY,
        wallet_address TEXT NOT NULL,
        plan_id TEXT NOT NULL CHECK (plan_id IN ('starter', 'basic', 'power', 'pro')),
        network TEXT NOT NULL,
        amount_atomic TEXT NOT NULL,
        settled_at TIMESTAMPTZ NOT NULL,
        starts_at TIMESTAMPTZ NOT NULL,
        ends_at TIMESTAMPTZ NOT NULL,
        CHECK (ends_at > starts_at)
      );
      CREATE INDEX IF NOT EXISTS ambient_periods_wallet_end_idx
        ON ambient_subscription_periods (wallet_address, ends_at);
      CREATE TABLE IF NOT EXISTS ambient_api_keys (
        id UUID PRIMARY KEY,
        wallet_address TEXT NOT NULL,
        token_hash TEXT UNIQUE NOT NULL,
        display_prefix TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL,
        revoked_at TIMESTAMPTZ
      );
      CREATE INDEX IF NOT EXISTS ambient_keys_wallet_idx
        ON ambient_api_keys (wallet_address);
      CREATE TABLE IF NOT EXISTS ambient_used_siwx_nonces (
        nonce TEXT PRIMARY KEY,
        used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );
    `);
  }

  async recordSettlement(settlement: Settlement): Promise<SubscriptionPeriod> {
    const client = await this.pool.connect();
    try {
      await client.query("BEGIN");
      await advisoryLock(client, `transaction:${settlement.transaction}`);
      const existing = await findTransaction(client, settlement.transaction);
      if (existing) {
        assertSamePurchase(existing, settlement);
        await client.query("COMMIT");
        return rowToPeriod(existing);
      }

      const walletAddress = normalizeAddress(settlement.walletAddress);
      await advisoryLock(client, `wallet:${walletAddress}`);
      const latest = await client.query<Pick<PeriodRow, "ends_at">>(
        `SELECT ends_at
           FROM ambient_subscription_periods
          WHERE wallet_address = $1 AND ends_at > $2
          ORDER BY ends_at DESC
          LIMIT 1`,
        [walletAddress, settlement.settledAt],
      );
      const latestEnd = latest.rows[0]?.ends_at;
      const startsAt = latestEnd && latestEnd > settlement.settledAt
        ? latestEnd
        : settlement.settledAt;
      const endsAt = addUtcCalendarMonth(startsAt);
      let maximumEnd = new Date(settlement.settledAt);
      for (let month = 0; month < MAX_STACKED_MONTHS; month += 1) {
        maximumEnd = addUtcCalendarMonth(maximumEnd);
      }
      if (endsAt > maximumEnd) {
        throw new Error("subscription cannot be stacked beyond 12 months");
      }

      const inserted = await client.query<PeriodRow>(
        `INSERT INTO ambient_subscription_periods
           (transaction, wallet_address, plan_id, network, amount_atomic,
            settled_at, starts_at, ends_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING transaction, wallet_address, plan_id, starts_at, ends_at`,
        [
          settlement.transaction,
          walletAddress,
          settlement.planId,
          settlement.network,
          settlement.amountAtomic,
          settlement.settledAt,
          startsAt,
          endsAt,
        ],
      );
      const insertedRow = inserted.rows[0];
      if (!insertedRow) throw new Error("settlement insert returned no subscription period");
      await client.query("COMMIT");
      return rowToPeriod(insertedRow);
    } catch (error) {
      await client.query("ROLLBACK");
      throw error;
    } finally {
      client.release();
    }
  }

  async listPeriods(walletAddress: string, now: Date): Promise<SubscriptionPeriod[]> {
    const result = await this.pool.query<PeriodRow>(
      `SELECT transaction, wallet_address, plan_id, starts_at, ends_at
         FROM ambient_subscription_periods
        WHERE wallet_address = $1 AND ends_at > $2
        ORDER BY starts_at`,
      [normalizeAddress(walletAddress), now],
    );
    return result.rows.map(rowToPeriod);
  }

  async activePeriod(
    walletAddress: string,
    now: Date,
  ): Promise<SubscriptionPeriod | undefined> {
    const result = await this.pool.query<PeriodRow>(
      `SELECT transaction, wallet_address, plan_id, starts_at, ends_at
         FROM ambient_subscription_periods
        WHERE wallet_address = $1 AND starts_at <= $2 AND $2 < ends_at
        ORDER BY starts_at
        LIMIT 1`,
      [normalizeAddress(walletAddress), now],
    );
    return result.rows[0] && rowToPeriod(result.rows[0]);
  }

  async createApiKey(
    walletAddress: string,
    now: Date,
    pepper: string,
  ): Promise<CreatedApiKey> {
    const normalized = normalizeAddress(walletAddress);
    if (!(await this.activePeriod(normalized, now))) {
      throw new Error("an active subscription is required to create an API key");
    }
    const token = issueToken(pepper);
    await this.pool.query(
      `INSERT INTO ambient_api_keys
         (id, wallet_address, token_hash, display_prefix, created_at)
       VALUES ($1, $2, $3, $4, $5)`,
      [token.id, normalized, token.hash, token.displayPrefix, now],
    );
    return {
      id: token.id,
      key: token.plaintext,
      displayPrefix: token.displayPrefix,
      createdAt: new Date(now),
    };
  }

  async authenticateApiKey(hash: string, now: Date): Promise<SubscriptionPeriod | undefined> {
    const result = await this.pool.query<PeriodRow>(
      `SELECT p.transaction, p.wallet_address, p.plan_id, p.starts_at, p.ends_at
         FROM ambient_api_keys AS k
         JOIN ambient_subscription_periods AS p
           ON p.wallet_address = k.wallet_address
        WHERE k.token_hash = $1 AND k.revoked_at IS NULL
          AND p.starts_at <= $2 AND $2 < p.ends_at
        ORDER BY p.starts_at
        LIMIT 1`,
      [hash, now],
    );
    return result.rows[0] && rowToPeriod(result.rows[0]);
  }

  async revokeApiKey(walletAddress: string, keyId: string, now: Date): Promise<boolean> {
    const result = await this.pool.query(
      `UPDATE ambient_api_keys
          SET revoked_at = $3
        WHERE id = $1 AND wallet_address = $2 AND revoked_at IS NULL`,
      [keyId, normalizeAddress(walletAddress), now],
    );
    return result.rowCount === 1;
  }

  hasPaid(): boolean {
    return false;
  }

  recordPayment(): void {
    // Auth-only routes accept verified wallet signatures. Purchase reuse is
    // represented by durable subscription periods instead of SIWX resources.
  }

  async hasUsedNonce(nonce: string): Promise<boolean> {
    const result = await this.pool.query(
      `INSERT INTO ambient_used_siwx_nonces (nonce)
       VALUES ($1)
       ON CONFLICT (nonce) DO NOTHING
       RETURNING nonce`,
      [nonce],
    );
    return result.rowCount === 0;
  }

  async recordNonce(): Promise<void> {
    // hasUsedNonce atomically reserves the nonce after signature verification.
  }
}

async function advisoryLock(client: PoolClient, key: string): Promise<void> {
  await client.query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))", [key]);
}

async function findTransaction(
  client: PoolClient,
  transaction: string,
): Promise<PeriodRow | undefined> {
  const result = await client.query<PeriodRow>(
    `SELECT transaction, wallet_address, plan_id, starts_at, ends_at
       FROM ambient_subscription_periods
      WHERE transaction = $1`,
    [transaction],
  );
  return result.rows[0];
}

function assertSamePurchase(row: PeriodRow, settlement: Settlement): void {
  if (
    row.wallet_address !== normalizeAddress(settlement.walletAddress) ||
    row.plan_id !== settlement.planId
  ) {
    throw new Error("settlement transaction is already bound to another purchase");
  }
}

function rowToPeriod(row: PeriodRow): SubscriptionPeriod {
  const planId = parsePlanId(row.plan_id);
  if (!planId) throw new Error(`database contains unsupported plan: ${row.plan_id}`);
  return {
    transaction: row.transaction,
    walletAddress: row.wallet_address,
    planId,
    startsAt: new Date(row.starts_at),
    endsAt: new Date(row.ends_at),
  };
}

function normalizeAddress(address: string): string {
  return address.trim();
}
