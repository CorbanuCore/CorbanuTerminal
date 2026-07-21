import { constants, promises as fs } from "node:fs";

import { base58 } from "@scure/base";
import { createKeyPairSignerFromBytes, type KeyPairSigner } from "@solana/kit";

export interface StoredSolanaWallet {
  address: string;
  privateKey: string;
}

export async function loadWalletFile(path: string): Promise<KeyPairSigner> {
  const metadata = await fs.stat(path);
  if ((metadata.mode & 0o077) !== 0) {
    throw new Error("wallet file must not be readable or writable by group or other users");
  }
  const value = JSON.parse(await fs.readFile(path, "utf8")) as Partial<StoredSolanaWallet>;
  if (typeof value.address !== "string" || typeof value.privateKey !== "string") {
    throw new Error("wallet file is missing its address or private key");
  }
  const signer = await createKeyPairSignerFromBytes(base58.decode(value.privateKey));
  if (signer.address !== value.address) {
    throw new Error("wallet file address does not match its private key");
  }
  return signer;
}

export async function writeSecretFile(path: string, value: unknown): Promise<void> {
  const handle = await fs.open(path, constants.O_CREAT | constants.O_EXCL | constants.O_WRONLY, 0o600);
  try {
    await handle.writeFile(`${JSON.stringify(value, null, 2)}\n`, "utf8");
  } finally {
    await handle.close();
  }
}
