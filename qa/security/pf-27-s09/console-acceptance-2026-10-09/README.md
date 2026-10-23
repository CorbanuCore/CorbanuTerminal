# PF-27-S09 console-session Credential Manager acceptance — 2026-10-09

**Executor:** independent functional acceptance (code-blind).
**Machine:** real Windows 11 Pro (`DESKTOP-UJS3HC4`), build 10.0.26200.
**Commit:** `b1e20a8ec6192ea16a4b3c7e730d28cb712d3707` (origin/main).
**Binary:** `corbanu.exe` v0.1.48, release, built from source in `C:\CorbanuQA\acc389\src` with the `C:\CorbanuQA` toolchain; shared `C:\CorbanuQA\target` retained.
**Build:** `cargo build --locked --release -j 14 -p codex-cli --bin corbanu` — finished in 19m 08s.
**Flag:** `broker_model_auth` (`--enable` / `--disable`).
**Model:** GLM 5.3 Flash on ZAI (`-m glm-5.3-flash -c model_provider="zai"`).
**Disposable home:** `C:\CorbanuQA\acc389\home`.
**Method:** PowerShell scheduled tasks with `LogonType=Interactive`, run only while the console user was logged on. Each run wrote its process session id and `[Environment]::UserInteractive`. Memory scans used an independently compiled C# `ReadProcessMemory` / `VirtualQueryEx` scanner over committed readable regions, searching both UTF-8 and UTF-16LE byte forms of the raw provider key.

## Backend and console-session proof

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Scheduled task session | task and child process run in console session, not SSH/services session 0 | `qwinsta`: console = session 1; task: `task_session=1 task_interactive=True`; child PIDs 13736 and 14632 both `child_session=1` | **PASS** |
| Vault seed backend | Credential Manager key available to broker; no file fallback | app-server `account/login/start` returned `{"id":1,"result":{"type":"apiKey"}}` and `account/login/completed success=true`; `local.age=present`; `keyring-fallback=absent`; interactive task emitted an existing `secrets|…` Credential Manager target | **PASS** |
| No env key in stored-key run | `ZAI_API_KEY` unset before launch | run script explicitly removed `ZAI_API_KEY`; no env-key hand-over log line appeared | **PASS** |

## Results

| Check | Expected | Actual | Verdict |
| --- | --- | --- | --- |
| Flag off = unchanged | direct model run still works | prompt got exact `off-env ok`; JSONL showed `agent_message` and `turn.completed`; stderr had no credential-broker lines | **PASS** |
| Positive control (flag off, env key) | raw provider key present in Core memory | `PID=13736 READ_MB=420.6 ASCII_HITS=8 UTF16_HITS=3 TOTAL_HITS=11` | **PASS** |
| Flag on, stored vault key, no env key | GLM answers through broker | prompt got exact `on-vault ok`; stderr: `isolated credential broker started containment=token+dacl+job`; `model provider credential held by the credential broker (host=api.z.ai, port=443, path=/api/paas/v4, header=Bearer)` | **PASS** |
| Flag on memory scan of Core | raw provider key absent after hand-over | `PID=14632 READ_MB=420.2 ASCII_HITS=0 UTF16_HITS=0 TOTAL_HITS=0` | **PASS** |

## Notes / limits

- `PASS WITH LIMITS`: the required Credential Manager backend conflicts with the standing `CORBANU_TEST_NO_NATIVE_KEYRING=1` rule for freshly built binaries. On Windows, setting it selected `secrets/keyring-fallback`, so the console seed and brokered runs intentionally did **not** set it. The disposable Windows home and Credential Manager entry were used instead; the keychain risk addressed by the rule does not apply here, but the exception is explicitly recorded.
- The memory scan covers the running Core process after hand-over, not a full Core start-up trace. This matches the prior acceptance method.
- The scanner was compiled from an independent small C# file, not reused from implementation sources.
- Windows outputs are retained only in the QA record; the raw provider key file was deleted after all scans.

## Cleanup

Performed after the acceptance record was merged: remove `C:\CorbanuQA\acc389`, all scheduled tasks created for this run, and the run-created Credential Manager entry. `C:\CorbanuQA\target` is left in place per the brief.
