import { randomUUID } from "node:crypto";

import type { SIWxStorage } from "@x402/extensions/sign-in-with-x";

import { PLANS, type PlanId } from "./plans.js";
import { issueToken } from "./token.js";
import type { ActualUsage } from "./usage.js";

const MAX_STACKED_MONTHS = 12;
const WEEK_MS = 7 * 24 * 60 * 60 * 1_000;

export interface Settlement {
  transaction: string;
  walletAddress: string;
  planId: PlanId;
  network: string;
  amountAtomic: string;
  settledAt: Date;
}

export interface SubscriptionPeriod {
  transaction: string;
  walletAddress: string;
  planId: PlanId;
  planRevision: number;
  startsAt: Date;
  endsAt: Date;
  monthlyLimitTokens: number;
  monthlyUsedTokens: number;
  monthlyReservedTokens: number;
}

export interface WeeklyUsage {
  startsAt: Date;
  endsAt: Date;
  limitTokens: number;
  usedTokens: number;
  reservedTokens: number;
}

export interface UsageSnapshot {
  period: SubscriptionPeriod;
  weekly: WeeklyUsage;
  monthlyRemainingTokens: number;
  weeklyRemainingTokens: number;
}

export interface UsageReservation extends UsageSnapshot {
  id: string;
  requestId: string;
  model: string;
  reservedTokens: number;
  state: "reserved" | "settled" | "released";
  actualUsage?: ActualUsage;
  chargedTokens?: number;
}

export interface PlanLimitReached {
  kind: "limit";
  window: "weekly" | "monthly";
  limitTokens: number;
  usedTokens: number;
  reservedTokens: number;
  remainingTokens: number;
  resetsAt: Date;
}

export type ReserveUsageResult =
  | { kind: "authorized"; reservation: UsageReservation }
  | PlanLimitReached;

export type UsageDisposition = "completed" | "rejected" | "ambiguous";

export interface ApiKeyRecord {
  id: string;
  walletAddress: string;
  hash: string;
  displayPrefix: string;
  createdAt: Date;
  revokedAt?: Date;
  lastUsedAt?: Date;
}

export interface CreatedApiKey {
  id: string;
  key: string;
  displayPrefix: string;
  createdAt: Date;
}

export interface ApiKeySummary {
  id: string;
  displayPrefix: string;
  createdAt: Date;
  revokedAt?: Date;
  lastUsedAt?: Date;
}

export interface AccountStatus extends UsageSnapshot {
  walletAddress: string;
  queuedPeriods: SubscriptionPeriod[];
}

export interface GatewayStore extends SIWxStorage {
  hasUsedNonce(nonce: string): boolean | Promise<boolean>;
  recordSettlement(settlement: Settlement): Promise<SubscriptionPeriod>;
  listPeriods(walletAddress: string, now: Date): Promise<SubscriptionPeriod[]>;
  activePeriod(walletAddress: string, now: Date): Promise<SubscriptionPeriod | undefined>;
  createApiKey(walletAddress: string, now: Date, pepper: string): Promise<CreatedApiKey>;
  authenticateApiKey(hash: string, now: Date): Promise<SubscriptionPeriod | undefined>;
  accountForApiKey(hash: string, now: Date): Promise<AccountStatus | undefined>;
  listApiKeys(walletAddress: string): Promise<ApiKeySummary[]>;
  reserveApiKeyUsage(
    hash: string,
    requestId: string,
    model: string,
    tokens: number,
    now: Date,
  ): Promise<ReserveUsageResult | undefined>;
  settleApiKeyUsage(
    reservationId: string,
    disposition: UsageDisposition,
    usage: ActualUsage | undefined,
    now: Date,
  ): Promise<UsageReservation | undefined>;
  revokeApiKey(walletAddress: string, keyId: string, now: Date): Promise<boolean>;
}

interface MemoryReservation extends UsageReservation {
  keyId: string;
}

export class InMemoryGatewayStore implements GatewayStore {
  private readonly periods: SubscriptionPeriod[] = [];
  private readonly apiKeys = new Map<string, ApiKeyRecord>();
  private readonly reservations = new Map<string, MemoryReservation>();
  private readonly usedNonces = new Set<string>();

  async recordSettlement(settlement: Settlement): Promise<SubscriptionPeriod> {
    const existing = this.periods.find(period => period.transaction === settlement.transaction);
    if (existing) {
      if (
        normalizeAddress(existing.walletAddress) !== normalizeAddress(settlement.walletAddress) ||
        existing.planId !== settlement.planId
      ) throw new Error("settlement transaction is already bound to another purchase");
      return clonePeriod(existing);
    }
    const walletAddress = normalizeAddress(settlement.walletAddress);
    const active = this.periods
      .filter(period => period.walletAddress === walletAddress && period.endsAt > settlement.settledAt)
      .sort((left, right) => left.endsAt.getTime() - right.endsAt.getTime());
    const latestEnd = active.at(-1)?.endsAt;
    const startsAt = latestEnd && latestEnd > settlement.settledAt ? latestEnd : settlement.settledAt;
    const endsAt = addUtcCalendarMonth(startsAt);
    if (endsAt > addUtcCalendarMonths(settlement.settledAt, MAX_STACKED_MONTHS)) {
      throw new Error("subscription cannot be stacked beyond 12 months");
    }
    const plan = PLANS[settlement.planId];
    const period: SubscriptionPeriod = {
      transaction: settlement.transaction,
      walletAddress,
      planId: settlement.planId,
      planRevision: plan.revision,
      startsAt: new Date(startsAt),
      endsAt,
      monthlyLimitTokens: plan.monthlyTokenLimit,
      monthlyUsedTokens: 0,
      monthlyReservedTokens: 0,
    };
    this.periods.push(period);
    return clonePeriod(period);
  }

  async listPeriods(walletAddress: string, now: Date): Promise<SubscriptionPeriod[]> {
    const normalized = normalizeAddress(walletAddress);
    return this.periods
      .filter(period => period.walletAddress === normalized && period.endsAt > now)
      .sort((left, right) => left.startsAt.getTime() - right.startsAt.getTime())
      .map(clonePeriod);
  }

  async activePeriod(walletAddress: string, now: Date): Promise<SubscriptionPeriod | undefined> {
    const normalized = normalizeAddress(walletAddress);
    const period = this.periods.find(
      candidate => candidate.walletAddress === normalized && candidate.startsAt <= now && now < candidate.endsAt,
    );
    return period && clonePeriod(period);
  }

  async createApiKey(walletAddress: string, now: Date, pepper: string): Promise<CreatedApiKey> {
    if (!(await this.activePeriod(walletAddress, now))) {
      throw new Error("an active subscription is required to create an API key");
    }
    const token = issueToken(pepper);
    this.apiKeys.set(token.id, {
      id: token.id,
      walletAddress: normalizeAddress(walletAddress),
      hash: token.hash,
      displayPrefix: token.displayPrefix,
      createdAt: new Date(now),
    });
    return { id: token.id, key: token.plaintext, displayPrefix: token.displayPrefix, createdAt: new Date(now) };
  }

  async authenticateApiKey(hash: string, now: Date): Promise<SubscriptionPeriod | undefined> {
    const key = this.keyForHash(hash);
    return key && this.activePeriod(key.walletAddress, now);
  }

  async accountForApiKey(hash: string, now: Date): Promise<AccountStatus | undefined> {
    const key = this.keyForHash(hash);
    if (!key) return undefined;
    const period = this.periodRef(key.walletAddress, now);
    if (!period) return undefined;
    const snapshot = this.snapshot(period, now);
    return {
      walletAddress: key.walletAddress,
      ...snapshot,
      queuedPeriods: this.periods
        .filter(candidate => candidate.walletAddress === key.walletAddress && candidate.startsAt >= period.endsAt)
        .map(clonePeriod),
    };
  }

  async listApiKeys(walletAddress: string): Promise<ApiKeySummary[]> {
    const normalized = normalizeAddress(walletAddress);
    return [...this.apiKeys.values()]
      .filter(key => key.walletAddress === normalized)
      .map(({ id, displayPrefix, createdAt, revokedAt, lastUsedAt }) => ({
        id,
        displayPrefix,
        createdAt: new Date(createdAt),
        revokedAt: revokedAt && new Date(revokedAt),
        lastUsedAt: lastUsedAt && new Date(lastUsedAt),
      }));
  }

  async reserveApiKeyUsage(
    hash: string,
    requestId: string,
    model: string,
    tokens: number,
    now: Date,
  ): Promise<ReserveUsageResult | undefined> {
    assertUsageTokens(tokens);
    const key = this.keyForHash(hash);
    if (!key) return undefined;
    const requestKey = `${key.walletAddress}:${requestId}`;
    const prior = [...this.reservations.values()].find(item => `${item.period.walletAddress}:${item.requestId}` === requestKey);
    if (prior) {
      if (prior.model !== model || prior.reservedTokens !== tokens) {
        throw new Error("client request ID was already used for a different request");
      }
      return { kind: "authorized", reservation: cloneReservation(prior) };
    }
    const period = this.periodRef(key.walletAddress, now);
    if (!period) return undefined;
    const weekly = this.weeklyRef(period, now);
    const monthlyRemaining = period.monthlyLimitTokens - period.monthlyUsedTokens - period.monthlyReservedTokens;
    const weeklyRemaining = weekly.limitTokens - weekly.usedTokens - weekly.reservedTokens;
    if (weeklyRemaining < tokens) return limit("weekly", weekly, weeklyRemaining);
    if (monthlyRemaining < tokens) {
      return {
        kind: "limit",
        window: "monthly",
        limitTokens: period.monthlyLimitTokens,
        usedTokens: period.monthlyUsedTokens,
        reservedTokens: period.monthlyReservedTokens,
        remainingTokens: Math.max(0, monthlyRemaining),
        resetsAt: new Date(period.endsAt),
      };
    }
    period.monthlyReservedTokens += tokens;
    weekly.reservedTokens += tokens;
    key.lastUsedAt = new Date(now);
    const reservation: MemoryReservation = {
      id: randomUUID(), requestId, model, reservedTokens: tokens, state: "reserved", keyId: key.id,
      period: clonePeriod(period),
      weekly,
      monthlyRemainingTokens: Math.max(0, monthlyRemaining - tokens),
      weeklyRemainingTokens: Math.max(0, weeklyRemaining - tokens),
    };
    this.reservations.set(reservation.id, reservation);
    return { kind: "authorized", reservation: cloneReservation(reservation) };
  }

  async settleApiKeyUsage(
    reservationId: string,
    disposition: UsageDisposition,
    usage: ActualUsage | undefined,
    now: Date,
  ): Promise<UsageReservation | undefined> {
    const reservation = this.reservations.get(reservationId);
    if (!reservation) return undefined;
    if (reservation.state !== "reserved") return cloneReservation(reservation);
    const period = this.periodRef(reservation.period.walletAddress, now) ??
      this.periods.find(item => item.transaction === reservation.period.transaction);
    if (!period) throw new Error("usage reservation lost its subscription period");
    period.monthlyReservedTokens -= reservation.reservedTokens;
    const completed = disposition === "completed" && usage !== undefined;
    const chargedTokens = completed ? usage.totalTokens : 0;
    period.monthlyUsedTokens += chargedTokens;
    reservation.state = completed ? "settled" : "released";
    reservation.actualUsage = usage;
    reservation.chargedTokens = chargedTokens;
    Object.assign(reservation, this.snapshot(period, now));
    return cloneReservation(reservation);
  }

  async revokeApiKey(walletAddress: string, keyId: string, now: Date): Promise<boolean> {
    const key = this.apiKeys.get(keyId);
    if (!key || key.walletAddress !== normalizeAddress(walletAddress) || key.revokedAt) return false;
    key.revokedAt = new Date(now);
    return true;
  }

  hasPaid(): boolean { return false; }
  recordPayment(): void {}
  hasUsedNonce(nonce: string): boolean {
    if (this.usedNonces.has(nonce)) return true;
    this.usedNonces.add(nonce);
    return false;
  }
  recordNonce(): void {}

  private keyForHash(hash: string): ApiKeyRecord | undefined {
    return [...this.apiKeys.values()].find(candidate => candidate.hash === hash && !candidate.revokedAt);
  }

  private periodRef(walletAddress: string, now: Date): SubscriptionPeriod | undefined {
    return this.periods.find(period => period.walletAddress === walletAddress && period.startsAt <= now && now < period.endsAt);
  }

  private weeklyRef(period: SubscriptionPeriod, now: Date): WeeklyUsage {
    const sequence = Math.max(0, Math.floor((now.getTime() - period.startsAt.getTime()) / WEEK_MS));
    const startsAt = new Date(period.startsAt.getTime() + sequence * WEEK_MS);
    const endsAt = new Date(Math.min(startsAt.getTime() + WEEK_MS, period.endsAt.getTime()));
    const relevant = [...this.reservations.values()].filter(item =>
      item.period.transaction === period.transaction && item.weekly.startsAt.getTime() === startsAt.getTime(),
    );
    return {
      startsAt,
      endsAt,
      limitTokens: PLANS[period.planId].weeklyTokenLimit,
      usedTokens: relevant.reduce((sum, item) => sum + (item.chargedTokens ?? 0), 0),
      reservedTokens: relevant.reduce((sum, item) => sum + (item.state === "reserved" ? item.reservedTokens : 0), 0),
    };
  }

  private snapshot(period: SubscriptionPeriod, now: Date): UsageSnapshot {
    const weekly = this.weeklyRef(period, now);
    return {
      period: clonePeriod(period), weekly,
      monthlyRemainingTokens: Math.max(0, period.monthlyLimitTokens - period.monthlyUsedTokens - period.monthlyReservedTokens),
      weeklyRemainingTokens: Math.max(0, weekly.limitTokens - weekly.usedTokens - weekly.reservedTokens),
    };
  }
}

export function addUtcCalendarMonth(value: Date): Date {
  const originalDay = value.getUTCDate();
  const result = new Date(value);
  result.setUTCDate(1);
  result.setUTCMonth(result.getUTCMonth() + 1);
  const finalDay = new Date(Date.UTC(result.getUTCFullYear(), result.getUTCMonth() + 1, 0)).getUTCDate();
  result.setUTCDate(Math.min(originalDay, finalDay));
  return result;
}

function addUtcCalendarMonths(value: Date, months: number): Date {
  let result = new Date(value);
  for (let index = 0; index < months; index += 1) result = addUtcCalendarMonth(result);
  return result;
}

function normalizeAddress(address: string): string { return address.trim(); }

function clonePeriod(period: SubscriptionPeriod): SubscriptionPeriod {
  return { ...period, startsAt: new Date(period.startsAt), endsAt: new Date(period.endsAt) };
}

function cloneReservation<T extends UsageReservation>(reservation: T): T {
  return {
    ...reservation,
    period: clonePeriod(reservation.period),
    weekly: { ...reservation.weekly, startsAt: new Date(reservation.weekly.startsAt), endsAt: new Date(reservation.weekly.endsAt) },
    actualUsage: reservation.actualUsage && { ...reservation.actualUsage },
  };
}

function limit(window: "weekly", usage: WeeklyUsage, remaining: number): PlanLimitReached {
  return {
    kind: "limit", window, limitTokens: usage.limitTokens, usedTokens: usage.usedTokens,
    reservedTokens: usage.reservedTokens, remainingTokens: Math.max(0, remaining), resetsAt: new Date(usage.endsAt),
  };
}

export function assertUsageTokens(tokens: number): void {
  if (!Number.isSafeInteger(tokens) || tokens < 1) {
    throw new Error("usage tokens must be a positive safe integer");
  }
}
