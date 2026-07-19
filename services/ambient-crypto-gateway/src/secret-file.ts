import { readFileSync, statSync } from "node:fs";

const MAX_SECRET_BYTES = 16 * 1024;

export function readSecretFile(path: string): string {
  const metadata = statSync(path);
  if (!metadata.isFile()) {
    throw new Error("secret path must identify a regular file");
  }
  if ((metadata.mode & 0o077) !== 0) {
    throw new Error("secret file must not be readable or writable by group or other users");
  }
  if (metadata.size > MAX_SECRET_BYTES) {
    throw new Error("secret file exceeds 16 KiB");
  }
  const value = readFileSync(path, "utf8").trim();
  if (!value) {
    throw new Error("secret file is empty");
  }
  return value;
}

export function readExclusiveSecret(
  env: NodeJS.ProcessEnv,
  valueName: string,
  fileName: string,
): string {
  const inlineValue = env[valueName]?.trim();
  const filePath = env[fileName]?.trim();
  if (inlineValue && filePath) {
    throw new Error(`${valueName} and ${fileName} cannot both be set`);
  }
  if (filePath) return readSecretFile(filePath);
  if (inlineValue) return inlineValue;
  throw new Error(`${valueName} or ${fileName} is required`);
}
