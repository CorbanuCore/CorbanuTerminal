import { base58 } from "@scure/base";

import type { Pool } from "pg";

import { PLAN_IDS, type PlanId } from "./plans.js";

export type WalletClassification = "customer" | "internal";
export type AccountingClassification = WalletClassification | "unclassified";

export interface AccountingPlanSummary {
  classification: AccountingClassification;
  planId: PlanId;
  periods: number;
  activePeriods: number;
  queuedPeriods: number;
  expiredPeriods: number;
  grossUsdc: string;
  tokensUsed: string;
  tokenAllowance: string;
}

export interface UnclassifiedWalletSummary {
  walletAddress: string;
  periods: number;
  grossUsdc: string;
}

export interface AccountingReport {
  generatedAt: string;
  receiverAddress: string;
  receiverUsdc: string;
  recordedGrossUsdc: string;
  recognizedCustomerRevenueUsdc: string;
  internalVolumeUsdc: string;
  unclassifiedVolumeUsdc: string;
  plans: AccountingPlanSummary[];
  unclassifiedWallets: UnclassifiedWalletSummary[];
}

interface AccountingRow {
  classification: AccountingClassification;
  plan_id: string;
  periods: string;
  active_periods: string;
  queued_periods: string;
  expired_periods: string;
  gross_atomic: string;
  tokens_used: string;
  token_allowance: string;
}

interface UnclassifiedWalletRow {
  wallet_address: string;
  periods: string;
  gross_atomic: string;
}

interface SolanaTokenAccountsResponse {
  error?: unknown;
  result?: {
    value?: Array<{
      account?: {
        data?: {
          parsed?: {
            info?: {
              tokenAmount?: { amount?: string; decimals?: number };
            };
          };
        };
      };
    }>;
  };
}

const USDC_DECIMALS = 6;

export async function loadAccountingReport(options: {
  pool: Pool;
  receiverAddress: string;
  usdcMint: string;
  solanaRpcUrl: string;
  now?: Date;
  fetch?: typeof globalThis.fetch;
}): Promise<AccountingReport> {
  const now = options.now ?? new Date();
  const [summaryResult, unclassifiedResult, receiverAtomic] = await Promise.all(
    [
      options.pool.query<AccountingRow>(
        `SELECT COALESCE(w.classification, 'unclassified') AS classification,
        p.plan_id, COUNT(*) AS periods,
        COUNT(*) FILTER (WHERE p.starts_at <= $1 AND $1 < p.ends_at) AS active_periods,
        COUNT(*) FILTER (WHERE p.starts_at > $1) AS queued_periods,
        COUNT(*) FILTER (WHERE p.ends_at <= $1) AS expired_periods,
        SUM(p.amount_atomic::NUMERIC)::TEXT AS gross_atomic,
        SUM(p.monthly_used_tokens)::TEXT AS tokens_used,
        SUM(p.monthly_limit_tokens)::TEXT AS token_allowance
       FROM ambient_subscription_periods p
       LEFT JOIN ambient_accounting_wallets w ON w.wallet_address = p.wallet_address
       GROUP BY COALESCE(w.classification, 'unclassified'), p.plan_id
       ORDER BY CASE COALESCE(w.classification, 'unclassified')
         WHEN 'customer' THEN 1 WHEN 'internal' THEN 2 ELSE 3 END, p.plan_id`,
        [now],
      ),
      options.pool.query<UnclassifiedWalletRow>(
        `SELECT p.wallet_address, COUNT(*) AS periods,
        SUM(p.amount_atomic::NUMERIC)::TEXT AS gross_atomic
       FROM ambient_subscription_periods p
       LEFT JOIN ambient_accounting_wallets w ON w.wallet_address = p.wallet_address
       WHERE w.wallet_address IS NULL
       GROUP BY p.wallet_address ORDER BY MIN(p.settled_at)`,
      ),
      receiverUsdcAtomic(
        options.receiverAddress,
        options.usdcMint,
        options.solanaRpcUrl,
        options.fetch ?? globalThis.fetch,
      ),
    ],
  );

  const plans = summaryResult.rows.map(rowToSummary);
  const customerAtomic = sumByClassification(plans, "customer");
  const internalAtomic = sumByClassification(plans, "internal");
  const unclassifiedAtomic = sumByClassification(plans, "unclassified");
  return {
    generatedAt: now.toISOString(),
    receiverAddress: options.receiverAddress,
    receiverUsdc: formatUsdc(receiverAtomic),
    recordedGrossUsdc: formatUsdc(
      customerAtomic + internalAtomic + unclassifiedAtomic,
    ),
    recognizedCustomerRevenueUsdc: formatUsdc(customerAtomic),
    internalVolumeUsdc: formatUsdc(internalAtomic),
    unclassifiedVolumeUsdc: formatUsdc(unclassifiedAtomic),
    plans,
    unclassifiedWallets: unclassifiedResult.rows.map((row) => ({
      walletAddress: row.wallet_address,
      periods: integer(row.periods, "period count"),
      grossUsdc: formatUsdc(atomic(row.gross_atomic, "wallet gross")),
    })),
  };
}

export async function setWalletClassification(
  pool: Pool,
  walletAddress: string,
  classification: WalletClassification | "unclassified",
  label: string | undefined,
  now = new Date(),
): Promise<void> {
  const wallet = normalizeSolanaAddress(walletAddress);
  if (classification === "unclassified") {
    await pool.query(
      "DELETE FROM ambient_accounting_wallets WHERE wallet_address=$1",
      [wallet],
    );
    return;
  }
  const normalizedLabel = label?.trim() ?? "";
  if (!normalizedLabel || normalizedLabel.length > 120) {
    throw new Error(
      "customer and internal classifications require a label of 1 through 120 characters",
    );
  }
  await pool.query(
    `INSERT INTO ambient_accounting_wallets(wallet_address,classification,label,updated_at)
     VALUES($1,$2,$3,$4)
     ON CONFLICT(wallet_address) DO UPDATE SET
       classification=EXCLUDED.classification,label=EXCLUDED.label,updated_at=EXCLUDED.updated_at`,
    [wallet, classification, normalizedLabel, now],
  );
}

export function renderAccountingReport(report: AccountingReport): string {
  const lines = [
    "Corbanu Terminal Plan accounting",
    `As of: ${report.generatedAt}`,
    `Receiver: ${report.receiverAddress}`,
    `Receiver balance: ${report.receiverUsdc} USDC`,
    `Recorded gross volume: ${report.recordedGrossUsdc} USDC`,
    `Recognized customer revenue: ${report.recognizedCustomerRevenueUsdc} USDC`,
    `Internal qualification volume: ${report.internalVolumeUsdc} USDC`,
    `Unclassified volume: ${report.unclassifiedVolumeUsdc} USDC`,
    "",
    "Classification  Plan      Periods  Active  Queued  Expired  Gross USDC  Tokens used / allowance",
  ];
  for (const plan of report.plans) {
    lines.push(
      `${plan.classification.padEnd(15)} ${plan.planId.padEnd(9)} ${String(plan.periods).padStart(7)}  ` +
        `${String(plan.activePeriods).padStart(6)}  ${String(plan.queuedPeriods).padStart(6)}  ` +
        `${String(plan.expiredPeriods).padStart(7)}  ${plan.grossUsdc.padStart(10)}  ` +
        `${plan.tokensUsed} / ${plan.tokenAllowance}`,
    );
  }
  if (report.unclassifiedWallets.length > 0) {
    lines.push("", "Unclassified wallets (not counted as customer revenue):");
    for (const wallet of report.unclassifiedWallets) {
      lines.push(
        `- ${wallet.walletAddress}: ${wallet.grossUsdc} USDC across ${wallet.periods} period(s)`,
      );
    }
  }
  return `${lines.join("\n")}\n`;
}

async function receiverUsdcAtomic(
  receiverAddress: string,
  usdcMint: string,
  solanaRpcUrl: string,
  fetchImpl: typeof globalThis.fetch,
): Promise<bigint> {
  normalizeSolanaAddress(receiverAddress);
  normalizeSolanaAddress(usdcMint);
  const response = await fetchImpl(solanaRpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getTokenAccountsByOwner",
      params: [
        receiverAddress,
        { mint: usdcMint },
        { encoding: "jsonParsed", commitment: "confirmed" },
      ],
    }),
  });
  if (!response.ok)
    throw new Error(`Solana RPC returned HTTP ${response.status}`);
  const body = (await response.json()) as SolanaTokenAccountsResponse;
  if (body.error || !Array.isArray(body.result?.value)) {
    throw new Error("Solana RPC did not return receiver token accounts");
  }
  return body.result.value.reduce((sum, item) => {
    const tokenAmount = item.account?.data?.parsed?.info?.tokenAmount;
    if (tokenAmount?.decimals !== USDC_DECIMALS || !tokenAmount.amount) {
      throw new Error("Solana RPC returned a malformed USDC token balance");
    }
    return sum + atomic(tokenAmount.amount, "receiver balance");
  }, 0n);
}

function rowToSummary(row: AccountingRow): AccountingPlanSummary {
  if (
    row.classification !== "customer" &&
    row.classification !== "internal" &&
    row.classification !== "unclassified"
  ) {
    throw new Error(
      `database contains unsupported accounting classification: ${row.classification}`,
    );
  }
  if (!PLAN_IDS.includes(row.plan_id as PlanId)) {
    throw new Error(`database contains unsupported plan: ${row.plan_id}`);
  }
  return {
    classification: row.classification,
    planId: row.plan_id as PlanId,
    periods: integer(row.periods, "period count"),
    activePeriods: integer(row.active_periods, "active period count"),
    queuedPeriods: integer(row.queued_periods, "queued period count"),
    expiredPeriods: integer(row.expired_periods, "expired period count"),
    grossUsdc: formatUsdc(atomic(row.gross_atomic, "plan gross")),
    tokensUsed: integerString(row.tokens_used, "tokens used"),
    tokenAllowance: integerString(row.token_allowance, "token allowance"),
  };
}

function sumByClassification(
  plans: AccountingPlanSummary[],
  classification: AccountingClassification,
): bigint {
  return plans
    .filter((plan) => plan.classification === classification)
    .reduce((sum, plan) => sum + usdcToAtomic(plan.grossUsdc), 0n);
}

function normalizeSolanaAddress(value: string): string {
  const normalized = value.trim();
  let decoded: Uint8Array;
  try {
    decoded = base58.decode(normalized);
  } catch {
    throw new Error("wallet address must be valid base58");
  }
  if (decoded.length !== 32)
    throw new Error("wallet address must decode to 32 bytes");
  return normalized;
}

function atomic(value: string, name: string): bigint {
  if (!/^\d+$/.test(value))
    throw new Error(`${name} must be a non-negative integer`);
  return BigInt(value);
}

function integer(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 0)
    throw new Error(`${name} must be a non-negative safe integer`);
  return parsed;
}

function integerString(value: string, name: string): string {
  return atomic(value, name).toString();
}

function formatUsdc(value: bigint): string {
  const whole = value / 1_000_000n;
  const fraction = (value % 1_000_000n)
    .toString()
    .padStart(6, "0")
    .replace(/0+$/, "");
  return fraction ? `${whole}.${fraction}` : whole.toString();
}

function usdcToAtomic(value: string): bigint {
  if (!/^\d+(?:\.\d{1,6})?$/.test(value))
    throw new Error("formatted USDC amount is invalid");
  const [whole = "0", fraction = ""] = value.split(".");
  return BigInt(whole) * 1_000_000n + BigInt(fraction.padEnd(6, "0"));
}
