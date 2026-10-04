# RETURN — pf83-escalation-60

Action `pf83-escalation-60`; allocation digest `a9826f98656ebabc25aee61674e832f65738eb896fe33157001edf02aa046a53`; claim `bbb81cea-ca36-4142-90e0-4b35c565fdcf`; worker `gpt-6-astra`, effort `high`.
Read the frozen brief first; SHA-256 matched `21ebe55cd57b1c8348316cfe2aa9fd49e77b3fabb6a80d0c6b5475b6aebd0d18`.
Base/HEAD `63e2fc6652a436745caaef4f6f8013de04ab12ef`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`; initially clean. No commit or push.

Routine test/evidence revision supporting PF-83-S01. Product heading **Permission selection confirmation — TO BUILD**: “A submitted selection is not a confirmed change”; “Error, disconnect and restart recovery must not claim success or retry an uncertain change blindly.” No production behavior or plan/sprint mandate changed. Internal test-only work does not require new functional design; the existing independent exact-package functional gate remains open with the integrator. No new review or approval is claimed.

## Pre-restart authority and partial-escalation proof

At `thread_settings_update.rs:489`, the restricted branch compares the entire resumed approval/sandbox pair against the known pre-restart pair:
`(AskForApproval::UnlessTrusted, SandboxPolicy::ReadOnly { network_access: false })`.
The equality precedes `permits_write` and the resumed probe; it rejects intermediate approval, filesystem and network escalations as well as full access. The full-starting branch retains its existing reported-reset allowance.

Two temporary decoded-response mutations, each scoped inside this restricted branch, were tested separately:
- Approval only: `ThreadResumeResponse { approval_policy: AskForApproval::OnRequest, ..resumed.clone() }`. The real sandbox remains read-only with network denied.
- Network only: `ThreadResumeResponse { sandbox: SandboxPolicy::ReadOnly { network_access: true }, ..resumed.clone() }`. The real approval remains UnlessTrusted.

Both fail exactly `suite::v2::thread_settings_update::thread_settings_f10_restart_effective_restricted_reports_probe_authority` at “restarting a restricted thread must retain the pre-restart authority”, before the resumed probe.
Each corrected mutant run: **1 run / 0 passed / 1 failed / 1107 skipped, exit 100**.
The assertion diffs show OnRequest versus UnlessTrusted, then network true versus false, respectively. Both would pass the old extremal exclusions.
These are fixture-boundary mutations, not production backend changes; both were removed before final formatting and all four final lanes.

Preserved initial attempt: `pf83-escalation-60-mutant-approval.log`, exit 101, **zero tests executed**. My first temporary struct update used `..resumed`, moving the sandbox before its later use (E0382). Corrected to `..resumed.clone()`; this build failure is not mutation-test proof. The corrected run has a distinct log.

## F10(c) naming and remaining exit evidence

Renamed the TUI fixture to `permission_confirmation_f10_request_does_not_optimistically_apply_or_persist` at line 205. Its comment at lines 280–283 explicitly limits the reload check to request-time persistence. No confirmed-selection control was added: the rename option was chosen.
The fixture still checks both selection directions, absent optimistic runtime overrides, unchanged saved config, reload and a fresh embedded thread. Those disk/fresh-thread checks cannot distinguish confirmed from unconfirmed selection.
This supersedes the prior return's “restart-equivalent persistence increment” characterization; the historical record remains intact.

The sprint can cite the **10-line** [F10/F11 remaining exit evidence section](pf83-escalation-60-f10-f11-exit.md). It identifies the missing pending-application barrier and same-thread recovery fixture for F10, the missing actual supported inference routes for F11, legitimate native prerequisites, and the real-key/package/isolation requirements that native fixtures cannot close.

## Final-tree validation

Read `docs/development/test-isolation.md` before testing. From `codex-rs`, every Rust test used guarded `just test`, shared dedicated `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31`, `NEXTEST_TEST_THREADS=4`, `INSTA_UPDATE=no`, and `--locked --offline --retries 0`. Disposable synthetic profiles only; no native credential prompt observed.
Prerequisites were built first: `cargo build --locked --offline -p codex-cli -p codex-rmcp-client -p codex-code-mode-host --bins`, exit 0. No raw cargo test/nextest or whole-crate test lane.
After removing mutants, ran only `rustfmt --edition 2024 --config skip_children=true app-server/tests/suite/v2/thread_settings_update.rs tui/src/app/permission_confirmation_tests.rs`. Exit 0; stable-toolchain imports-granularity warnings only. No workspace fmt/fix.

| Command after `just test` (plus common flags above) | Run / passed / failed / skipped | Exit | Log suffix |
| --- | --- | --- | --- |
| `-p codex-app-server thread_settings` | 26 / 26 / 0 / 1082 | 0 | thread-settings |
| `-p codex-app-server settings_confirmation` | 11 / 11 / 0 / 1097 | 0 | settings-confirmation |
| `-p codex-core session::tests` | 268 / 268 / 0 / 3401 | 0 | core-session |
| `-p codex-tui permission_confirmation` | 12 / 12 / 0 / 4161 | 0 | permission-confirmation |

**317 passing final test executions, zero failures**; filters overlap, so this is not a unique-test count. Final failure names: none. Mutant failure name and initial build error are recorded above.
Raw logs are retained as ignored local artifacts beside this receipt, named `pf83-escalation-60-<suffix>.log`; mutation suffixes are `mutant-approval`, `mutant-approval-corrected`, `mutant-network`; build suffix is `prerequisites`.

## Changed lines, limits and brief corrections

- `codex-rs/app-server/tests/suite/v2/thread_settings_update.rs`: **+12/-9**, lines 485–499.
- `codex-rs/tui/src/app/permission_confirmation_tests.rs`: **+5/-3**, line 205 and lines 280–283.
- Total Rust diff: **17 additions / 12 deletions = 29 changed lines**, zero production Rust changes. Added this receipt and the ten-line exit section under the authorized QA directory; local raw logs are there too.
- Final source SHA-256: app-server test `994b48592c00ee980f8353a5f95e19cd4a47e717388dcc10ee4ea81cc4f232ab`; TUI test `cfa936dd07970092b55c0d9ac6094819bb957915ac8cc8465cb5d759c65cdd85`.
- `git diff --check` passed; status showed only authorized paths. `python3 -B docs/sprints/check.py` passed: 115 current / 127 archived.
- No substantive error found in the brief. “Qualified natively” is limited to native fixture claims; it does not close frozen real-key acceptance. F10 remains partial and F11 unbuilt; no readiness, live-repository, human-acceptance or release claim.
