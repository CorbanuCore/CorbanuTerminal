# D319 vault compatibility fixtures — never regenerate

Generated once on 2026-11-07 from `main` at `18fa372aaaf5` (age 0.11.2 in `Cargo.lock`), before the upstream sync
(`rust-v0.167.0`) touched `age`. Owner decision D319: vault files written by earlier builds must keep
opening. `tests/d319_vault_compat.rs` checks them; if a change makes it fail, fix the change, not the
fixtures.

- `wf16/`: written by `main`'s own code with a fixed fixture key (`Vault::add` for `local.age`,
  `SecretsManager::set` for `codex_auth.age` and `mcp_oauth.age`). Work factor 16 is what current builds write.
- `wf18/`, `wf20/`: the same plaintexts re-encrypted with age 0.11.2's scrypt recipient at work factor 18 and
  20 (20 is the decrypt limit; older builds calibrated up to it).
- `expected.json`: the decrypted plaintext of each file.

The key is `FIXTURE_PASSPHRASE` in the test. All values are fixture strings, not credentials.
SHA-256 of the fixtures at generation:

```
f4444d81e0b1b5bdca0c09e1a2129ee961dba9ce41f6197b0f266c755d6fb299  wf16/secrets/codex_auth.age
c6826c6c20609133bfcdcdc7d842a484c75f595f7e8cb370015bde59252659c7  wf16/secrets/local.age
ecf5cbaaefb425aa75b151d8870d31c072774d45cfa4e7e32aa81f41c9b39676  wf16/secrets/mcp_oauth.age
ad1dd5f476ec7a314a4ff50e4d4d79f7085776993a3d7d149de9ab538f11f30b  wf18/secrets/codex_auth.age
3732d85d121f30348e9f43d8b64da9459a42dd043e9b061bcd05f2546fddc635  wf18/secrets/local.age
9292e5f8c4afdee8941122dc99c13d1edb4a97499e25b0fb93194cad31b0e503  wf18/secrets/mcp_oauth.age
771383f959dbd4dfb90e899a3de4739759ee7ec746c60a6f893ee4669fb33532  wf20/secrets/codex_auth.age
afc369438f898e7a392748b2308abc79e12ce8aaf836d6b940b6a18d37f7cbe9  wf20/secrets/local.age
753befc5ec399eca66e832295cdcd94f1792b462c14bf0292c17a809fb3553cf  wf20/secrets/mcp_oauth.age
6908f2a791d050791a4af3448af89d3304a2d661506e2d568e82c75fc65d882e  expected.json
```
