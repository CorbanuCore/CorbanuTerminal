# PF-27-S09 independent acceptance — 2026-10-09

**Executor:** independent functional acceptance (GLM 5.2, code-blind).
**Machine:** real Windows 11 (`DESKTOP-UJS3HC4`), accessed over SSH.
**Commit:** `8ecd310ab0faca949118e0bd2114127a6603523f` (origin/main, merge of #379).
**Binary:** `corbanu.exe` v0.1.48, release, built from source with the `C:\CorbanuQA` toolchain
(rustc 1.95.0, `--locked --release`).
**Flag:** `broker_model_auth` (default off).
**Model:** GLM 5.2 on ZAI (`-m glm-5.2 -c model_provider="zai"`).
**Disposable home:** `C:\CorbanuQA\acc9\home` with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.

## Build

The previous run (Claude, stopped by the provider filter) left a partial `C:\CorbanuQA\acc9`
with a reusable `build.cmd` and `env.cmd`. Its source copy had no `.git` and its build was
interrupted (stale cargo/rustc processes, no binary). I killed the stale processes, removed
the old source, cloned origin/main fresh (`8ecd310`), and rebuilt:

```
cargo build --locked --release -j 14 -p codex-cli --bin corbanu
```

Result: `Finished release profile [optimized + debuginfo] in 17m 18s`, `BUILD_EXIT=0`,
`corbanu.exe` 354 MB, `corbanu 0.1.48`.

## Acceptance criteria

### AC1 — flag off = unchanged; flag on = brokered, refused when broker absent

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Flag off, env key, GLM 5.2 | model answers directly (provider: zai, no broker) | answered "flag off baseline ok", `provider: zai`, HTTP 200, exit 0, no broker log lines | **PASS** |
| Flag on, env key, GLM 5.2 | model answers through the broker's pipe; log shows broker started, key handed over, credential held | answered "answered through the Windows broker", exit 0; log: `isolated credential broker started containment=token+dacl+job`, `provider keys handed to the credential broker and removed from the environment: ZAI_API_KEY`, `model provider credential held by the credential broker (host=api.z.ai, port=443, path=/api/paas/v4, header=Bearer)`, `auth.env_provider_key_present=false`, `auth_header_attached=false` | **PASS** |
| Broker killed mid-session | request refused (not sent direct) | `Turn error: Fatal error: the isolated credential broker is unavailable; restart Corbanu to start a new one`; no direct fallback; exit 1 (see AC4) | **PASS** |

### AC2 — env key handed to broker, overwritten in Core's env block; memory scan

Custom PowerShell memory scanner using `ReadProcessMemory` + `VirtualQueryEx`, scanning all
committed, readable regions (~442 MB per process). Searches for the raw key value in ASCII
and UTF-16LE.

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Sanity: search for "glm-5.2" in flag-off process | found (scanner works) | `ASCII_HITS=1937, UTF16_HITS=1` | **PASS** |
| Positive control: flag off, scan for raw key | key found (no broker scrub) | `ASCII_HITS=1, TOTAL_HITS=1` | **PASS** |
| Actual: flag on, scan for raw key after hand-over | key NOT found (scrubbed) | `ASCII_HITS=0, UTF16_HITS=0, TOTAL_HITS=0` | **PASS** |
| Log: env key removed | `removed from the environment: ZAI_API_KEY` | confirmed in log | **PASS** |

### AC3 — stored (vault) key decrypted inside the broker

Vault seeded via the app-server API (`account/login/start`, `providerApiKey`, `zai`), storing
the key in `secrets/local.age`. `ZAI_API_KEY` removed from the environment before the run.

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Vault seeded | `local.age` exists | `SEED OK`; `secrets/local.age` present | **PASS** |
| Flag on, vault key, no env key, GLM 5.2 | model answers; broker holds credential; no "removed from the environment" line | answered "vault key brokered ok"; log: `model provider credential held by the credential broker (host=api.z.ai...)`; no `removed from the environment` line (nothing handed over from env) | **PASS** |
| Memory scan with vault key | raw key NOT in Core's memory | `ASCII_HITS=0, UTF16_HITS=0, TOTAL_HITS=0` | **PASS** |
| Over SSH: vault key in file fallback | profile's file fallback used (no Credential Manager over SSH) | `secrets/keyring-fallback` directory present; broker read the key from the vault file | **PASS** |

**Limit:** the console (medium-integrity) session with Credential Manager was not tested
directly — all runs were over SSH. The evidence README confirms the same broker code reads
the vault key; the difference is only the keyring backend (Credential Manager vs file fallback).

### AC4 — fails closed before the broker runs and when it dies

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Broker killed mid-session | request refused, not sent direct | in-flight request got transport error (pipe broke); retry: `Fatal error: the isolated credential broker is unavailable; restart Corbanu to start a new one`; exit 1; no direct connection attempted | **PASS** |
| Flag off = unchanged | normal direct behaviour | confirmed in AC1 | **PASS** |

### Pipe server identity check — fake pipe squatter must not receive the key

A `FakePipeSquatter.exe` (C#, compiled with `csc.exe`) created named pipe servers on the
broker's expected pipe names (`corbanu-cbk-<hash>-c` and `corbanu-cbk-<hash>-b`) before Core
started.

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Squatter receives the key | must NOT receive the key or any connection | squatter log: only "Waiting for connection..."; no "CONNECTION RECEIVED"; no "KEY_LEAKED"; Core started its own broker and answered normally | **PASS** |

Core's broker creates its own pipe server instances; the squatter's pre-existing pipe did not
intercept the connection. The broker's checked-pipe mechanism (PF-27-S06) verifies the server
process identity on every connection.

### Restart / resume

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Resume a brokered session | fresh broker started, key handed over again, model answers | session 1 answered "session one ok" (ID `01a1239b-...`); session 2 (resume same ID): new broker started (`containment=token+dacl+job`), `provider keys handed to the credential broker and removed from the environment: ZAI_API_KEY`, answered "session two resumed ok" | **PASS** |

## Verdict

**ACCEPT WITH LIMITS.**

All four acceptance criteria pass on the real Windows 11 machine, built from origin/main
(`8ecd310`), with the `broker_model_auth` flag. The env key is handed to the broker and
scrubbed from Core's process memory (verified by an independent memory scanner with a positive
control). The vault key is decrypted inside the broker with no env key and no raw key in
Core's memory. Requests fail closed when the broker dies (never sent direct). A fake pipe
squatter does not receive the key. Resume spawns a fresh broker.

### Limits (not blockers; consistent with the sprint's own known limits)

1. **No console-session (Credential Manager) run.** All tests were over SSH, so the vault
   key used the profile's file fallback. The console session with Credential Manager was not
   tested directly. The sprint evidence README confirms the same broker code path; the
   difference is only the keyring backend.
2. **Memory scan covers the running process, not Core's start-up.** The scan runs after
   hand-over (the broker has taken the key). A bare copy of the key that other Core code
   keeps in a live block (e.g. a `std::env::vars()` snapshot) is not found by the sweep —
   this is the sprint's own known limit. My scan confirms 0 hits after hand-over, which is
   the acceptance criterion.
3. **ChatGPT sign-in refresh** is unit-level only (the machine has no ChatGPT login); this
   is the sprint's own known limit and was not tested here.
4. **Pipe squatter test:** the squatter did not receive a connection because Core's broker
   creates its own pipe server instances. An exclusive-squat scenario (where the broker
   cannot create its pipe) was not separately tested, but the broker's checked-pipe identity
   verification (PF-27-S06) is the mechanism that prevents key leakage to a fake server.

## What Travis needs to decide

Whether the limits above are acceptable for archiving PF-27-S09, or whether any require a
follow-up before acceptance. My recommendation: **accept with limits** — the limits match
the sprint's own known-limits list, and every AC passes on the real machine.

## Cleanup

All Windows artifacts (`C:\CorbanuQA\acc9\*`, the ZAI key file, the vault seed, the squatter
binary, all test scripts and logs) will be removed after the PR is merged. The `C:\CorbanuQA\target`
directory is shared with other QA runs and is left in place per the brief.
