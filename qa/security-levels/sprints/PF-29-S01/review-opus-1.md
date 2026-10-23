VERDICT: CHANGES REQUIRED

This was a read-only review of `git diff origin/main...HEAD -- codex-rs`. I didn't run cargo or any tests. I looked closely at the preflight, receipt, launch, picker and resume code; parts of `inventory.rs` I only skimmed.

**P1**
- **A file in the project can crash or hang the preflight** (`core/src/security/inventory.rs:431-436`, `read_bounded`). The size check uses `fs::metadata`, then `fs::read` reads the whole file. Device files and pipes report length 0, so the check passes. A cloned repo can ship a `.env` (or a scanned dotfile) as a symlink to `/dev/zero` (endless read, memory runs out) or as a named pipe (read blocks forever). This hits both `/security` and every Aggressive launch (`isolation_paths` and `audit_at_launch`). It also means the size check can be dodged if the file grows between the two calls.
  - Fix: open the file once, skip anything that isn't a regular file (`file.metadata()?.is_file()`), open it non-blocking, and read with `take(MAX_READ_BYTES + 1)`. Treat going over the limit as "unreadable / listed".
- **Resume refusal only covers the TUI.** It's checked in `tui/src/lib.rs:1834` (startup resume/fork) and `app/session_lifecycle.rs:961` (`/resume`). Two other routes reopen old conversations without it:
  - `codex exec resume` (`exec/src/lib.rs:763,839`).
  - App-server `thread/resume` and `thread/fork` used by other clients.

  If Aggressive applies outside the TUI, an old conversation can still be loaded through these. I also didn't confirm that every in-TUI fork-from-picker path goes through `resume_target_session_with_events`.
  - Fix: put the check in Core or app-server, keyed on the stored level plus the receipt's `activated_at`, or at least add it to exec resume. Then test every resume/fork entry point.

**P2**
- **Environment secrets can appear in Debug output.** `InventorySources` (which holds `env: Vec<(String,String)>` with full values) and `PreflightInput` both derive `Debug`, and `PreflightInput` sits inside the picker. Any `{:?}` or tracing of these prints raw secrets. `Preflight` also prints its drift key.
  - Fix: write a manual `Debug` that shows only variable names and the count, and redact `key`.
- **Isolation is per-file and fixed at launch** (`tui/src/security/preflight.rs` `isolation_paths`, plus `aggressive::deny_reads`). Each SSH key file is denied separately, so a key added to `~/.ssh` during the session stays readable. A key with an unusual name and no `PRIVATE KEY-----` in its first 256 bytes (OpenSSH new format is fine, but e.g. a PuTTY `.ppk`) is never found at all.
  - Fix: deny the whole `~/.ssh` directory except `*.pub`, `known_hosts` and `config`, or the whole directory. Also deny the parent folder of other credential files where practical.
- **Small change when the flag is off.** `LaunchPlan::prepare` applies the receipt-driven denials before config is loaded, so it ignores `protected_mode_preflight`. A leftover receipt keeps adding denials with the flag off. Also, `save(Permissive)` always calls `remove_receipt`. Both only make things stricter, but it's not literally "flag-off unchanged".
  - Fix: document this as intended, or check the feature flag from the raw config file in `prepare`.
- **Launch can't deny paths found only via config sources.** Launch denies paths using `file_sources` (user/project `config.toml` only, no environment). The audit uses `sources_from_config` (all config layers plus environment). A path found only through a profile, managed config or launch-flag layer is never denied. It's correctly reported as "not clean", but the suggested remedy ("restart so launch can deny it") doesn't fix it.
  - Fix: take the same layers in `prepare`, or change the message.

**P3**
- **Same-second edge case** in `refusal_for`: it compares whole seconds (`created < activated_at`), so a thread created in the same second just before activation slips through. Use milliseconds, or `<=`.
- **Receipt tampering looks fine.** Under Permissive, an agent's edits to the receipt are overwritten on the next Aggressive save (`activated_at: None`). Under Aggressive, CODEX_HOME is read-only to agents. Consider deleting the receipt if `level::save` fails after `save_receipt`, so a stale receipt isn't left behind.
- **Drift check looks sound.** It reruns with the same key and compares the readiness lists and manifests in full. There's still a gap between the recheck and the save, but launch re-audits, so that's acceptable.
- **No secret values found in displayed or saved output.** Blocker lines, the receipt (finding IDs and times only) and the picker show locations and classes only.