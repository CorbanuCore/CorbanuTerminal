import { randomBytes } from "node:crypto";

import { ed25519 } from "@noble/curves/ed25519";
import { base58 } from "@scure/base";
import { createKeyPairSignerFromBytes } from "@solana/kit";

import { writeSecretFile } from "./wallet-file.js";

const outputPath = process.env.PFT_SOLANA_WALLET_FILE?.trim();
if (!outputPath) throw new Error("PFT_SOLANA_WALLET_FILE is required");

const privateSeed = randomBytes(32);
const secretKey = new Uint8Array(64);
secretKey.set(privateSeed);
secretKey.set(ed25519.getPublicKey(privateSeed), 32);
const signer = await createKeyPairSignerFromBytes(secretKey);

await writeSecretFile(outputPath, {
  address: signer.address,
  privateKey: base58.encode(secretKey),
});
process.stdout.write(`Created Solana wallet ${signer.address} at ${outputPath}\n`);
