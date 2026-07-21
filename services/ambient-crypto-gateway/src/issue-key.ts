import { x402Client, x402HTTPClient, wrapFetchWithPayment } from "@x402/fetch";
import { createSIWxClientHook, type SolanaSigner } from "@x402/extensions/sign-in-with-x";
import { ExactSvmScheme } from "@x402/svm/exact/client";

import { loadWalletFile, writeSecretFile } from "./wallet-file.js";

const gatewayUrl = requireEnv("PFT_AMBIENT_GATEWAY_URL");
const walletPath = requireEnv("PFT_SOLANA_WALLET_FILE");
const keyOutputPath = requireEnv("PFT_AMBIENT_KEY_OUTPUT");
const signer = await loadWalletFile(walletPath);
const client = new x402Client().register("solana:*", new ExactSvmScheme(signer));
const httpClient = new x402HTTPClient(client).onPaymentRequired(
  createSIWxClientHook(signer as unknown as SolanaSigner),
);
const paidFetch = wrapFetchWithPayment(fetch, httpClient);

const response = await paidFetch(new URL("/v1/keys", gatewayUrl), {
  method: "POST",
  headers: { Accept: "application/json", "Content-Type": "application/json" },
  body: "{}",
});
if (!response.ok) throw new Error(`API key creation failed with HTTP ${response.status}`);
const key = (await response.json()) as { id?: string; key?: string };
if (!key.id || !key.key) throw new Error("key response was incomplete");
await writeSecretFile(keyOutputPath, key);
process.stdout.write(`API key written to ${keyOutputPath}\n`);

function requireEnv(name: string): string {
  const value = process.env[name]?.trim();
  if (!value) throw new Error(`${name} is required`);
  return value;
}
