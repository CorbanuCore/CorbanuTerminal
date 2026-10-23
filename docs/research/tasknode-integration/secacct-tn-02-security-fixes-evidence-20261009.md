# Task Node evidence record: SECACCT-TN-02

Compile the SECACCT-TN-02 Seven-Fix Security Evidence Record. Task `task_9e695438561fca22216b86eccb784f09`, request `req_43bfc8ff66425019e2be5999b4510ee044588d8e3dc2d08d9e69d2614aab6748`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: all seven fixes merged; the issues they fix are closed.** Six are Windows fixes found during and after the PF-27-S06/S07 real-Windows gates (measured on a real Windows 11 machine, elevated and in a normal session); #317 makes Windows clippy `-D warnings` clean and adds a postmerge Windows clippy leg; #313 keeps provider API keys out of the commands the model runs, at every security level. **Open, not claimed:** #323, #345 (narrowing the SSH window-station grant), PF-27-S08 and S09.

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#321](https://github.com/CorbanuCore/CorbanuTerminal/pull/321) | `9810e07e31` | 2026-10-08T21:27:40Z | Windows: keep the logon launcher's pipe ends out of other children (#307) |
| [#327](https://github.com/CorbanuCore/CorbanuTerminal/pull/327) | `7a11f9068b` | 2026-10-09T04:53:44Z | Windows: keep spawn_protected's stdout pipe out of other children (#320) |
| [#331](https://github.com/CorbanuCore/CorbanuTerminal/pull/331) | `62f997f3ba` | 2026-10-09T07:44:03Z | Windows: refuse deny-read profiles on the unelevated sandbox's tool path, with an actionable message (#300) |
| [#326](https://github.com/CorbanuCore/CorbanuTerminal/pull/326) | `6b94db40f4` | 2026-10-09T09:27:46Z | Windows: remove a deny-read entry only once its rule is gone; exact removal; none while a contract is armed (#304, #301) |
| [#343](https://github.com/CorbanuCore/CorbanuTerminal/pull/343) | `a141b3e749` | 2026-10-09T12:31:44Z | Windows sandbox: run commands over SSH (non-interactive window station) (#341) |
| [#317](https://github.com/CorbanuCore/CorbanuTerminal/pull/317) | `5933e2eb0d` | 2026-10-08T20:08:01Z | Fix Windows clippy -D warnings across the workspace; add a postmerge Windows clippy leg |
| [#313](https://github.com/CorbanuCore/CorbanuTerminal/pull/313) | `8131beefb2` | 2026-10-08T19:41:07Z | fix(core): keep provider API keys out of model-run commands (#310) |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#321 MERGED base=main merge=9810e07e31 checks: SKIPPED=10 SUCCESS=25; git merge-base --is-ancestor 9810e07e31 3254a302fd -> exit 0
#327 MERGED base=main merge=7a11f9068b checks: SKIPPED=10 SUCCESS=34; git merge-base --is-ancestor 7a11f9068b 3254a302fd -> exit 0
#331 MERGED base=main merge=62f997f3ba checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor 62f997f3ba 3254a302fd -> exit 0
#326 MERGED base=main merge=6b94db40f4 checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor 6b94db40f4 3254a302fd -> exit 0
#343 MERGED base=main merge=a141b3e749 checks: SKIPPED=10 SUCCESS=25; git merge-base --is-ancestor a141b3e749 3254a302fd -> exit 0
#317 MERGED base=main merge=5933e2eb0d checks: SKIPPED=9 SUCCESS=30; git merge-base --is-ancestor 5933e2eb0d 3254a302fd -> exit 0
#313 MERGED base=main merge=8131beefb2 checks: SKIPPED=10 SUCCESS=30; git merge-base --is-ancestor 8131beefb2 3254a302fd -> exit 0
```

Issues fixed (`gh issue view <n> --json state,closedAt`):

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#307](https://github.com/CorbanuCore/CorbanuTerminal/issues/307) | CLOSED | 2026-10-08T21:27:41Z | Windows: logon launcher pipe ends are briefly inheritable by other processes Core starts |
| [#320](https://github.com/CorbanuCore/CorbanuTerminal/issues/320) | CLOSED | 2026-10-09T04:53:45Z | Windows: spawn_protected's stdout pipe end is briefly inheritable by other processes Core starts |
| [#300](https://github.com/CorbanuCore/CorbanuTerminal/issues/300) | CLOSED | 2026-10-09T07:44:05Z | Windows: deny-read entries are not enforced on the tool path under the unelevated sandbox |
| [#301](https://github.com/CorbanuCore/CorbanuTerminal/issues/301) | CLOSED | 2026-10-09T09:28:05Z | Windows: a flag-off session on the same CODEX_HOME can revoke the launch contract's deny ACEs |
| [#304](https://github.com/CorbanuCore/CorbanuTerminal/issues/304) | CLOSED | 2026-10-09T09:27:48Z | Windows sandbox: persistent deny-read ACEs are never revoked (REVOKE_ACCESS leaves deny ACEs) |
| [#341](https://github.com/CorbanuCore/CorbanuTerminal/issues/341) | CLOSED | 2026-10-09T12:31:46Z | Windows: elevated sandbox commands time out over SSH (runner can't start on a non-interactive window station, 0xC0000142) |
| [#310](https://github.com/CorbanuCore/CorbanuTerminal/issues/310) | CLOSED | 2026-10-08T19:41:08Z | Provider API key env var (ZAI_API_KEY) is visible to the model's shell commands |

| PR | Fixes | Before → after (measured) | Regression tests | Review |
| --- | --- | --- | --- | --- |
| #321 | #307 | children holding a launcher pipe: 55 of 687 (elevated), 60 of 685 (normal) → 0 of 880, 0 of 879 | `sec_win_307_launcher_pipes_never_reach_other_children` | Opus 5.5 High APPROVE |
| #327 | #320 | children holding the broker stdout pipe: 25 of 102, 32 of 102 → 0 of 306, 0 of 306 | `sec_win_320_protected_stdout_never_reaches_other_children`, `sec_win_320_protected_child_names_its_spawner` | Opus 5.5 High APPROVE (PR comment) |
| #331 | #300 | main + tests: 2 fail (no refusal) → 2 pass, elevated and normal | `sec_win_300_unelevated_tool_path_refuses_deny_read_profiles`, `sec_win_300_unelevated_tool_launch_refuses_deny_read_profiles` | Opus 5.5 High APPROVE (see note) |
| #326 | #301, #304 | S1 test fails on the path-driven code → passes, elevated and normal (CI run 37897315520 and the real machine) | `sec_win_304_s1_a_secret_hidden_from_the_scan_keeps_its_deny`, `sec_win_304_entries_follow_their_rules` | Opus 5.5 High round 5 APPROVE |
| #343 | #341 | over SSH: timeout after 15 s → commands run, high and medium integrity | two `sec_win_341` tests (fail on main with `0xC0000142`, pass with the fix) | Opus 5.5 High round 2 APPROVE |
| #317 | — (found by sec-win3) | Windows `cargo clippy --workspace --all-targets -- -D warnings` failing → exit 0 | postmerge Windows clippy leg (cold run 37828036385, ~17 min) | Opus 5.5 High APPROVE |
| #313 | #310 | `exec_command` case fails on main (reproduces #310) → every key ABSENT | `exec/tests/suite/provider_key_env.rs` | Opus 5.5 High APPROVE (after 2 REQUEST_CHANGES) |

Note on #331: its PR description (quoted in full below) has no review line. The verdict comes from the review run kept on the operator Mac, not in the repository: `.codex-work/workers-20261002/sec-win7-review-300/review-output.txt`, whose last line is `VERDICT: APPROVE`.

## #321: logon launcher pipe ends never inheritable (#307)

Verbatim, PR #321 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/321), in full:

> Fixes #307.
>
> **Problem.** Core gave the logon launcher (#295) its two pipe ends by marking them inheritable in Core's own handle table until the launcher started. The launcher took only those two through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. But any process Core started on another thread in that window without a handle list (`std::process::Command` always inherits) got them too. That includes a sandboxed command running as another sandbox user. Such a process could read the launch request, which holds a sandbox user's password, or hold the reply pipe open.
>
> **Fix** (`windows-sandbox-rs/src/logon_launch.rs`): the pipe ends are never inheritable in Core.
> - Core starts a holder process: the launcher binary, suspended, inheriting nothing, environment `SystemRoot` only, never resumed.
> - Core copies the two ends into the holder as inheritable handles.
> - Core starts the launcher with `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` set to the holder, and a handle list and std handles that use the holder's values. It then ends the holder.
> - `ProcThreadAttributeList::set_parent_process` is new.
> - The launcher's token, job, desktop and environment are the same as before. The secondary logon service still opens the launcher.
>
> **Regression test** `sec_win_307_launcher_pipes_never_reach_other_children` (`logon_launch_tests.rs`):
> - It runs alone in a re-executed test process.
> - One thread makes 300 launcher starts while another starts ordinary `std` children, suspended.
> - Each child's handle table is checked for pipes other than the ones every child inherits.
>
> **Measured on a real Windows 11 machine**
>
> | | Elevated | Normal session (medium integrity) |
> |---|---|---|
> | Before (main) | **55 of 687** children held a launcher pipe (FAIL) | **60 of 685** (FAIL) |
> | After | 0 of 880 (pass) | 0 of 879 (pass) |
>
> - The #295 end-to-end probe (`pf_27_s06_d1`) passes in a normal session with this change (`via launcher: true`), with a throwaway local user that was then removed.
> - CI: `windows-security-probes` runs `sec_win_` elevated, and in the normal-session step through `run-at-medium-integrity.ps1`. A count check makes sure the tests exist.
>
> **Gate**
> - Windows clippy `-D warnings` on `codex-windows-sandbox`: clean.
> - Linux clippy `-D warnings` on the RTX box: clean.
> - Opus 5.5 High review: APPROVE. Its low findings are addressed:
>   - The comments now say exactly what the holder guarantees.
>   - The holder runs detached, and Core waits for it to end.
>   - The test proves it raced, so a rename can't make it pass silently.
> - The review noted that `spawn_protected` (the credential broker's start) has the same short window. Filed as #320.
>
> **Notes**
> - `windows-sandbox` lib tests in `unified_exec` that use the elevated runner fail on the QA machine, on main as well (`CreateProcessWithLogonW failed: 2`). That's local to that machine; CI is unaffected.
> - The holder plus parent-process pattern is also what some EDR products watch for (parent-PID spoofing). It's worth one run on a machine with Defender for Endpoint before release.

## #327: `spawn_protected` stdout pipe (#320)

Verbatim, PR #327 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/327), in full:

> Fixes #320.
>
> **Problem.** `spawn_protected` (it starts the credential broker) marked the broker's stdout write end inheritable in Core's own handle table for the length of `CreateProcessW`. The broker took only that handle through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. But a process Core started on another thread in that window without a handle list (`std::process::Command` always inherits) got it too. It could then hold the broker's stdout open or write into it. Same race as #307.
>
> **Fix:** the #307 approach (PR #321), now one shared helper.
> - **Shared helper.** `codex_process_hardening::HandleHolder` (`process-hardening/src/windows_handle_holder.rs`) is a process that never runs. It is created suspended, inherits nothing, has an empty environment and is ended on drop. `hold()` copies a handle into it as inheritable.
>   - The logon launcher (`windows-sandbox-rs/src/logon_launch.rs`) now uses this helper instead of its own copy. Its behaviour is unchanged apart from the holder's environment, which is now empty.
> - **`spawn_protected`:**
>   - The holder is made from the broker's own image, with the protected process and thread DACLs, so no other process of the user can take the pipe end out of it.
>   - The write end is copied into the holder and closed in Core.
>   - The broker starts with `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` = holder, and its handle list and stdout use the holder's value.
>   - The broker's protected descriptors, the suspended start and the token default-DACL step are unchanged.
> - **Broker's controller.** Windows now reports the holder as the broker's parent, but the broker used its parent's process id to find its controller. So `spawn_protected` adds `CODEX_PROTECTED_SPAWNER_PID` (Core's id) last in the child's environment, where a caller-supplied value can't override it.
>   - The broker (`pipe.rs`) reads it through `protected_spawner_pid()` and falls back to the parent's id.
>   - The existing check still applies: the controller must have started before the broker.
>
> **Regression tests** (`windows_protected_spawn_tests.rs`)
> - `sec_win_320_protected_stdout_never_reaches_other_children` re-runs itself alone in a fresh process. One thread makes 300 `spawn_protected` starts while another starts ordinary `std` children, suspended. Each child's handle table is checked for pipes other than the ones every child inherits.
> - `sec_win_320_protected_child_names_its_spawner`
>
> **Measured on a real Windows 11 machine**
>
> | | Elevated | Normal session (medium integrity) |
> |---|---|---|
> | Before (main + test) | **25 of 102** children held the broker's stdout pipe (FAIL) | **32 of 102** (FAIL) |
> | After | 0 of 306 (pass) | 0 of 306 (pass) |
> | After, interactive session | 0 of 319 | 0 of 323 |
>
> These also pass with the fix, elevated and at medium integrity in the interactive session:
> - The PF-27-S06/S07 process-access probes, including `pf_27_s07_protected_spawn_is_never_openable` and `_is_unopenable_while_suspended`.
> - The #307 race test (0 of 959 and 0 of 965).
> - The full `credential_broker::isolated::` suite over named pipes (24 of 24), which exercises the new controller lookup.
>
> The #295 logon-launch probe also passes at medium integrity, going through the launcher, with a throwaway user that was then removed.
>
> **CI:** `windows-security-probes` runs `sec_win_` for `codex-process-hardening` elevated, and in the normal-session step through `run-at-medium-integrity.ps1`. A count check (2 tests) makes sure the tests exist.
>
> **Gate**
> - Windows clippy `-D warnings` (process-hardening, windows-sandbox, network-proxy): clean.
> - Linux clippy on the RTX box: clean.
> - Opus 5.5 High review: see the comment below.

Verbatim, PR #327 review comment:

> Opus 5.5 High review (read-only, `corbanu exec`): **APPROVE**, no blocking issues. Addressed in e07c0b2d8a: the holder comment now says the job is shared rather than copied; the attribute-list comments are fixed; the #320 tests have their own CI step. Not changed: (1) no probe checks that the holder itself can't be opened; the pf_27_s07 probes cover only the broker. (2) A holder leaked by a Core crash can't be killed by the user, like a leaked suspended broker today. (3) Every broker start now uses the parent-process attribute, which EDR products flag as PPID spoofing; check it against Defender for Endpoint before release. (6) The handle scan stops at 0x10000, as in the #307 test.

## #331: unelevated sandbox refuses deny-read profiles on the tool path (#300)

Verbatim, PR #331 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/331), in full:

> Fixes #300 (Travis's decision, 2026-10-08: fail closed).
>
> **Change.** On the tool path (`SandboxAttempt::env_for`), a launch under the unelevated Windows sandbox is refused up front when its profile restricts reads: deny-read entries (exact paths or globs, user rules or PF-23 protected paths) or a limited set of readable roots. This uses the same read checks as `process_exec_tool_call` (`codex_sandboxing::refuse_unenforceable_windows_read_restrictions`). The refusal says what happened and how to fix it:
>
> > unsupported operation: the command was not run because this Windows sandbox mode (unelevated) can't block reading the files this permission profile protects. To run it, switch to the elevated Windows sandbox: run /setup-default-sandbox in the TUI, or set `sandbox = "elevated"` under `[windows]` in config.toml (for one run: `-c windows.sandbox=elevated`).
>
> `process_exec_tool_call` now gives the same message. No change for the elevated backend (#294's override attach is unchanged), macOS or Linux. The write-side checks of the unelevated resolver are deliberately not added to this path: on macOS the product's own `workspace-write` profile fails them, and they are not part of this decision.
>
> **What main actually did (measured).** The issue assumed these commands ran with the denied files readable. On the real machine they did not: the unelevated backend itself refuses any profile with a deny entry at spawn time, with an unclear error (`windows sandbox: Restricted read-only access requires the elevated Windows sandbox backend`). So this was a confusing late failure, not a leak. The fix moves the refusal ahead of any spawn preparation and makes it actionable.
>
> **Tests**
> - `sec_win_300_unelevated_tool_path_refuses_deny_read_profiles` (all platforms): `env_for` refuses a deny-read profile unelevated, still accepts the same profile without the deny entry, and the elevated launch still carries the deny entry.
> - `sec_win_300_unelevated_tool_launch_refuses_deny_read_profiles` (Windows, real sandbox): control run reads both files without a deny entry; with an exact-path deny and with a `**/*.env` glob deny, the denied file is never read and the launch gets the actionable refusal.
> - Both added to `windows-security-probes`, elevated and in the normal-session (medium-integrity) step.
>
> **Real Windows 11, elevated and normal session (medium integrity)**
> | | Elevated | Normal session |
> |---|---|---|
> | main + tests (fix reverted) | 2 fail: no refusal; backend error "Restricted read-only access requires…" for exact and glob | same, 2 fail |
> | this branch | 2 pass | 2 pass |
>
> Also on that machine: `windows_restricted_token_rejects_exact_and_glob_deny_read_policy` (integration), `exec::tests` (60) and `codex-sandboxing` (52) pass. Windows clippy `-D warnings` (codex-core, codex-sandboxing, all targets) clean; Linux clippy (RTX box) clean.
>
> **Docs.** `docs/sandbox.md` (Windows non-admin sandbox) explains that profiles that protect files from reading need the default (elevated) sandbox, and what the refusal looks like.

## #326: rule-driven, exact deny-read removal (#301, #304)

Verbatim, PR #326 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/326), in full:

> Fixes #304 and #301. Implements Travis's decision on S1 (2026-10-08): **option 1, rule-driven removal**.
>
> **What it does**
> - **#304, entries are actually removed, and exactly.** `acl::remove_deny_read_ace` removes exactly the entry the sync added (Windows splits it into `(D;;FR)` + `(D;OICIIO;0x80120089)` on a directory; both go). Only entries this `CODEX_HOME`'s sync added at a given path are owned, recorded by volume serial + 128-bit file ID; removal opens the object without following links, refuses hard-linked files and checks identity. The record lives in `.sandbox-secrets`, which the sandbox can't write. A failed apply records instead of rolling back. (Rounds 1–4.)
> - **S1, removal is driven by rules, not by what a scan saw.** The configured deny-read rules (exact paths and glob patterns, resolved against cwd) travel with their expanded paths: `codex_windows_sandbox::DenyReadTargets` → sandboxing crate (`WindowsSandboxFilesystemOverrides.additional_deny_read`) → core exec (the armed contract's denies are merged as rules) → unified exec / wrapper (`--deny-read-json`) / elevated capture → elevated setup payload (`deny_read`). The sync records which rule(s) produced each owned entry (`.sandbox-secrets\deny_read_acl_rules.json`) and removes an entry only once **none of its rules is configured any more**. A glob that matches nothing this launch is still listed, so a match hidden from the scan (folder held open, depth-limit junction) keeps its entry.
> - **Refreshes without rules change nothing.** Setup refreshes that don't carry a launch's rules (read-root refresh, first setup / `/setup-default-sandbox`, provisioning) pass `None`: no entry is added or removed. On main they passed an empty list, which would have meant "remove everything" once removal worked (the dependency noted on #304).
> - **#301:** while any process holds the armed contract's `.secretless-launch.lock`, nothing is removed; otherwise the sync takes it exclusively for the removal.
> - `corbanu sandbox` (debug, elevated) now lists and enforces the profile's rules, so it can't strip other sessions' entries. The setup payload also carries the flattened paths, so a mismatched older helper still applies them.
>
> **Evidence**
> - **S1 regression** `sec_win_304_s1_a_secret_hidden_from_the_scan_keeps_its_deny`: holds the folder open with share mode 0, asserts the scan really misses the secret, syncs, asserts the deny stays; removing the rule then removes it.
>   - Path-driven code (#326 at 750d813, test adapted to its API, temporary branch, `windows-security-probes` on windows-2022): **fails** elevated and in the normal session ("deny kept after the hidden scan: false").
>   - This branch: **passes** elevated and in the normal session (run 37897315520).
> - `sec_win_304_entries_follow_their_rules`: a `None` refresh keeps entries, a respelled rule is the same rule, a replaced rule removes the old entries.
> - All 13 `sec_win_` windows-sandbox tests and the core `pf_27_s07` tests pass elevated and at medium integrity on CI (windows-2022).
> - Fixed a test-only problem that made the elevated CI step fail since round 4: rewriting a DACL sets the `AI` control flag on GitHub's runners; the tests now compare DACL entries.
> - **Real Windows 11 QA machine, at f339a94 (elevated and normal session):** all 13 `sec_win_` windows-sandbox tests pass in both sessions (the two junction steps skip at medium, as before); core `pf_27_s07` and `sec_win_300` pass. The S1 test against the path-driven code fails there in both sessions ("deny kept after the hidden scan: false").
>   - Known machine issues, the same on main (checked at 8532483): elevated end-to-end launches time out connecting the runner pipe since the machine came back from being offline (`pf_27_s06_elevated_launch…`, `pf_27_s06_d2_vault…`), and five `unified_exec` tests fail. CI's windows-2022 runner runs those end-to-end probes, including real setup-helper syncs, elevated and at medium, and they pass. `pf_27_s07_armed_lock_refuses_links_and_deletion` needs the symlink privilege, which a normal session lacks.
> - Windows clippy `-D warnings` (windows-sandbox, sandboxing, core, cli; all targets) is clean on the QA machine. Linux clippy (RTX box) is clean.
>
> **Review (Opus 5.5 High), round 5: APPROVE.** Low findings addressed (payload compatibility with an older helper, an overclaiming doc comment). Accepted as designed: entries whose rule is removed while an attacker hides a match lose it with the rule; an attacker can make the sync forget an entry, which then stays forever (fails closed); rule keys fold ASCII case only.
>
> **Out of scope:** #323 (sessions on different `CODEX_HOME`s, or with different rule sets, removing each other's entries). This change doesn't fix it and doesn't make it worse than the path-based version.

## #343: Windows sandbox over SSH (#341)

Verbatim, PR #343 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/343), in full:

> Fixes #341.
>
> **Problem.** Corbanu Terminal started from a Windows OpenSSH session (or a service) couldn't run any command under the elevated (default) Windows sandbox. Each one failed after 15 s with `timed out after 15000ms connecting runner pipe-in`. This is the "QA machine broke after the Windows-update reboots" finding from sec-win7. It wasn't caused by the reboots: the earlier gates ran in the console session (scheduled tasks with `/it`), and over plain SSH the same failure shows on 2026-10-08 (win-gate1).
>
> **Cause, measured on the Windows 11 26200 QA machine:**
> - Over SSH, Core runs on the window station `Service-0x0-<logon id>$`. Its DACL grants only the SSH user and Administrators.
> - `CreateProcessWithLogonW` (no desktop name) starts the runner there, but the secondary logon service doesn't grant the sandbox user access. The runner loads user32 and dies with `0xC0000142` (System event 26). Core then waits 15 s.
> - Granting the sandbox user's SID `0x6E|READ_CONTROL` on the window station and `0xCF|READ_CONTROL` on the desktop fixes it. Without `READ_CONTROL` it still fails.
> - The commands the runner then starts died the same way. `LaunchDesktop` named their desktop `Winsta0\<name>`, but the runner created the private desktop in its own window station.
> - The old (9.5.5.1) and new (9.5.6.2) OpenSSH servers behave the same. Upstream: openai/codex#37722, #46412.
>
> **Fix:**
> - `window_station.rs`: before `CreateProcessWithLogonW`, when Core isn't on `WinSta0`, `create_process_with_logon` grants the sandbox user the least access the runner and its commands need. Each right was measured over SSH on a fresh window station:
>   - window station: `READ_CONTROL`, `WINSTA_READATTRIBUTES`, `WINSTA_ACCESSGLOBALATOMS`, `WINSTA_EXITWINDOWS`, `WINSTA_CREATEDESKTOP`;
>   - desktop: `DESKTOP_READOBJECTS`, `DESKTOP_WRITEOBJECTS`;
>   - not granted: hooks, windows, menus, clipboard, `WRITE_DAC`.
> - The grant is idempotent and checks for deny entries. If it fails, the launch still goes ahead; the error is logged and added to the pipe-connect error.
> - `desktop.rs`: off `WinSta0`, launch desktops are named in the process's own window station. `WinSta0` is unchanged.
> - The grant is for the sandbox accounts and lasts until the SSH session ends. Narrowing it to the runner's logon is #345.
>
> **Tests:**
> - Two `sec_win_341` tests. Each reruns itself in a child that moves to a fresh window station with an SSH session's name pattern and DACLs.
>   - The first starts `whoami.exe` on both launch desktops.
>   - The second checks that the grant is idempotent. Elevated, it also starts `whoami.exe` as a temporary local user (deleted afterwards) through `create_process_with_logon`.
>   - On `main` both fail with `0xC0000142`; with the fix both pass.
>   - A normal session may not create window stations (measured both over SSH and in the console session), so the medium runs skip them.
> - CI's `sec_win_` count goes from 13 to 15.
>
> **Evidence on the real machine:**
> - `corbanu sandbox -- whoami` / `powershell`, over SSH, in read-only and workspace-write:
>   - Before the fix: timeout.
>   - After: works, at high integrity and medium (seeded home).
> - `corbanu exec` (GLM 5.2) over SSH: runs `whoami` and writes a file in workspace-write, at high and medium integrity.
> - The console session still works.
> - Windows clippy `-D warnings`: clean.
> - Linux clippy: clean on the RTX box at the first commit; the RTX box has been offline since. The later commits touch only Windows-only modules, and PR CI's Linux clippy covers them.
> - `sec_win_` tests: 15/15 elevated and at medium, over SSH and in the console session.
> - Non-admin sandbox over SSH:
>   - `main` fails in both modes.
>   - With this PR, the default private-desktop mode works.
>   - With `sandbox_private_desktop = false` it still fails, as on `main`. Noted in #345.
> - Independent review (Opus 5.5 High): round 1 requested changes (masks, interactive path, CI skip, lows), all addressed; round 2 **APPROVE**, and its lows are fixed. The reviews are in `sec-win8-review-341*/` in the workers folder.

## #317: Windows clippy clean, postmerge Windows clippy leg

Verbatim, PR #317 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/317), in full:

> ## Problem
> `cargo clippy -- -D warnings` fails on Windows at `origin/main` (found by sec-win3). No CI job runs clippy for a Windows target: PR CI runs Linux gnu clippy only, postmerge `rust-ci-full` runs Linux gnu, Linux musl and musl release, and the Bazel Windows clippy leg is turned off. Two problems also fail on macOS with `--all-targets`: `codex-protected-state`, and the `wallet-daemon` example that CI's `--tests` run doesn't build.
>
> ## Warnings found
> I ran `cargo clippy --workspace --all-targets -- -D warnings` on a real Windows machine (x86_64-pc-windows-msvc, toolchain 1.95.0). Almost all of these are Unix- or Linux-only code that is dead or unused on Windows:
>
> - **codex-core** `model_broker_auth.rs` has 12. They are unused imports `MODEL_BROKER_FRAME_HEADER` and `HeaderValue`; unread `BrokerSettings` fields; three error variants that are never constructed; the unread `Source::ProviderKey.env_vars`; an unused `BrokerRewrite`/`for_url`; and unreachable code, unused variables and a redundant clone in `credential_for`.
> - **codex-core** `exec_policy.rs`: `strict_rules()` is never used (only Unix shell escalation reads it). Test-only problems: `LINUX_SANDBOX_SKIP_REASON` in core_test_support, unused imports in `suite/pf_23_s01.rs`, and `security/pf_29_s01_tests.rs`.
> - **codex-protected-state**: `Checkpoint`, `Binding`, `hash_valid` and the `impl Checkpoint` are never used. Only the Linux-only `store` and `native` modules use them, so this also fails on macOS.
> - **codex-windows-sandbox** `logon_launch.rs`: a redundant closure (`Vec::as_ptr`).
> - **codex-http-client**: `uninstall_model_broker_client` is test-only and its only caller is a Unix test.
> - **codex-telegram**: `SANDBOX_WARNING_DOCS` is never used.
> - **codex-wallet** `envelope.rs`: the `mode` argument is unused.
> - **codex-browser-isolation** `image_tests.rs`: the `assert_eq` import is unused.
> - **codex-tui**:
>   - `claude_panes/registry.rs`: unused `File` import.
>   - `claude_panes/containment.rs`: `arg0` is never read.
>   - `app_event.rs`: `UpdateAskForApprovalPolicy` is never constructed. An orphaned doc comment and a `cfg_attr(not(windows), allow(dead_code))` left behind by a removed variant were attached to it, so the warning was hidden everywhere except Windows. It is now `#[allow(dead_code)]` with the same note as the variant next to it.
>   - `claude_panes/tests.rs`: three imports are used only by Unix tests.
> - **codex-wallet-daemon** `examples/package_client_probe.rs`: `expect()` on an `Option`. This fails on every platform with `--all-targets`; the example now returns an `anyhow` usage error instead of panicking, so running it with no argument exits with code 1 instead of 101. That is the only behaviour change. Its callers, `qualify-rtx.sh` and `smoke_wallet_package.py`, always pass the argument.
>
> ## Fix
> Most items are now behind the right `cfg`: `cfg(unix)`, `cfg(target_os = "linux")` or `cfg(any(unix, test))`. Some code is deliberately shared across platforms, so I used a scoped `cfg_attr(not(unix), allow(..., reason = ...))` instead:
> - `BrokerSettings`, the three broker error variants, `Source::ProviderKey.env_vars` and the tui `arg0` field.
> - `credential_for`, where `Credential` is an uninhabited enum without the broker, so everything after a successful registration is statically unreachable on Windows.
>
> There are no workspace-wide or crate-wide allows. Apart from that example's exit code, behaviour is unchanged on every platform: the Linux and macOS builds compile the same code, and on Windows only dead items were removed.
>
> ## CI
> The smallest change that would catch this: one matrix entry in the postmerge `rust-ci-full.yml` `lint_build` matrix. It adds `windows-2022` / `x86_64-pc-windows-msvc` / `dev` and reuses `rust-ci-lint-build.yml`, which already handles Windows (Dev Drive, no musl/v8 steps) and its runner/target-keyed cargo-home and sccache caches. That entry has a 60-minute timeout; the other entries keep 30. It is added here. In a test dispatch of `rust-ci-full` on this branch, run 37828036385, the new Windows leg passed cold in about 17 minutes, so 60 minutes leaves room.
> It does not slow PRs. The shared job runs `--tests`, not `--all-targets`, so examples and benches (such as the wallet-daemon probe) stay unlinted in CI on every platform. I left that alone so the gating Linux PR leg doesn't change. If we want PR-time coverage like `lint_build_linux`, the same entry could go in `rust-ci.yml` once its warm runtime is known.
>
> ## Evidence
> - Windows (real machine), `cargo clippy --workspace --all-targets -- -D warnings`: was failing at main, now exit 0.
> - Linux (RTX box), `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
> - macOS, `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. At main, protected-state and the wallet-daemon example failed.
> - Focused tests on macOS pass: protected-state, http-client, telegram, wallet, browser-isolation, core lib (`model_broker_auth`, `exec_policy`, `security::pf_29_s01`), tui lib (`claude_panes`, `app_event`).
> - Focused tests on Windows: the same crates pass, and core `model_broker_auth`/`exec_policy` pass, including the Windows-only `pf_27_s05_non_unix_start_is_the_refusing_broker`. 5 `security::pf_29_s01` preflight tests fail on this Windows machine. This change only adds `#[cfg(unix)]` to one import in that test file, so the failures don't come from it (Windows tests are not in CI).
> - Linux: protected-state passes with `TMPDIR` on ext4. On tmpfs, `/tmp` gives `Unsupported` by design. `pf20_s03_torn_pending_and_symlink_replacements_latch_unavailable` is flaky when run in parallel (lock `Conflict`, 2 of 3 runs). Linux code is not changed by this PR.
> - Independent review (Opus 5.5 High, read-only): **APPROVE**, no blockers. It confirmed every cfg against its call sites. It raised two should-fix items: the Windows timeout, which the 17-minute cold run answers, and the `--tests`/`--all-targets` gap, which is noted above. Its nits were the example's exit code (noted above) and optionally deleting the never-constructed `UpdateAskForApprovalPolicy` variant later.

## #313: provider keys kept out of model-run commands (#310)

Verbatim, PR #313 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/313), in full:

> Fixes #310.
>
> **Change record.** Security repair that narrows protected-data disclosure (provider credentials stop reaching model-run commands); it adds no new disclosure.
> - Product spec, `# Corbanu Terminal` → `## Product definition`: "permit action without exposing strategy, credentials, or financial information."
> - `# P0 /security levels` → `## Non-negotiable controls`: "Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging."
> - Product decision: the task owner (security program, issue #310 assignment) required the filter at every level including Permissive, and an explicit opt-in through the existing `shell_environment_policy` include/set. A trusted project's `.codex/config.toml` can also set that policy; that adds no new route, since trusted project config can already pass a key to an MCP server through `env_vars`.
>
> **Problem.** Provider API keys in Corbanu's environment (e.g. `ZAI_API_KEY`) reached commands the model runs. `shell_command` filtered a hand-kept list. Unified exec (`exec_command`) didn't filter at all, and neither did the exec-server, which rebuilds the environment from its own process env. Shell snapshots could also re-export a configured key whose name doesn't look like a secret.
>
> **Fix.**
> - Blocked names come from the built-in provider table (`built_in_provider_api_key_env_vars`, incl. aliases such as `CORBANU_API_KEY`), plus other credentials Corbanu reads (`OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN`, `AZURE_OPENAI_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN`, `AWS_BEARER_TOKEN_BEDROCK`), plus every configured provider's `env_key`. This replaces the old hand list, which missed `KIMI_API_KEY`, `DEEPSEEK_API_KEY`, `CORBANU_API_KEY` and others.
> - Unified exec builds its env with the same filter and adds the blocked names to the exec-server policy's `exclude`.
> - The shell-snapshot shell starts without these names, and the snapshot scripts never export them (only plain identifiers are added, matched case-insensitively), even when `~/.zshrc` or `~/.bashrc` sets them.
> - Applies at every security level. The provider client still reads the variable.
> - Opt-in: an exact (wildcard-free) `include_only` or `filters = include` entry passes Corbanu's value through. A `set` entry supplies only its own value. Wildcards never pass a provider key.
> - MCP stdio servers already receive only a basic allowlist plus the server's own `env_vars`/`env`; unchanged. Vault and onboarding never put keys into the process env. `CODEX_HOME/.env` does, and this filter covers it.
> - Docs: `docs/config.md#shell-environment`, `docs/integrations/zai-glm-52.md`.
>
> **Gate evidence.**
> - Focused tests (macOS, `just test`): 570 passed across model-provider-info, exec, and the exec_env/unified_exec/shell/snapshot/security filters. Linux (RTX box, isolated harness): 121 snapshot/provider_key_env/exec_env tests passed.
> - `exec/tests/suite/provider_key_env.rs`: `corbanu exec` runs with `CORP_LLM_CRED` (the mock provider's own `env_key`), `ZAI_API_KEY`, `OPENROUTER_API_KEY` and `CODEX_API_KEY` set. Under `shell_command` and `exec_command`, with login and non-login shells, every key reports ABSENT, and every request carries `Bearer` with the provider key. An exact `include_only` entry reports PRESENT. On main the `exec_command` case fails, which reproduces #310. Without the snapshot fix, the login cases fail.
> - Linux clippy `-D warnings` (codex-core, codex-model-provider-info, codex-exec) on the RTX box: clean at 4cd36164c.
> - Independent review (Opus 5.5 High, read-only `corbanu exec`): REQUEST_CHANGES twice (snapshot path, extra credential names, case handling for `set`), all addressed; final verdict APPROVE.
> - Functional check: a fresh debug binary in a disposable home (`CORBANU_TEST_NO_NATIVE_KEYRING=1`) ran `corbanu exec -s read-only -m glm-5.2` against real Z.AI, with the key supplied only through `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"`. It reported `ABSENT` with unified exec on and off, and no key material appeared in the outputs.

The rule as stated in #313 (quoted above): blocked names are "plus every configured provider's `env_key`"; it "Applies at every security level. The provider client still reads the variable."; the only opt-in is "an exact (wildcard-free) `include_only` or `filters = include` entry"; "Wildcards never pass a provider key."

## Open, not done

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#323](https://github.com/CorbanuCore/CorbanuTerminal/issues/323) | OPEN | — | Windows: an armed contract's deny-read entries are not protected from another CODEX_HOME's sync |
| [#345](https://github.com/CorbanuCore/CorbanuTerminal/issues/345) | OPEN | — | Windows sandbox over SSH: scope the window-station grant to the runner's logon and its lifetime |

- PF-27-S08 (in progress) and PF-27-S09 (planned).
- #321/#327: the holder plus parent-process pattern may be flagged by EDR products as parent-PID spoofing; a run on a machine with Defender for Endpoint is suggested before release (stated in both PRs).
