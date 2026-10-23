# RETURN — pf83-fixtures-57

Action `pf83-fixtures-57`; allocation digest `d44e66a800596bc3a6e1d681e353a4d62f055725b41e6c9a486f7c9758e95e40`; claim `8d60edaf-3339-4c24-a8d4-4bfcfba980a6`; worker `gpt-6-astra`, effort `high`.
Read frozen brief first and verified SHA-256 `2571ad58e0f4b383a09f1bd55cf2711cff2a9808e5307277bb9b77ec18fa2a61`.
Base/HEAD `c124a3a1c42712ce36265910cda24cc9520f7c12`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`; initially clean. No commit or push.

Routine test-only revision supporting PF-83-S01. Product heading **Permission selection confirmation — TO BUILD**: “A submitted selection is not a confirmed change”; “Error, disconnect and restart recovery must not claim success or retry an uncertain change blindly.” No production behavior, product scope, plan or sprint is changed. This internal fixture repair does not establish independent functional acceptance, human-test readiness, packaged real-key qualification, live-repository coverage or release readiness. Those gates remain with the manager; no additional opinion review was commissioned.

## Changes

- In `restart_settled_permission_case`, the restricted starting branch now rejects both `AskForApproval::Never` and `SandboxPolicy::DangerFullAccess` before deriving the probe expectation. The full starting branch retains the documented allowance for a reported reset.
- Deleted the post-settlement stale-completion block and its unused `settled` binding. It did not exercise the stale-ID guard. The earlier block still delivers an old completion while a newer selection is pending. The prior return's claim of two independent stale-completion proofs is withdrawn.
- Removed diagnostic `eprintln!`. The pre-selection command drain must now be nonempty AND every item must be `ListSkills`; subsequent selection-command assertions remain unchanged.
- Added `permission_confirmation_f10_unconfirmed_selection_does_not_persist_launch_authority`. Two independent synthetic profiles begin restricted/full respectively. Each issues the opposite real embedded-server permission request and observes its real Applied RPC reply without delivering completion/settings observations to the TUI. Pending selection identity and absent runtime overrides are checked. After server shutdown and App destruction, the saved config bytes, fresh ConfigBuilder permission/policy values, and a newly started embedded session must retain the original launch authority.

## F10(c) residual

A modest restart-equivalent persistence increment is expressible and is implemented. It checks that an **unconfirmed TUI selection does not become saved launch authority**, in both directions, using real request handling, disk reload and a fresh embedded server/session. This is not a cold operating-system process restart or a resume of the changed thread. The backend has actually applied the request in this fixture; withholding its reply from the TUI does not hold application pending. Thus the original F10(c) case remains partially uncovered: the available native API lacks a controllable pre-application barrier for an accepted permission request plus ordinary restart/resume navigation at that barrier. Exact-package isolated execution must still establish the relevant pending state, resume the affected session, run a fresh protected-work probe and verify the user's recovery explanation. The brief's phrase “UNCONFIRMED selection never persists authority” is valid here only for saved launch authority; absence of UI confirmation cannot imply that a request known to have applied in Core must be erased from thread state.

## Deliberate regression checks

Temporary changes are confined to the owned app-server test file and removed before final validation. Immediately inside the restricted branch, shadow the real decoded response with:

```rust
let resumed = ThreadResumeResponse {
    approval_policy: AskForApproval::Never,
    sandbox: SandboxPolicy::DangerFullAccess,
    ..resumed
};
```

This simulates an over-permissive response at the fixture boundary; it is not a production backend mutation. A second variant overrides only `sandbox`, retaining the actual approval policy so the sandbox assertion is independently reachable. Both mutations were removed before final formatting/tests.

Both guarded focused runs failed exactly `suite::v2::thread_settings_update::thread_settings_f10_restart_effective_restricted_reports_probe_authority`: **1 run / 0 passed / 1 failed / 1107 skipped, exit 100**, each. `pf83-fixtures-57-mutant-full-resume.log` fails with “restarting a restricted thread must not silently remove approval requirements”, left/right `Never`. `pf83-fixtures-57-mutant-sandbox-resume.log` independently fails with “restarting a restricted thread must not silently remove the sandbox”, left/right `DangerFullAccess`. Neither mutation changes production code or executes the protected probe under fabricated authority; each fails before the probe.

## Validation

Read `docs/development/test-isolation.md` before tests. All Rust tests use guarded `just test` from `codex-rs`, `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31`, `NEXTEST_TEST_THREADS=4`, `INSTA_UPDATE=no`, `--locked --offline --retries 0`. Only disposable synthetic profiles are used. Only the two owned Rust files are formatted with `rustfmt --edition 2024 --config skip_children=true`; no workspace formatter/fix command.

Prerequisite attempt `pf83-fixtures-57-prerequisites.log` exited 1 before building: Cargo rejects duplicate `--bins` flags. Corrected command `cargo build --locked --offline -p codex-cli -p codex-rmcp-client -p codex-code-mode-host --bins` built all required binaries and exited 0; log `pf83-fixtures-57-prerequisites-corrected.log`. No raw Cargo test command was used.

Initial TUI lane `pf83-fixtures-57-permission-confirmation.log`: **12 run / 11 passed / 1 failed / 4161 skipped, exit 100**. Exact failure: `app::permission_confirmation::tests::permission_confirmation_f10_unconfirmed_selection_does_not_persist_launch_authority`. Its byte-for-byte disk assertion compared post-run config against the pre-startup input; Full Access startup had added `[projects.<synthetic cwd>] trust_level = "trusted"`. The fixture now freezes disk bytes after thread/session initialization and before the selection request. This retains the no-selection-persistence assertion without treating startup metadata as a selection effect. Original attempt retained; final rerun uses a new log name. The corrected complete TUI lane passes all 12 tests (4161 skipped), including both directions of the new fixture. The startup write is implemented in app-server `thread_processor.rs`'s project-trust handling; the passing post-startup baseline locates the write before selection.

Final required lanes (command suffix after `just test`, plus common flags above):

| Lane | Run / passed / failed / skipped | Exit | Raw log |
| --- | --- | --- | --- |
| `-p codex-tui permission_confirmation` | 12 / 12 / 0 / 4161 | 0 | `pf83-fixtures-57-permission-confirmation-final.log` |
| `-p codex-app-server thread_settings` | 26 / 26 / 0 / 1082 | 0 | `pf83-fixtures-57-thread-settings.log` |
| `-p codex-app-server settings_confirmation` | 11 / 11 / 0 / 1097 | 0 | `pf83-fixtures-57-settings-confirmation.log` |
| `-p codex-core session::tests` | 268 / 268 / 0 / 3401 | 0 | `pf83-fixtures-57-core-session.log` |

All four final lanes exited 0: **317 passing test executions, zero failures** (filters overlap; this is not a unique-test count). No ignored/zero-test lane, whole-crate lane, automatic retry or suppressed failure. Exact historical failure names and counts appear above. Final affected lanes ran after the last source edit/format.

## Scope and brief corrections

- The three review findings are correct; each is addressed. The sibling pending-approval restart fixture has an approval-policy guard only, not a sandbox guard; this revision adds both requested guards to the settled restricted fixture.
- F10(c)'s native opportunity was real but narrower than complete pending-application recovery; see the explicit residual above. An unconfirmed UI request can already be applied server-side.
- Initial build syntax and new-fixture baseline mistakes are recorded above as worker errors, not product findings.
- Source diff: app-server test **+12/-0**, TUI test **+108/-8**: **120 added / 8 deleted**, zero production Rust changes. The TUI source includes a 107-line new fixture (attribute plus function), one replaced startup assertion, and seven removed post-settlement lines.
- Final source SHA-256: `thread_settings_update.rs` = `7c7b42795bc3d5b4c0d0ad307ef10289a3a4958a95c2e42b0aee5ab8b36935ac`; `permission_confirmation_tests.rs` = `7c5b5a41cb4b85a39a5f0830dd0c367418d6de9170e289737d556f78a8ca5048`.
- `git diff --check` passed; status contains only the two authorized test files and this return record. Local ignored raw logs also live under the authorized management-bootstrap directory. `python3 -B docs/sprints/check.py` passed: 115 current / 127 archived. No native credential prompt observed.

## Raw log integrity

Logs are local ignored artifacts beside this report; original attempts were not overwritten.

| Log | SHA-256 |
| --- | --- |
| `pf83-fixtures-57-core-session.log` | `6b6ea869dc881c4bfe9482625dad004d1ec76707e2ec0d12482abd2fac8756ab` |
| `pf83-fixtures-57-mutant-full-resume.log` | `b9052bd8eeb6488695d4cd914e38f7e748442fe4f645d574a73d1d3317a2bd3e` |
| `pf83-fixtures-57-mutant-sandbox-resume.log` | `738834f3b760c8996ed06abdf5b1240566c4d9d727751497b377103d842566f9` |
| `pf83-fixtures-57-permission-confirmation-final.log` | `9824f2627304b2233417656a8e8d7c587f509d567ceaaa8281688041630b47e6` |
| `pf83-fixtures-57-permission-confirmation.log` | `0bc4ca9f5e9bcdcb8a2a17b2ed7e835899f8fa630b8babd5262297fbaacb3c43` |
| `pf83-fixtures-57-prerequisites-corrected.log` | `01157329fb38a9d8f2dfdd3b1eb7dd32eac10431373049f57a0887e2b73d8e8f` |
| `pf83-fixtures-57-prerequisites.log` | `7e1cc99569a6521d384107374e969762fbe5aa67bb95688d5f400c4210f10605` |
| `pf83-fixtures-57-settings-confirmation.log` | `92dea9472b8a8b52346de1f610102713950150ee8e0ea416d3cbd9decea70c86` |
| `pf83-fixtures-57-thread-settings.log` | `20ca48fde0cfeffe8581220116c3edbac2eda450640d632c017a35d0c397bc1b` |
