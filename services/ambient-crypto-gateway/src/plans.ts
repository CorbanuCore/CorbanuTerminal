export const PLAN_IDS = ["starter", "basic", "power", "pro"] as const;

export type PlanId = (typeof PLAN_IDS)[number];

export interface Plan {
  id: PlanId;
  priceUsd: string;
  termMonths: 1;
}

export const PLANS: Readonly<Record<PlanId, Plan>> = {
  starter: { id: "starter", priceUsd: "$1", termMonths: 1 },
  basic: { id: "basic", priceUsd: "$20", termMonths: 1 },
  power: { id: "power", priceUsd: "$50", termMonths: 1 },
  pro: { id: "pro", priceUsd: "$200", termMonths: 1 },
};

export function parsePlanId(value: string): PlanId | undefined {
  return PLAN_IDS.find(planId => planId === value);
}

export function purchasePath(planId: PlanId): string {
  return `/v1/subscriptions/${planId}`;
}
