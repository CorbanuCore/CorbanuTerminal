import { writeFile } from "node:fs/promises";

import { x402Client, x402HTTPClient, wrapFetchWithPayment } from "@x402/fetch";
import { createSIWxClientHook, type SolanaSigner } from "@x402/extensions/sign-in-with-x";
import { ExactSvmScheme } from "@x402/svm/exact/client";

import { parsePlanId } from "./plans.js";
import { preflightSubscriptionPayment } from "./payment-preflight.js";
import { loadWalletFile, writeSecretFile } from "./wallet-file.js";

const gatewayUrl = requireEnv("PFT_AMBIENT_GATEWAY_URL");
const walletPath = requireEnv("PFT_SOLANA_WALLET_FILE");
const keyOutputPath = requireEnv("PFT_AMBIENT_KEY_OUTPUT");
const receiptOutputPath = requireEnv("PFT_AMBIENT_RECEIPT_OUTPUT");
const planId = parsePlanId(requireEnv("PFT_AMBIENT_PLAN"));
if (!planId) throw new Error("PFT_AMBIENT_PLAN must be starter, basic, power, or pro");

const signer = await loadWalletFile(walletPath);
const client = new x402Client().register("solana:*", new ExactSvmScheme(signer));
const httpClient = new x402HTTPClient(client).onPaymentRequired(
  createSIWxClientHook(signer as unknown as SolanaSigner),
);
const paidFetch = wrapFetchWithPayment(fetch, httpClient);
const purchaseUrl = new URL(`/v1/subscriptions/${planId}`, gatewayUrl);

process.stdout.write(`Checking ${planId} payment requirement and payer balance…\n`);
const requirement = await preflightSubscriptionPayment(
  purchaseUrl,
  signer.address,
  process.env.PFT_SOLANA_RPC_URL,
);
process.stdout.write(
  `Payer balance covers ${requirement.amount} atomic units; submitting signed payment…\n`,
);
const purchaseResponse = await paidFetch(
  purchaseUrl,
  { method: "POST", headers: { Accept: "application/json" } },
);
if (!purchaseResponse.ok) throw await responseError("subscription payment", purchaseResponse);
const settlement = httpClient.getPaymentSettleResponse(name => purchaseResponse.headers.get(name));
if (!settlement?.success || !settlement.transaction) {
  throw new Error("subscription response did not contain a successful settlement receipt");
}

process.stdout.write("Payment settled; proving wallet ownership and creating API key…\n");
const keyResponse = await paidFetch(new URL("/v1/keys", gatewayUrl), {
  method: "POST",
  headers: { Accept: "application/json", "Content-Type": "application/json" },
  body: "{}",
});
if (!keyResponse.ok) throw await responseError("API key creation", keyResponse);
const key = (await keyResponse.json()) as {
  id?: string;
  key?: string;
  displayPrefix?: string;
  createdAt?: string;
};
if (!key.id || !key.key) throw new Error("key response was incomplete");

await writeSecretFile(keyOutputPath, key);
await writeFile(
  receiptOutputPath,
  `${JSON.stringify({ planId, payer: signer.address, settlement }, null, 2)}\n`,
  { encoding: "utf8", flag: "wx", mode: 0o600 },
);
process.stdout.write(`Purchase settled as ${settlement.transaction}\n`);
process.stdout.write(`API key written to ${keyOutputPath}; receipt written to ${receiptOutputPath}\n`);

function requireEnv(name: string): string {
  const value = process.env[name]?.trim();
  if (!value) throw new Error(`${name} is required`);
  return value;
}

async function responseError(operation: string, response: Response): Promise<Error> {
  const body = await response.text();
  return new Error(`${operation} failed with HTTP ${response.status}: ${body.slice(0, 500)}`);
}
