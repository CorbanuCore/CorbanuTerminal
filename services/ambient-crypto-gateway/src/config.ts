import { base58 } from "@scure/base";

import { readExclusiveSecret } from "./secret-file.js";

export const SOLANA_MAINNET = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
export const SOLANA_DEVNET = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";

export interface GatewayConfig {
  host: string;
  port: number;
  databaseUrl: string;
  tokenPepper: string;
  ambientApiKey: string;
  ambientBaseUrl: URL;
  network: typeof SOLANA_MAINNET | typeof SOLANA_DEVNET;
  payTo: string;
  facilitatorUrl: URL;
  publicBaseUrl: URL;
  solanaRpcUrl: URL;
}

export function readGatewayConfig(env: NodeJS.ProcessEnv = process.env): GatewayConfig {
  const networkValue = requireValue(env, "PFT_X402_NETWORK");
  if (networkValue !== SOLANA_MAINNET && networkValue !== SOLANA_DEVNET) {
    throw new Error("PFT_X402_NETWORK must be the supported Solana mainnet or devnet CAIP-2 ID");
  }
  const payTo = requireValue(env, "PFT_X402_PAY_TO");
  assertSolanaAddress(payTo);
  const tokenPepper = readExclusiveSecret(
    env,
    "PFT_AMBIENT_TOKEN_PEPPER",
    "PFT_AMBIENT_TOKEN_PEPPER_FILE",
  );
  if (tokenPepper.length < 32) {
    throw new Error("the PfTerminal plan token pepper must contain at least 32 characters");
  }

  return {
    host: env.PFT_AMBIENT_HOST?.trim() || "127.0.0.1",
    port: parsePort(env.PFT_AMBIENT_PORT),
    databaseUrl: requireValue(env, "DATABASE_URL"),
    tokenPepper,
    ambientApiKey: readExclusiveSecret(env, "AMBIENT_API_KEY", "AMBIENT_API_KEY_FILE"),
    ambientBaseUrl: parseHttpUrl(env.AMBIENT_BASE_URL || "https://api.ambient.xyz", "AMBIENT_BASE_URL"),
    network: networkValue,
    payTo,
    facilitatorUrl: parseHttpUrl(
      env.PFT_X402_FACILITATOR_URL ||
        (networkValue === SOLANA_MAINNET
          ? "https://facilitator.payai.network"
          : "https://x402.org/facilitator"),
      "PFT_X402_FACILITATOR_URL",
    ),
    publicBaseUrl: parseHttpUrl(
      requireValue(env, "PFT_AMBIENT_PUBLIC_BASE_URL"),
      "PFT_AMBIENT_PUBLIC_BASE_URL",
    ),
    solanaRpcUrl: parseHttpUrl(
      env.PFT_SOLANA_RPC_URL ||
        (networkValue === SOLANA_MAINNET ? "https://api.mainnet-beta.solana.com" : "https://api.devnet.solana.com"),
      "PFT_SOLANA_RPC_URL",
    ),
  };
}

function requireValue(env: NodeJS.ProcessEnv, name: string): string {
  const value = env[name]?.trim();
  if (!value) throw new Error(`${name} is required`);
  return value;
}

function parsePort(value: string | undefined): number {
  const port = value ? Number(value) : 4021;
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error("PFT_AMBIENT_PORT must be an integer from 1 through 65535");
  }
  return port;
}

function parseHttpUrl(value: string, name: string): URL {
  const parsed = new URL(value);
  if (parsed.protocol !== "https:" && parsed.hostname !== "127.0.0.1" && parsed.hostname !== "localhost") {
    throw new Error(`${name} must use HTTPS unless it points to localhost`);
  }
  return parsed;
}

function assertSolanaAddress(value: string): void {
  let decoded: Uint8Array;
  try {
    decoded = base58.decode(value);
  } catch {
    throw new Error("PFT_X402_PAY_TO must be a base58 Solana address");
  }
  if (decoded.length !== 32) {
    throw new Error("PFT_X402_PAY_TO must decode to a 32-byte Solana address");
  }
}
