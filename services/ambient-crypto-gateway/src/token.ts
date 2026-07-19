import { createHmac, randomBytes, randomUUID } from "node:crypto";

const TOKEN_PREFIX = "pft_amb_";

export interface IssuedToken {
  id: string;
  plaintext: string;
  hash: string;
  displayPrefix: string;
}

export function issueToken(pepper: string): IssuedToken {
  if (pepper.length < 32) {
    throw new Error("token pepper must contain at least 32 characters");
  }

  const plaintext = `${TOKEN_PREFIX}${randomBytes(32).toString("base64url")}`;
  return {
    id: randomUUID(),
    plaintext,
    hash: hashToken(plaintext, pepper),
    displayPrefix: `${plaintext.slice(0, TOKEN_PREFIX.length + 8)}…`,
  };
}

export function hashToken(token: string, pepper: string): string {
  return createHmac("sha256", pepper).update(token, "utf8").digest("hex");
}
