import assert from "node:assert/strict";
import { describe, test } from "node:test";

import { ed25519 } from "@noble/curves/ed25519";
import { base58 } from "@scure/base";

import { createWalletChallenge, verifyWalletChallenge } from "../src/wallet-auth.js";

const PEPPER = "wallet-auth-test-pepper-with-at-least-32-characters";
const ORIGIN = "https://plans.pfterminal.example";
const NOW = new Date("2026-07-18T12:00:00.000Z");
const PRIVATE_KEY = new Uint8Array(32).fill(7);
const WALLET = base58.encode(ed25519.getPublicKey(PRIVATE_KEY));

function signature(challenge: string): string {
  const message = `pfterminal-plan-ownership-v1\n${ORIGIN}\n${challenge}`;
  return base58.encode(ed25519.sign(new TextEncoder().encode(message), PRIVATE_KEY));
}

describe("wallet ownership challenges", () => {
  test("accepts one correctly bound signature and rejects replay", async () => {
    const used = new Set<string>();
    const issued = createWalletChallenge(WALLET, PEPPER, NOW);
    const verify = () => verifyWalletChallenge({
      walletAddress: WALLET,
      challenge: issued.challenge,
      signature: signature(issued.challenge),
      gatewayOrigin: ORIGIN,
      pepper: PEPPER,
      now: NOW,
      consumeNonce: nonce => {
        if (used.has(nonce)) return true;
        used.add(nonce);
        return false;
      },
    });
    await verify();
    await assert.rejects(verify, /already used/);
  });

  test("rejects changed wallets, signatures, origins, and expired challenges", async () => {
    const issued = createWalletChallenge(WALLET, PEPPER, NOW);
    const base = {
      walletAddress: WALLET,
      challenge: issued.challenge,
      signature: signature(issued.challenge),
      gatewayOrigin: ORIGIN,
      pepper: PEPPER,
      now: NOW,
      consumeNonce: () => false,
    };
    await assert.rejects(verifyWalletChallenge({ ...base, signature: base58.encode(new Uint8Array(64)) }), /invalid/);
    await assert.rejects(verifyWalletChallenge({ ...base, gatewayOrigin: "https://other.example" }), /invalid/);
    await assert.rejects(verifyWalletChallenge({ ...base, now: new Date(NOW.getTime() + 301_000) }), /expired/);
    await assert.rejects(verifyWalletChallenge({ ...base, challenge: `${issued.challenge}x` }), /invalid/);
  });
});
