import assert from "node:assert/strict";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, test } from "node:test";

import { ed25519 } from "@noble/curves/ed25519";
import { base58 } from "@scure/base";
import { createKeyPairSignerFromBytes } from "@solana/kit";

import { loadWalletFile, writeSecretFile } from "../src/wallet-file.js";

const directories: string[] = [];

afterEach(async () => {
  await Promise.all(directories.splice(0).map(path => rm(path, { force: true, recursive: true })));
});

async function walletFixture(): Promise<{ address: string; privateKey: string }> {
  const seed = new Uint8Array(32).fill(7);
  const bytes = new Uint8Array(64);
  bytes.set(seed);
  bytes.set(ed25519.getPublicKey(seed), 32);
  const signer = await createKeyPairSignerFromBytes(bytes);
  return { address: signer.address, privateKey: base58.encode(bytes) };
}

describe("Solana wallet files", () => {
  test("writes a new owner-only wallet and loads the matching signer", async () => {
    const directory = await mkdtemp(join(tmpdir(), "pft-wallet-"));
    directories.push(directory);
    const path = join(directory, "wallet.json");
    const wallet = await walletFixture();
    await writeSecretFile(path, wallet);
    const signer = await loadWalletFile(path);
    assert.equal(signer.address, wallet.address);
  });

  test("refuses overwrite, broad permissions, and mismatched addresses", async () => {
    const directory = await mkdtemp(join(tmpdir(), "pft-wallet-"));
    directories.push(directory);
    const path = join(directory, "wallet.json");
    const wallet = await walletFixture();
    await writeSecretFile(path, wallet);
    await assert.rejects(writeSecretFile(path, wallet), /EEXIST/);

    await writeFile(path, await readFile(path), { mode: 0o644 });
    await chmod(path, 0o644);
    await assert.rejects(loadWalletFile(path), /group or other users/);

    await chmod(path, 0o600);
    await writeFile(path, JSON.stringify({ ...wallet, address: "11111111111111111111111111111111" }));
    await assert.rejects(loadWalletFile(path), /does not match/);
  });
});
