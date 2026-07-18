import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, test } from "node:test";

import { readExclusiveSecret, readSecretFile } from "../src/secret-file.js";

const directories: string[] = [];

afterEach(() => {
  for (const path of directories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
});

function secretFixture(value = "upstream-secret\n"): string {
  const directory = mkdtempSync(join(tmpdir(), "pft-ambient-secret-"));
  directories.push(directory);
  const path = join(directory, "secret.txt");
  writeFileSync(path, value, { mode: 0o600 });
  return path;
}

describe("file-backed secrets", () => {
  test("reads an owner-only secret without retaining its trailing newline", () => {
    const path = secretFixture();
    assert.equal(readSecretFile(path), "upstream-secret");
    assert.equal(readExclusiveSecret({ AMBIENT_API_KEY_FILE: path }, "AMBIENT_API_KEY", "AMBIENT_API_KEY_FILE"), "upstream-secret");
  });

  test("rejects broad permissions, empty files, and ambiguous sources", () => {
    const path = secretFixture();
    chmodSync(path, 0o644);
    assert.throws(() => readSecretFile(path), /group or other users/);

    const emptyPath = secretFixture("\n");
    assert.throws(() => readSecretFile(emptyPath), /empty/);
    assert.throws(
      () => readExclusiveSecret(
        { AMBIENT_API_KEY: "inline", AMBIENT_API_KEY_FILE: emptyPath },
        "AMBIENT_API_KEY",
        "AMBIENT_API_KEY_FILE",
      ),
      /cannot both be set/,
    );
  });
});
