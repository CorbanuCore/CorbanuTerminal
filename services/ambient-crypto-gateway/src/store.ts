import type { SIWxStorage } from "@x402/extensions/sign-in-with-x";

import type { PlanId } from "./plans.js";
import { issueToken } from "./token.js";

const MAX_STACKED_MONTHS = 12;

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
  startsAt: Date;
  endsAt: Date;
}

export interface ApiKeyRecord {
  id: string;
  walletAddress: string;
  hash: string;
  displayPrefix: string;
  createdAt: Date;
  revokedAt?: Date;
}

export interface CreatedApiKey {
  id: string;
  key: string;
  displayPrefix: string;
  createdAt: Date;
}

export interface GatewayStore extends SIWxStorage {
  recordSettlement(settlement: Settlement): Promise<SubscriptionPeriod>;
  listPeriods(walletAddress: string, now: Date): Promise<SubscriptionPeriod[]>;
  activePeriod(walletAddress: string, now: Date): Promise<SubscriptionPeriod | undefined>;
  createApiKey(walletAddress: string, now: Date, pepper: string): Promise<CreatedApiKey>;
  authenticateApiKey(hash: string, now: Date): Promise<SubscriptionPeriod | undefined>;
  revokeApiKey(walletAddress: string, keyId: string, now: Date): Promise<boolean>;
}

export class InMemoryGatewayStore implements GatewayStore {
  private readonly periods: SubscriptionPeriod[] = [];
  private readonly apiKeys = new Map<string, ApiKeyRecord>();
  private readonly usedNonces = new Set<string>();

  async recordSettlement(settlement: Settlement): Promise<SubscriptionPeriod> {
    const existing = this.periods.find(period => period.transaction === settlement.transaction);
    if (existing) {
      if (
        normalizeAddress(existing.walletAddress) !== normalizeAddress(settlement.walletAddress) ||
        existing.planId !== settlement.planId
      ) {
        throw new Error("settlement transaction is already bound to another purchase");
      }
      return clonePeriod(existing);
    }

    const walletAddress = normalizeAddress(settlement.walletAddress);
    const currentPeriods = this.periods
      .filter(period => normalizeAddress(period.walletAddress) === walletAddress)
      .filter(period => period.endsAt > settlement.settledAt)
      .sort((left, right) => left.startsAt.getTime() - right.startsAt.getTime());
    const latestEnd = currentPeriods.at(-1)?.endsAt;
    const startsAt = latestEnd && latestEnd > settlement.settledAt ? latestEnd : settlement.settledAt;
    const endsAt = addUtcCalendarMonth(startsAt);
    const maximumEnd = addUtcCalendarMonths(settlement.settledAt, MAX_STACKED_MONTHS);
    if (endsAt > maximumEnd) {
      throw new Error("subscription cannot be stacked beyond 12 months");
    }

    const period: SubscriptionPeriod = {
      transaction: settlement.transaction,
      walletAddress,
      planId: settlement.planId,
      startsAt: new Date(startsAt),
      endsAt,
    };
    this.periods.push(period);
    return clonePeriod(period);
  }

  async listPeriods(walletAddress: string, now: Date): Promise<SubscriptionPeriod[]> {
    const normalized = normalizeAddress(walletAddress);
    return this.periods
      .filter(period => normalizeAddress(period.walletAddress) === normalized)
      .filter(period => period.endsAt > now)
      .sort((left, right) => left.startsAt.getTime() - right.startsAt.getTime())
      .map(clonePeriod);
  }

  async activePeriod(
    walletAddress: string,
    now: Date,
  ): Promise<SubscriptionPeriod | undefined> {
    const normalized = normalizeAddress(walletAddress);
    const period = this.periods.find(
      candidate =>
        normalizeAddress(candidate.walletAddress) === normalized &&
        candidate.startsAt <= now &&
        now < candidate.endsAt,
    );
    return period && clonePeriod(period);
  }

  async createApiKey(
    walletAddress: string,
    now: Date,
    pepper: string,
  ): Promise<CreatedApiKey> {
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
    return {
      id: token.id,
      key: token.plaintext,
      displayPrefix: token.displayPrefix,
      createdAt: new Date(now),
    };
  }

  async authenticateApiKey(hash: string, now: Date): Promise<SubscriptionPeriod | undefined> {
    const record = [...this.apiKeys.values()].find(
      candidate => candidate.hash === hash && !candidate.revokedAt,
    );
    return record && this.activePeriod(record.walletAddress, now);
  }

  async revokeApiKey(walletAddress: string, keyId: string, now: Date): Promise<boolean> {
    const record = this.apiKeys.get(keyId);
    if (
      !record ||
      normalizeAddress(record.walletAddress) !== normalizeAddress(walletAddress) ||
      record.revokedAt
    ) {
      return false;
    }
    record.revokedAt = new Date(now);
    return true;
  }

  hasPaid(): boolean {
    return false;
  }

  recordPayment(): void {
    // Purchase routes are never reusable through SIWX. Settlement is recorded
    // by transaction ID through recordSettlement instead.
  }

  hasUsedNonce(nonce: string): boolean {
    if (this.usedNonces.has(nonce)) return true;
    this.usedNonces.add(nonce);
    return false;
  }

  recordNonce(): void {
    // hasUsedNonce reserves the nonce atomically after signature verification,
    // preventing two concurrent requests from both passing the replay check.
  }
}

export function addUtcCalendarMonth(value: Date): Date {
  const originalDay = value.getUTCDate();
  const result = new Date(value);
  result.setUTCDate(1);
  result.setUTCMonth(result.getUTCMonth() + 1);
  const finalDay = new Date(
    Date.UTC(result.getUTCFullYear(), result.getUTCMonth() + 1, 0),
  ).getUTCDate();
  result.setUTCDate(Math.min(originalDay, finalDay));
  return result;
}

function addUtcCalendarMonths(value: Date, months: number): Date {
  let result = new Date(value);
  for (let index = 0; index < months; index += 1) {
    result = addUtcCalendarMonth(result);
  }
  return result;
}

function normalizeAddress(address: string): string {
  return address.trim();
}

function clonePeriod(period: SubscriptionPeriod): SubscriptionPeriod {
  return {
    ...period,
    startsAt: new Date(period.startsAt),
    endsAt: new Date(period.endsAt),
  };
}
