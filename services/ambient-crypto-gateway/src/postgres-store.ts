import { randomUUID } from "node:crypto";

import type { Pool, PoolClient } from "pg";

import { PLANS, parsePlanId } from "./plans.js";
import {
  addUtcCalendarMonth,
  assertUsageTokens,
  type AccountStatus,
  type ApiKeySummary,
  type CreatedApiKey,
  type GatewayStore,
  type ReserveUsageResult,
  type Settlement,
  type SubscriptionPeriod,
  type UsageDisposition,
  type UsageReservation,
  type UsageSnapshot,
  type WeeklyUsage,
} from "./store.js";
import { issueToken } from "./token.js";
import type { ActualUsage } from "./usage.js";

const MAX_STACKED_MONTHS = 12;
const WEEK_MS = 7 * 24 * 60 * 60 * 1_000;

interface PeriodRow {
  transaction: string; wallet_address: string; plan_id: string; plan_revision: string;
  starts_at: Date; ends_at: Date; monthly_limit_tokens: string;
  monthly_used_tokens: string; monthly_reserved_tokens: string;
}
interface WeeklyRow {
  sequence: string; starts_at: Date; ends_at: Date; limit_tokens: string;
  used_tokens: string; reserved_tokens: string;
}
interface ReservationRow extends PeriodRow, WeeklyRow {
  id: string; request_id: string; model: string; reserved_tokens_request: string;
  state: "reserved" | "settled" | "released"; input_tokens: string | null;
  output_tokens: string | null; cached_input_tokens: string | null;
  reasoning_tokens: string | null; charged_tokens: string | null;
  usage_source: "upstream" | "estimated" | "reservation" | "legacy_unmetered" | null;
}

export class PostgresGatewayStore implements GatewayStore {
  constructor(private readonly pool: Pool) {}

  async initialize(): Promise<void> {
    await this.pool.query(`
      CREATE TABLE IF NOT EXISTS ambient_subscription_periods (
        transaction TEXT PRIMARY KEY,
        wallet_address TEXT NOT NULL,
        plan_id TEXT NOT NULL CHECK (plan_id IN ('starter', 'basic', 'power', 'pro')),
        plan_revision INTEGER NOT NULL DEFAULT 1,
        network TEXT NOT NULL,
        amount_atomic TEXT NOT NULL,
        settled_at TIMESTAMPTZ NOT NULL,
        starts_at TIMESTAMPTZ NOT NULL,
        ends_at TIMESTAMPTZ NOT NULL,
        monthly_limit_tokens BIGINT,
        monthly_used_tokens BIGINT NOT NULL DEFAULT 0,
        monthly_reserved_tokens BIGINT NOT NULL DEFAULT 0,
        allowance_units BIGINT,
        used_units BIGINT NOT NULL DEFAULT 0,
        CHECK (ends_at > starts_at)
      );
      ALTER TABLE ambient_subscription_periods ADD COLUMN IF NOT EXISTS plan_revision INTEGER NOT NULL DEFAULT 1;
      ALTER TABLE ambient_subscription_periods ADD COLUMN IF NOT EXISTS monthly_limit_tokens BIGINT;
      ALTER TABLE ambient_subscription_periods ADD COLUMN IF NOT EXISTS monthly_used_tokens BIGINT NOT NULL DEFAULT 0;
      ALTER TABLE ambient_subscription_periods ADD COLUMN IF NOT EXISTS monthly_reserved_tokens BIGINT NOT NULL DEFAULT 0;
      UPDATE ambient_subscription_periods SET monthly_limit_tokens = COALESCE(
        monthly_limit_tokens, allowance_units, CASE plan_id
          WHEN 'starter' THEN 1000000 WHEN 'basic' THEN 20000000
          WHEN 'power' THEN 50000000 WHEN 'pro' THEN 200000000 END);
      UPDATE ambient_subscription_periods SET monthly_used_tokens = GREATEST(monthly_used_tokens, used_units);
      ALTER TABLE ambient_subscription_periods ALTER COLUMN monthly_limit_tokens SET NOT NULL;
      CREATE INDEX IF NOT EXISTS ambient_periods_wallet_end_idx ON ambient_subscription_periods (wallet_address, ends_at);
      CREATE TABLE IF NOT EXISTS ambient_api_keys (
        id UUID PRIMARY KEY, wallet_address TEXT NOT NULL, token_hash TEXT UNIQUE NOT NULL,
        display_prefix TEXT NOT NULL, created_at TIMESTAMPTZ NOT NULL,
        last_used_at TIMESTAMPTZ, revoked_at TIMESTAMPTZ
      );
      ALTER TABLE ambient_api_keys ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMPTZ;
      CREATE INDEX IF NOT EXISTS ambient_keys_wallet_idx ON ambient_api_keys (wallet_address);
      CREATE TABLE IF NOT EXISTS ambient_weekly_windows (
        period_transaction TEXT NOT NULL REFERENCES ambient_subscription_periods(transaction),
        sequence INTEGER NOT NULL, starts_at TIMESTAMPTZ NOT NULL, ends_at TIMESTAMPTZ NOT NULL,
        limit_tokens BIGINT NOT NULL, used_tokens BIGINT NOT NULL DEFAULT 0,
        reserved_tokens BIGINT NOT NULL DEFAULT 0,
        PRIMARY KEY (period_transaction, sequence), CHECK (ends_at > starts_at)
      );
      CREATE TABLE IF NOT EXISTS ambient_inference_ledger (
        id UUID PRIMARY KEY, key_id UUID NOT NULL REFERENCES ambient_api_keys(id),
        wallet_address TEXT NOT NULL, period_transaction TEXT NOT NULL,
        weekly_sequence INTEGER NOT NULL, request_id TEXT NOT NULL, model TEXT NOT NULL,
        reserved_tokens BIGINT NOT NULL, input_tokens BIGINT, output_tokens BIGINT,
        cached_input_tokens BIGINT, reasoning_tokens BIGINT, charged_tokens BIGINT,
        usage_source TEXT,
        state TEXT NOT NULL CHECK (state IN ('reserved', 'settled', 'released')),
        created_at TIMESTAMPTZ NOT NULL, settled_at TIMESTAMPTZ,
        UNIQUE (wallet_address, request_id),
        FOREIGN KEY (period_transaction, weekly_sequence)
          REFERENCES ambient_weekly_windows(period_transaction, sequence)
      );
      CREATE TABLE IF NOT EXISTS ambient_used_siwx_nonces (
        nonce TEXT PRIMARY KEY, used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );
      ALTER TABLE ambient_inference_ledger ADD COLUMN IF NOT EXISTS usage_source TEXT;
      UPDATE ambient_inference_ledger SET usage_source='upstream'
        WHERE usage_source IS NULL AND input_tokens IS NOT NULL AND output_tokens IS NOT NULL;

      BEGIN;
      WITH refunds AS (
        SELECT period_transaction, SUM(charged_tokens)::BIGINT AS tokens
        FROM ambient_inference_ledger
        WHERE state='settled' AND usage_source IS NULL
          AND input_tokens IS NULL AND output_tokens IS NULL
          AND charged_tokens=reserved_tokens
        GROUP BY period_transaction
      )
      UPDATE ambient_subscription_periods p
        SET monthly_used_tokens=GREATEST(0,p.monthly_used_tokens-refunds.tokens)
        FROM refunds WHERE p.transaction=refunds.period_transaction;
      WITH refunds AS (
        SELECT period_transaction, weekly_sequence, SUM(charged_tokens)::BIGINT AS tokens
        FROM ambient_inference_ledger
        WHERE state='settled' AND usage_source IS NULL
          AND input_tokens IS NULL AND output_tokens IS NULL
          AND charged_tokens=reserved_tokens
        GROUP BY period_transaction,weekly_sequence
      )
      UPDATE ambient_weekly_windows w
        SET used_tokens=GREATEST(0,w.used_tokens-refunds.tokens)
        FROM refunds
        WHERE w.period_transaction=refunds.period_transaction
          AND w.sequence=refunds.weekly_sequence;
      UPDATE ambient_inference_ledger
        SET state='released',charged_tokens=0,usage_source='legacy_unmetered'
        WHERE state='settled' AND usage_source IS NULL
          AND input_tokens IS NULL AND output_tokens IS NULL
          AND charged_tokens=reserved_tokens;

      -- The gateway does not begin accepting traffic until initialize completes. Any
      -- reservation still open at process startup belongs to an ambiguously interrupted
      -- request whose owner can no longer settle it. Move the reservation to used capacity
      -- before serving traffic; releasing it would permit free generated output.
      WITH orphaned AS (
        SELECT period_transaction, SUM(reserved_tokens)::BIGINT AS tokens
        FROM ambient_inference_ledger
        WHERE state='reserved'
        GROUP BY period_transaction
      )
      UPDATE ambient_subscription_periods p
        SET monthly_reserved_tokens=GREATEST(0,p.monthly_reserved_tokens-orphaned.tokens),
            monthly_used_tokens=p.monthly_used_tokens+orphaned.tokens
        FROM orphaned WHERE p.transaction=orphaned.period_transaction;
      WITH orphaned AS (
        SELECT period_transaction, weekly_sequence, SUM(reserved_tokens)::BIGINT AS tokens
        FROM ambient_inference_ledger
        WHERE state='reserved'
        GROUP BY period_transaction,weekly_sequence
      )
      UPDATE ambient_weekly_windows w
        SET reserved_tokens=GREATEST(0,w.reserved_tokens-orphaned.tokens),
            used_tokens=w.used_tokens+orphaned.tokens
        FROM orphaned
        WHERE w.period_transaction=orphaned.period_transaction
          AND w.sequence=orphaned.weekly_sequence;
      UPDATE ambient_inference_ledger
        SET state='settled',charged_tokens=reserved_tokens,usage_source='reservation',settled_at=NOW()
        WHERE state='reserved';
      COMMIT;
    `);
  }

  async recordSettlement(settlement: Settlement): Promise<SubscriptionPeriod> {
    const client = await this.pool.connect();
    try {
      await client.query("BEGIN");
      await advisoryLock(client, `transaction:${settlement.transaction}`);
      const existing = await findTransaction(client, settlement.transaction);
      if (existing) {
        if (existing.wallet_address !== normalizeAddress(settlement.walletAddress) || existing.plan_id !== settlement.planId) {
          throw new Error("settlement transaction is already bound to another purchase");
        }
        await client.query("COMMIT");
        return rowToPeriod(existing);
      }
      const wallet = normalizeAddress(settlement.walletAddress);
      await advisoryLock(client, `wallet:${wallet}`);
      const latest = await client.query<{ ends_at: Date }>(
        "SELECT ends_at FROM ambient_subscription_periods WHERE wallet_address=$1 AND ends_at>$2 ORDER BY ends_at DESC LIMIT 1",
        [wallet, settlement.settledAt],
      );
      const startsAt = latest.rows[0]?.ends_at && latest.rows[0].ends_at > settlement.settledAt
        ? latest.rows[0].ends_at : settlement.settledAt;
      const endsAt = addUtcCalendarMonth(startsAt);
      let maximumEnd = new Date(settlement.settledAt);
      for (let month = 0; month < MAX_STACKED_MONTHS; month += 1) maximumEnd = addUtcCalendarMonth(maximumEnd);
      if (endsAt > maximumEnd) throw new Error("subscription cannot be stacked beyond 12 months");
      const plan = PLANS[settlement.planId];
      const inserted = await client.query<PeriodRow>(
        `INSERT INTO ambient_subscription_periods
          (transaction,wallet_address,plan_id,plan_revision,network,amount_atomic,settled_at,
           starts_at,ends_at,monthly_limit_tokens,monthly_used_tokens,monthly_reserved_tokens,
           allowance_units,used_units)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,0,0,$10,0) RETURNING ${PERIOD_COLUMNS}`,
        [settlement.transaction,wallet,settlement.planId,plan.revision,settlement.network,
         settlement.amountAtomic,settlement.settledAt,startsAt,endsAt,plan.monthlyTokenLimit],
      );
      await client.query("COMMIT");
      return rowToPeriod(required(inserted.rows[0], "settlement insert returned no period"));
    } catch (error) {
      await client.query("ROLLBACK"); throw error;
    } finally { client.release(); }
  }

  async listPeriods(walletAddress: string, now: Date): Promise<SubscriptionPeriod[]> {
    const result = await this.pool.query<PeriodRow>(
      `SELECT ${PERIOD_COLUMNS} FROM ambient_subscription_periods
       WHERE wallet_address=$1 AND ends_at>$2 ORDER BY starts_at`,
      [normalizeAddress(walletAddress), now],
    );
    return result.rows.map(rowToPeriod);
  }

  async activePeriod(walletAddress: string, now: Date): Promise<SubscriptionPeriod | undefined> {
    const result = await this.pool.query<PeriodRow>(
      `SELECT ${PERIOD_COLUMNS} FROM ambient_subscription_periods
       WHERE wallet_address=$1 AND starts_at<=$2 AND $2<ends_at ORDER BY starts_at LIMIT 1`,
      [normalizeAddress(walletAddress), now],
    );
    return result.rows[0] && rowToPeriod(result.rows[0]);
  }

  async createApiKey(walletAddress: string, now: Date, pepper: string): Promise<CreatedApiKey> {
    const wallet = normalizeAddress(walletAddress);
    if (!(await this.activePeriod(wallet, now))) throw new Error("an active subscription is required to create an API key");
    const token = issueToken(pepper);
    await this.pool.query(
      "INSERT INTO ambient_api_keys(id,wallet_address,token_hash,display_prefix,created_at) VALUES($1,$2,$3,$4,$5)",
      [token.id,wallet,token.hash,token.displayPrefix,now],
    );
    return { id: token.id, key: token.plaintext, displayPrefix: token.displayPrefix, createdAt: new Date(now) };
  }

  async authenticateApiKey(hash: string, now: Date): Promise<SubscriptionPeriod | undefined> {
    const result = await this.pool.query<PeriodRow>(
      `SELECT ${prefixColumns("p")} FROM ambient_api_keys k JOIN ambient_subscription_periods p ON p.wallet_address=k.wallet_address
       WHERE k.token_hash=$1 AND k.revoked_at IS NULL AND p.starts_at<=$2 AND $2<p.ends_at ORDER BY p.starts_at LIMIT 1`,
      [hash, now],
    );
    return result.rows[0] && rowToPeriod(result.rows[0]);
  }

  async accountForApiKey(hash: string, now: Date): Promise<AccountStatus | undefined> {
    const client = await this.pool.connect();
    try {
      const active = await activeForHash(client, hash, now, false);
      if (!active) return undefined;
      const period = rowToPeriod(active.period);
      const weekly = await ensureWeeklyWindow(client, active.period, now);
      const queued = await client.query<PeriodRow>(
        `SELECT ${PERIOD_COLUMNS} FROM ambient_subscription_periods WHERE wallet_address=$1 AND starts_at>=$2 ORDER BY starts_at`,
        [period.walletAddress, period.endsAt],
      );
      return { walletAddress: period.walletAddress, ...snapshot(period, weekly), queuedPeriods: queued.rows.map(rowToPeriod) };
    } finally { client.release(); }
  }

  async listApiKeys(walletAddress: string): Promise<ApiKeySummary[]> {
    const result = await this.pool.query<{
      id: string; display_prefix: string; created_at: Date; revoked_at: Date | null; last_used_at: Date | null;
    }>("SELECT id,display_prefix,created_at,revoked_at,last_used_at FROM ambient_api_keys WHERE wallet_address=$1 ORDER BY created_at", [normalizeAddress(walletAddress)]);
    return result.rows.map(row => ({
      id: row.id, displayPrefix: row.display_prefix, createdAt: new Date(row.created_at),
      revokedAt: row.revoked_at ? new Date(row.revoked_at) : undefined,
      lastUsedAt: row.last_used_at ? new Date(row.last_used_at) : undefined,
    }));
  }

  async reserveApiKeyUsage(
    hash: string, requestId: string, model: string, tokens: number, now: Date,
  ): Promise<ReserveUsageResult | undefined> {
    assertUsageTokens(tokens);
    const client = await this.pool.connect();
    try {
      await client.query("BEGIN");
      const active = await activeForHash(client, hash, now, true);
      if (!active) { await client.query("ROLLBACK"); return undefined; }
      const prior = await findReservation(client, active.period.wallet_address, requestId);
      if (prior) {
        if (prior.model !== model || units(prior.reserved_tokens_request) !== tokens) {
          throw new Error("client request ID was already used for a different request");
        }
        await client.query("COMMIT");
        return { kind: "authorized", reservation: reservationFromRow(prior) };
      }
      const weekly = await ensureWeeklyWindow(client, active.period, now, true);
      const period = rowToPeriod(active.period);
      const weeklyRemaining = units(weekly.limit_tokens) - units(weekly.used_tokens) - units(weekly.reserved_tokens);
      const monthlyRemaining = period.monthlyLimitTokens - period.monthlyUsedTokens - period.monthlyReservedTokens;
      if (weeklyRemaining < tokens) {
        await client.query("ROLLBACK");
        return { kind:"limit", window:"weekly", limitTokens:units(weekly.limit_tokens), usedTokens:units(weekly.used_tokens),
          reservedTokens:units(weekly.reserved_tokens), remainingTokens:Math.max(0,weeklyRemaining), resetsAt:new Date(weekly.ends_at) };
      }
      if (monthlyRemaining < tokens) {
        await client.query("ROLLBACK");
        return { kind:"limit", window:"monthly", limitTokens:period.monthlyLimitTokens, usedTokens:period.monthlyUsedTokens,
          reservedTokens:period.monthlyReservedTokens, remainingTokens:Math.max(0,monthlyRemaining), resetsAt:new Date(period.endsAt) };
      }
      const id = randomUUID();
      await client.query("UPDATE ambient_subscription_periods SET monthly_reserved_tokens=monthly_reserved_tokens+$2 WHERE transaction=$1", [period.transaction,tokens]);
      await client.query("UPDATE ambient_weekly_windows SET reserved_tokens=reserved_tokens+$3 WHERE period_transaction=$1 AND sequence=$2", [period.transaction,weekly.sequence,tokens]);
      await client.query("UPDATE ambient_api_keys SET last_used_at=$2 WHERE id=$1", [active.keyId,now]);
      await client.query(
        `INSERT INTO ambient_inference_ledger(id,key_id,wallet_address,period_transaction,weekly_sequence,
          request_id,model,reserved_tokens,state,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'reserved',$9)`,
        [id,active.keyId,period.walletAddress,period.transaction,weekly.sequence,requestId,model,tokens,now],
      );
      const saved = await findReservationById(client, id);
      await client.query("COMMIT");
      return { kind:"authorized", reservation:reservationFromRow(required(saved,"reservation insert failed")) };
    } catch (error) {
      await client.query("ROLLBACK"); throw error;
    } finally { client.release(); }
  }

  async settleApiKeyUsage(
    reservationId: string, disposition: UsageDisposition, usage: ActualUsage | undefined, now: Date,
  ): Promise<UsageReservation | undefined> {
    const client = await this.pool.connect();
    try {
      await client.query("BEGIN");
      const existing = await findReservationById(client, reservationId, true);
      if (!existing) { await client.query("ROLLBACK"); return undefined; }
      if (existing.state !== "reserved") { await client.query("COMMIT"); return reservationFromRow(existing); }
      const reserved = units(existing.reserved_tokens_request);
      const completed = disposition === "completed" && usage !== undefined;
      const ambiguous = disposition === "ambiguous";
      const charged = completed ? usage.totalTokens : ambiguous ? reserved : 0;
      const state = completed || ambiguous ? "settled" : "released";
      const usageSource = usage?.source ?? (ambiguous ? "reservation" : undefined);
      await client.query(
        `UPDATE ambient_subscription_periods SET monthly_reserved_tokens=monthly_reserved_tokens-$2,
          monthly_used_tokens=monthly_used_tokens+$3 WHERE transaction=$1`,
        [existing.transaction,reserved,charged],
      );
      await client.query(
        `UPDATE ambient_weekly_windows SET reserved_tokens=reserved_tokens-$3,used_tokens=used_tokens+$4
         WHERE period_transaction=$1 AND sequence=$2`,
        [existing.transaction,existing.sequence,reserved,charged],
      );
      await client.query(
        `UPDATE ambient_inference_ledger SET state=$2,input_tokens=$3,output_tokens=$4,
          cached_input_tokens=$5,reasoning_tokens=$6,charged_tokens=$7,settled_at=$8,
          usage_source=$9 WHERE id=$1`,
        [reservationId,state,usage?.inputTokens,usage?.outputTokens,usage?.cachedInputTokens,
         usage?.reasoningTokens,charged,now,usageSource],
      );
      const saved = await findReservationById(client, reservationId);
      await client.query("COMMIT");
      return reservationFromRow(required(saved,"settled reservation disappeared"));
    } catch (error) {
      await client.query("ROLLBACK"); throw error;
    } finally { client.release(); }
  }

  async revokeApiKey(walletAddress: string, keyId: string, now: Date): Promise<boolean> {
    const result = await this.pool.query(
      "UPDATE ambient_api_keys SET revoked_at=$3 WHERE id=$1 AND wallet_address=$2 AND revoked_at IS NULL",
      [keyId,normalizeAddress(walletAddress),now],
    );
    return result.rowCount === 1;
  }

  hasPaid(): boolean { return false; }
  recordPayment(): void {}
  async hasUsedNonce(nonce: string): Promise<boolean> {
    const result = await this.pool.query("INSERT INTO ambient_used_siwx_nonces(nonce) VALUES($1) ON CONFLICT(nonce) DO NOTHING RETURNING nonce", [nonce]);
    return result.rowCount === 0;
  }
  async recordNonce(): Promise<void> {}
}

const PERIOD_COLUMNS = "transaction,wallet_address,plan_id,plan_revision,starts_at,ends_at,monthly_limit_tokens,monthly_used_tokens,monthly_reserved_tokens";
function prefixColumns(prefix: string): string { return PERIOD_COLUMNS.split(",").map(column => `${prefix}.${column}`).join(","); }

async function activeForHash(client: PoolClient, hash: string, now: Date, lock: boolean) {
  const result = await client.query<PeriodRow & { key_id: string }>(
    `SELECT ${prefixColumns("p")},k.id key_id FROM ambient_api_keys k
     JOIN ambient_subscription_periods p ON p.wallet_address=k.wallet_address
     WHERE k.token_hash=$1 AND k.revoked_at IS NULL AND p.starts_at<=$2 AND $2<p.ends_at
     ORDER BY p.starts_at LIMIT 1${lock ? " FOR UPDATE OF p,k" : ""}`,[hash,now]);
  const row = result.rows[0];
  return row && { period: row, keyId: row.key_id };
}

async function ensureWeeklyWindow(client: PoolClient, period: PeriodRow, now: Date, lock=false): Promise<WeeklyRow> {
  const sequence = Math.max(0,Math.floor((now.getTime()-period.starts_at.getTime())/WEEK_MS));
  const startsAt = new Date(period.starts_at.getTime()+sequence*WEEK_MS);
  const endsAt = new Date(Math.min(startsAt.getTime()+WEEK_MS,period.ends_at.getTime()));
  const planId = required(parsePlanId(period.plan_id),"unsupported stored plan");
  await client.query(
    `INSERT INTO ambient_weekly_windows(period_transaction,sequence,starts_at,ends_at,limit_tokens)
     VALUES($1,$2,$3,$4,$5) ON CONFLICT(period_transaction,sequence) DO NOTHING`,
    [period.transaction,sequence,startsAt,endsAt,PLANS[planId].weeklyTokenLimit]);
  const result = await client.query<WeeklyRow>(
    `SELECT sequence,starts_at,ends_at,limit_tokens,used_tokens,reserved_tokens
     FROM ambient_weekly_windows WHERE period_transaction=$1 AND sequence=$2${lock ? " FOR UPDATE" : ""}`,
    [period.transaction,sequence]);
  return required(result.rows[0],"weekly window was not created");
}

async function findTransaction(client: PoolClient, transaction: string): Promise<PeriodRow|undefined> {
  return (await client.query<PeriodRow>(`SELECT ${PERIOD_COLUMNS} FROM ambient_subscription_periods WHERE transaction=$1`,[transaction])).rows[0];
}

const RESERVATION_SELECT = `SELECT l.id,l.request_id,l.model,l.reserved_tokens reserved_tokens_request,l.state,
 l.input_tokens,l.output_tokens,l.cached_input_tokens,l.reasoning_tokens,l.charged_tokens,l.usage_source,
 ${prefixColumns("p")},w.sequence,w.starts_at,w.ends_at,w.limit_tokens,w.used_tokens,w.reserved_tokens
 FROM ambient_inference_ledger l JOIN ambient_subscription_periods p ON p.transaction=l.period_transaction
 JOIN ambient_weekly_windows w ON w.period_transaction=l.period_transaction AND w.sequence=l.weekly_sequence`;

async function findReservation(client: PoolClient,wallet:string,requestId:string):Promise<ReservationRow|undefined>{
  return (await client.query<ReservationRow>(`${RESERVATION_SELECT} WHERE l.wallet_address=$1 AND l.request_id=$2`,[wallet,requestId])).rows[0];
}
async function findReservationById(client:PoolClient,id:string,lock=false):Promise<ReservationRow|undefined>{
  return (await client.query<ReservationRow>(`${RESERVATION_SELECT} WHERE l.id=$1${lock ? " FOR UPDATE OF l,p,w" : ""}`,[id])).rows[0];
}

function rowToPeriod(row:PeriodRow):SubscriptionPeriod {
  const planId=required(parsePlanId(row.plan_id),`unsupported stored plan: ${row.plan_id}`);
  return {transaction:row.transaction,walletAddress:row.wallet_address,planId,planRevision:units(row.plan_revision),
    startsAt:new Date(row.starts_at),endsAt:new Date(row.ends_at),monthlyLimitTokens:units(row.monthly_limit_tokens),
    monthlyUsedTokens:units(row.monthly_used_tokens),monthlyReservedTokens:units(row.monthly_reserved_tokens)};
}
function rowToWeekly(row:WeeklyRow):WeeklyUsage { return {startsAt:new Date(row.starts_at),endsAt:new Date(row.ends_at),
  limitTokens:units(row.limit_tokens),usedTokens:units(row.used_tokens),reservedTokens:units(row.reserved_tokens)}; }
function snapshot(period:SubscriptionPeriod,weeklyRow:WeeklyRow):UsageSnapshot { const weekly=rowToWeekly(weeklyRow); return {period,weekly,
  monthlyRemainingTokens:Math.max(0,period.monthlyLimitTokens-period.monthlyUsedTokens-period.monthlyReservedTokens),
  weeklyRemainingTokens:Math.max(0,weekly.limitTokens-weekly.usedTokens-weekly.reservedTokens)}; }
function reservationFromRow(row:ReservationRow):UsageReservation { const snap=snapshot(rowToPeriod(row),row); const input=nullableUnits(row.input_tokens);
  const output=nullableUnits(row.output_tokens); return {id:row.id,requestId:row.request_id,model:row.model,
    reservedTokens:units(row.reserved_tokens_request),state:row.state,chargedTokens:nullableUnits(row.charged_tokens),...snap,
    actualUsage:input===undefined||output===undefined?undefined:{source:row.usage_source==="estimated"?"estimated":"upstream",inputTokens:input,outputTokens:output,
      cachedInputTokens:nullableUnits(row.cached_input_tokens)??0,reasoningTokens:nullableUnits(row.reasoning_tokens)??0,totalTokens:input+output}}; }
function units(value:string|number):number { const parsed=Number(value); if(!Number.isSafeInteger(parsed)||parsed<0) throw new Error("database contains invalid token units"); return parsed; }
function nullableUnits(value:string|null):number|undefined { return value===null?undefined:units(value); }
function normalizeAddress(address:string):string{return address.trim();}
function required<T>(value:T|undefined,message:string):T { if(value===undefined) throw new Error(message); return value; }
async function advisoryLock(client:PoolClient,key:string):Promise<void>{await client.query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))",[key]);}
