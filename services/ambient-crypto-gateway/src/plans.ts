export const PLAN_IDS = ["starter", "basic", "power", "pro"] as const;

export type PlanId = (typeof PLAN_IDS)[number];

export interface Plan {
  id: PlanId;
  revision: number;
  priceUsdc: string;
  amountAtomic: string;
  termMonths: 1;
  weeklyTokenLimit: number;
  monthlyTokenLimit: number;
  maxOutputTokens: number;
  modelAllowlist: readonly string[];
}

export const PLAN_MODELS = ["z-ai/glm-5.2", "moonshotai/kimi-k2.7-code"] as const;
export const SOLANA_USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

export const PLANS: Readonly<Record<PlanId, Plan>> = {
  starter: plan("starter", "1", 250_000, 1_000_000),
  basic: plan("basic", "20", 5_000_000, 20_000_000),
  power: plan("power", "50", 12_500_000, 50_000_000),
  pro: plan("pro", "200", 50_000_000, 200_000_000),
};

function plan(
  id: PlanId,
  priceUsdc: string,
  weeklyTokenLimit: number,
  monthlyTokenLimit: number,
): Plan {
  return {
    id,
    revision: 1,
    priceUsdc,
    amountAtomic: `${priceUsdc}000000`,
    termMonths: 1,
    weeklyTokenLimit,
    monthlyTokenLimit,
    maxOutputTokens: 32_768,
    modelAllowlist: PLAN_MODELS,
  };
}

export function parsePlanId(value: string): PlanId | undefined {
  return PLAN_IDS.find(planId => planId === value);
}

export function purchasePath(planId: PlanId): string {
  return `/v1/subscriptions/${planId}`;
}
