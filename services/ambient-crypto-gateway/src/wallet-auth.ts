import { createHmac, randomBytes, timingSafeEqual } from "node:crypto";

import { ed25519 } from "@noble/curves/ed25519";
import { base58 } from "@scure/base";

const CHALLENGE_TTL_MS = 5 * 60 * 1_000;

interface ChallengeClaims {
  version: 1;
  action: "create_key";
  walletAddress: string;
  nonce: string;
  issuedAt: number;
  expiresAt: number;
}

export interface WalletChallenge {
  challenge: string;
  expiresAt: string;
}

export function createWalletChallenge(
  walletAddress: string,
  pepper: string,
  now: Date,
): WalletChallenge {
  assertWalletAddress(walletAddress);
  const claims: ChallengeClaims = {
    version: 1,
    action: "create_key",
    walletAddress,
    nonce: randomBytes(24).toString("base64url"),
    issuedAt: now.getTime(),
    expiresAt: now.getTime() + CHALLENGE_TTL_MS,
  };
  const payload = Buffer.from(JSON.stringify(claims)).toString("base64url");
  const mac = challengeMac(payload, pepper);
  return {
    challenge: `${payload}.${mac}`,
    expiresAt: new Date(claims.expiresAt).toISOString(),
  };
}

export async function verifyWalletChallenge(input: {
  walletAddress: string;
  challenge: string;
  signature: string;
  gatewayOrigin: string;
  pepper: string;
  now: Date;
  consumeNonce: (nonce: string) => boolean | Promise<boolean>;
}): Promise<void> {
  assertWalletAddress(input.walletAddress);
  const [payload, suppliedMac, extra] = input.challenge.split(".");
  if (!payload || !suppliedMac || extra) throw new Error("wallet challenge is malformed");
  const expectedMac = Buffer.from(challengeMac(payload, input.pepper), "base64url");
  const actualMac = Buffer.from(suppliedMac, "base64url");
  if (actualMac.length !== expectedMac.length || !timingSafeEqual(actualMac, expectedMac)) {
    throw new Error("wallet challenge is invalid");
  }
  let claims: ChallengeClaims;
  try {
    claims = JSON.parse(Buffer.from(payload, "base64url").toString("utf8")) as ChallengeClaims;
  } catch {
    throw new Error("wallet challenge is malformed");
  }
  if (
    claims.version !== 1 ||
    claims.action !== "create_key" ||
    claims.walletAddress !== input.walletAddress ||
    !claims.nonce ||
    !Number.isSafeInteger(claims.issuedAt) ||
    !Number.isSafeInteger(claims.expiresAt) ||
    claims.issuedAt > input.now.getTime() + 30_000 ||
    claims.expiresAt < input.now.getTime() ||
    claims.expiresAt - claims.issuedAt !== CHALLENGE_TTL_MS
  ) {
    throw new Error("wallet challenge is expired or does not match this request");
  }
  let signature: Uint8Array;
  try {
    signature = base58.decode(input.signature);
  } catch {
    throw new Error("wallet signature is malformed");
  }
  const message = `pfterminal-plan-ownership-v1\n${input.gatewayOrigin}\n${input.challenge}`;
  if (!ed25519.verify(signature, new TextEncoder().encode(message), base58.decode(input.walletAddress))) {
    throw new Error("wallet signature is invalid");
  }
  if (await input.consumeNonce(claims.nonce)) {
    throw new Error("wallet challenge was already used");
  }
}

function challengeMac(payload: string, pepper: string): string {
  return createHmac("sha256", pepper).update("wallet-challenge-v1\0").update(payload).digest("base64url");
}

function assertWalletAddress(value: string): void {
  let decoded: Uint8Array;
  try {
    decoded = base58.decode(value);
  } catch {
    throw new Error("walletAddress must be a base58 Solana address");
  }
  if (decoded.length !== 32) throw new Error("walletAddress must be a base58 Solana address");
}
