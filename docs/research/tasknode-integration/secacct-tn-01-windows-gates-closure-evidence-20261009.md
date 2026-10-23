# Task Node evidence record: SECACCT-TN-01

Compile the PF-27-S06 and S07 Windows Gate Closure Evidence Record. Task `task_95e3a96c397ef87cbbba10a3036f91ef`, request `req_c40ba35afbad67a117e4a553581bdce33c6f9425aaf39d186531f7c7b13315e5`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: completed and accepted.** Travis accepted PF-27-S06 and PF-27-S07 **with known limits** on 2026-10-08; both records were archived by PR #329. The real-Windows gates ran on a Windows 11 Pro machine in a normal (medium-integrity) session with GLM 5.2 driving the TUI in tmux. Earlier task P0SEC-TN-10 (rewarded) covered PF-27-S06's code with a partial gate; this record covers the defects found by the first real-Windows run, the rerun, PF-27-S07 and the archive. **Open, not claimed:** #323 (an armed contract's deny entries vs another `CODEX_HOME`'s sync); PF-27-S08 and S09; the milestone code-blind VM run, human sign-off and flag removal. The post-archive fixes #321, #327, #331, #326 and #343 are a separate task (SECACCT-TN-02).

Records: [qa/security-levels/sprints/PF-27-S06/README.md](../../../qa/security-levels/sprints/PF-27-S06/README.md), [qa/security-levels/sprints/PF-27-S07/README.md](../../../qa/security-levels/sprints/PF-27-S07/README.md), [docs/sprints/archive/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md](../../sprints/archive/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md), [docs/sprints/archive/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md](../../sprints/archive/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md). Flags: `isolated_credential_broker`, `secretless_agent_launch` (default off).

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#298](https://github.com/CorbanuCore/CorbanuTerminal/pull/298) | `b2ac6e6f00` | 2026-10-08T15:14:43Z | PF-27-S06 #294: apply the launch contract's deny entries as Windows ACLs on the tool path (vault readable under workspace-write) |
| [#302](https://github.com/CorbanuCore/CorbanuTerminal/pull/302) | `661b5c6a48` | 2026-10-08T16:53:17Z | PF-27-S06 #295: start the elevated sandbox's runner from a hardened Core (normal-session agent commands); CI medium-integrity probes |
| [#312](https://github.com/CorbanuCore/CorbanuTerminal/pull/312) | `8a566c88c4` | 2026-10-08T18:09:21Z | PF-27-S06: real-Windows gate rerun passes (normal session), videos |
| [#280](https://github.com/CorbanuCore/CorbanuTerminal/pull/280) | `a3d470a8a5` | 2026-10-08T11:17:10Z | PF-27-S07 record; PF-27-S08/S09 drafts in the P1 plan |
| [#281](https://github.com/CorbanuCore/CorbanuTerminal/pull/281) | `23838bbe2a` | 2026-10-08T12:37:42Z | PF-27-S07 slice 1: remove the CODEX_HOME deny when secretless_agent_launch is off |
| [#282](https://github.com/CorbanuCore/CorbanuTerminal/pull/282) | `97d8ae109e` | 2026-10-08T11:53:33Z | PF-27-S07 slice 2a: Core and broker threads protected from creation |
| [#284](https://github.com/CorbanuCore/CorbanuTerminal/pull/284) | `f755628967` | 2026-10-08T12:28:34Z | PF-27-S07 slice 2b: Core starts the broker never openable |
| [#316](https://github.com/CorbanuCore/CorbanuTerminal/pull/316) | `f4da7b2674` | 2026-10-08T19:56:42Z | PF-27-S07: real-Windows gate run (pass) and videos |
| [#329](https://github.com/CorbanuCore/CorbanuTerminal/pull/329) | `8165c1375f` | 2026-10-09T05:36:01Z | docs(security): archive PF-27-S06 and PF-27-S07 (accepted with known limits) |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#298 MERGED base=main merge=b2ac6e6f00 checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor b2ac6e6f00 3254a302fd -> exit 0
#302 MERGED base=main merge=661b5c6a48 checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor 661b5c6a48 3254a302fd -> exit 0
#312 MERGED base=main merge=8a566c88c4 checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 8a566c88c4 3254a302fd -> exit 0
#280 MERGED base=main merge=a3d470a8a5 checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor a3d470a8a5 3254a302fd -> exit 0
#281 MERGED base=main merge=23838bbe2a checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor 23838bbe2a 3254a302fd -> exit 0
#282 MERGED base=main merge=97d8ae109e checks: SKIPPED=10 SUCCESS=25; git merge-base --is-ancestor 97d8ae109e 3254a302fd -> exit 0
#284 MERGED base=main merge=f755628967 checks: SKIPPED=10 SUCCESS=27; git merge-base --is-ancestor f755628967 3254a302fd -> exit 0
#316 MERGED base=main merge=f4da7b2674 checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor f4da7b2674 3254a302fd -> exit 0
#329 MERGED base=main merge=8165c1375f checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 8165c1375f 3254a302fd -> exit 0
```

Defects fixed by #298 and #302:

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#294](https://github.com/CorbanuCore/CorbanuTerminal/issues/294) | CLOSED | 2026-10-08T15:14:45Z | Windows: vault store readable by agent commands under workspace-write with secretless_agent_launch (contract deny entries never applied as ACLs) |
| [#295](https://github.com/CorbanuCore/CorbanuTerminal/issues/295) | CLOSED | 2026-10-08T16:53:19Z | Windows: every agent command fails with CreateProcessWithLogonW 5 in a normal session when secretless_agent_launch is on |

## PF-27-S06: first real-Windows run (failed) and the rerun (pass)

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 89-150:

> ## Real Windows gate run (2026-10-08): FAILED, two defects
>
> Host: Windows 11 Pro 25H2 (build 26200), 16 cores, 31 GB, local admin `User`, logged on at the console. Candidate:
> debug build of `origin/main` at `63ea3d0cbd0c` (built on the host). GLM 5.2 (`zai`) drove the real TUI in tmux on a
> fresh `CODEX_HOME` per run. Toolchain: Rust 1.95.0 MSVC, VS 2022 Build Tools 17.14 (MSVC 14.44.35207, SDK
> 10.0.26100), MSYS2 (runtime 3.6.10) with tmux 3.7c and Python 3.12.15, asciinema 2.4.0, agg 1.9.0, ffmpeg 9.0.2,
> Git 2.54.0, gh 2.93.0.
>
> | Check | Result |
> | --- | --- |
> | `pf_27_s06` suites on the host (elevated session, like CI): process-hardening 9, broker over pipes 24, core 5 | pass |
> | Unelevated sandbox + contract: agent command refused with the stated reason | pass |
> | Broker pipes, same-user foreign process (the user's `!` command) | data pipe: connects, dropped with 0 bytes received; control pipe: no free instance |
> | Broker pipes, agent command (sandbox user `CodexSandboxOffline`) | data pipe open denied; control pipe: no free instance |
> | Elevated sandbox + contract, product `workspace-write` profile: `auth.json`, `config.toml`, `*state*.sqlite`, writing `config.toml` | denied |
> | Same probe: vault store `CODEX_HOME\secrets\local.age` (created before start or during the run) | **READ: defect 2** |
> | Control, contract off: vault, `auth.json`, `config.toml`, state database | all read |
> | Elevated sandbox + contract in a normal (medium-integrity) session | **every command fails: defect 1** |
>
> **Defect 1: unusable in a normal session.** Started from the user's shell (medium integrity) with
> `[windows] sandbox = "elevated"` and `secretless_agent_launch` on, every agent command fails with
> `windows sandbox: CreateProcessWithLogonW failed: 5`. The same profile works with the flag off (broker on or off)
> and from an elevated session. The Security log has no logon by the sandbox user, so the secondary logon service
> refuses before logging on. Likely cause: the contract arms `restrict_current_process_access()` (user keeps only
> query-limited and synchronize on Core), and the service opens the caller's process while impersonating it.
>
> **Defect 2: the vault store is readable.** Under the product's `workspace-write` profile the contract's deny
> entries are never applied as ACLs (`.sandbox\deny_read_acl_state.json` stays empty; the sandbox log shows only
> the `.git` deny). `auth.json`, `config.toml` and the databases are denied only by the files-only deny on
> `CODEX_HOME`, which does not reach `secrets\`. The CI probe passes because it uses a restricted-read base profile.
>
> CI missed both because the `windows-2022` runners are elevated and the probe does not use the product profile.
>
> ### Videos
>
> [qa/demos/index/PF-27-S06.md](../../../qa/demos/index/PF-27-S06.md); specs `qa/demos/specs/pf27s06-win-*.toml`. Leak
> scan: no credential value or key-shaped string in any cast (script scan and a second scan of the published files),
> and no redactions in private logs.
>
> | Video | Session | Shows |
> | --- | --- | --- |
> | `pf27s06-win-protected-files` | elevated | defect 2 next to the denials |
> | `pf27s06-win-baseline-flag-off` | elevated | control: everything readable without the contract |
> | `pf27s06-win-unelevated-refused` | normal | refusal with the reason |
> | `pf27s06-win-broker-pipe-foreign-client` | normal (contract off) | foreign clients dropped or denied |
> | `pf27s06-win-normal-session-launch-fails` | normal | defect 1 |
>
> ### How it was run (for reruns)
>
> - The recorder works under MSYS2 tmux; native console programs run inside tmux panes through ConPTY. asciinema 3
>   has no Windows build, so a shim maps the script's asciinema 3 flags to asciinema 2.4.0 (MSYS2 Python) and shifts
>   its events by asciinema's start-up delay so cuts and holds line up. Publishing ran on macOS from the copied run
>   directories. Shims and job scripts: `.codex-work/workers-20261002/win-gate1/` (not in the repository).
> - The elevated sandbox's runner does not start from an SSH session (`timed out ... connecting runner pipe-in`), so
>   runs were started in the console session by scheduled tasks: elevated runs directly, normal-session runs through
>   `explorer.exe` so they get the user's ordinary token. MSYS2's `TEMP` was pointed back at the user's temp folder.
> - The elevated sandbox's admin setup is stored per `CODEX_HOME` and needs a UAC answer in a normal session. Normal-
>   session runs were seeded with the marker and sandbox-user record of the latest elevated setup (same user, same
>   DPAPI scope, the setup's deny on `.sandbox-secrets`). Defect 1 reproduces identically on a profile set up by the
>   product itself, and in `corbanu exec` started from `cmd.exe` without MSYS2.
> - Each run's directory names showed in the TUI's cwd; one GLM reply remarked on the folder name. No tool call or
>   result was affected.

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 152-183:

> ## Gate rerun after the fixes (2026-10-08): PASS
>
> Same host. Candidate: debug build of `origin/main` at `661b5c6a48cd`, which includes #298 (fixes #294) and #302 (fixes
> #295). Every run below was in a **normal session**: medium integrity, started through `explorer.exe`, and the
> recorder printed `Medium Mandatory Level`.
>
> | Failed item | Rerun |
> | --- | --- |
> | Defect 1: agent commands fail with `CreateProcessWithLogonW failed: 5` | **pass**: `whoami` runs as `CodexSandboxOffline`. The sandbox log records `runner started through the logon launcher (protected process, #295)` |
> | Defect 2: vault store readable under `workspace-write` | **pass**: vault store, `auth.json`, `config.toml` and the state database are denied, and so is writing `config.toml`; the workspace control file is read |
> | `pf_27_s06_d1` (#295 probe), normal session | pass: the fallback ran (`via launcher: true`) |
> | `pf_27_s06_d2` (#294 probes, product profile through the tool path), normal session | pass: 2 tests. The vault is denied to a protected launch, to an unprotected launch from an armed process, and to a command still running across it |
>
> Before the fixes, the same probes failed on this host: `pf_27_s06_d1` in a normal session, from an elevated session
> through the CI script, and (with PF-27-S07 merged) even elevated; both `pf_27_s06_d2` tests in both sessions. CI
> now covers this: `windows-security-probes` runs the probes again with the token UAC gives an administrator's normal
> session (`.github/scripts/run-at-medium-integrity.ps1`). What CI does not cover: a separate non-administrator
> account, and the elevated setup itself from a normal session (it needs a UAC answer, so the step reuses a setup
> recorded just before).
>
> Videos ([index](../../../qa/demos/index/PF-27-S06.md); leak scan: no credential value in any cast, screen or log, checked
> by the recorder and again on macOS):
>
> | Video | Session | Shows |
> | --- | --- | --- |
> | `pf27s06-win-normal-session-works` | normal | agent command runs as the sandbox user (defect 1 fixed) |
> | `pf27s06-win-vault-denied` | normal | vault read denied under `workspace-write`, next to the other denials (defect 2 fixed) |
>
> Follow-ups filed while fixing: #300 (unelevated tool path doesn't enforce deny-read; needs a product decision),
> #301 (a flag-off session on the same `CODEX_HOME`), #304 (deny ACEs are never revoked), #307 (launcher pipe
> inheritance window). The rerun used the same harness as the first run. The normal-session runs were seeded from the
> setup of an elevated run made just before, because each elevated setup resets the sandbox passwords.

Regression tests for the defects, named in the rerun table above: `pf_27_s06_d1` (#295 probe, fails before #302 and passes after, normal session) and the two `pf_27_s06_d2` tests (#294, product profile through the tool path). `windows-security-probes` now also runs them at medium integrity (`.github/scripts/run-at-medium-integrity.ps1`).

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 56-75:

> ## Known limits
>
> - **`CODEX_HOME` ACL change (contract).** The files-only deny for `CodexSandboxUsers` stays on `CODEX_HOME` after
>   the flag is turned off (sandboxed commands then cannot read files directly in `CODEX_HOME`); reset or uninstall
>   notes should mention it. A file *moved* into `CODEX_HOME` from another directory keeps its own ACL, so protected
>   writers must create temporary files inside `CODEX_HOME` (the `auth.json` storage does). Until the elevated
>   sandbox's setup has created its users group, protected launches are refused.
> - **Threads (DACL).** A new thread has the default DACL until its TLS callback runs, and threads created without
>   loader notifications keep it; a same-user unsandboxed process could open such a thread in that window. Commands
>   under the elevated sandbox are another user and are denied either way. Needs Travis's acceptance (review 3).
> - **Same-user unsandboxed processes** (MCP servers, hooks) can connect to the broker pipes and be dropped (denial of
>   service, not disclosure), as PF-27-S02 already declares them not contained.
> - **Broker containment** on Windows: no file-write confinement, no protection of other processes from the broker,
>   and requests through WMI, Task Scheduler or COM are possible. A restricted or AppContainer token for the broker is
>   the follow-up.
> - **Model auth** (PF-27-S05) stays Unix-only; Windows `broker_model_auth` refuses as before.
> - In-process file tools (structured edits, image view) get no file access on Windows under the contract; patches
>   work only under the elevated sandbox; external agents (Claude panes) stay refused on Windows.
> - Debug-privileged administrators and `SYSTEM` bypass any DACL. Credential Manager and DPAPI blobs are bound to the
>   real user and are not readable by the sandbox user.

## PF-27-S07: what shipped, reviews, real-Windows gate

Verbatim, `qa/security-levels/sprints/PF-27-S07/README.md` lines 9-15:

> ## What shipped
>
> | Slice | PR | What |
> | --- | --- | --- |
> | 1 | #281 | **`CODEX_HOME` deny removed when the flag is off.** The first config load of a process that has `secretless_agent_launch` off and no armed contract removes exactly the entry `add_deny_read_ace_for_new_files` adds: an explicit deny for `CodexSandboxUsers`, flags OI\|IO\|NP, with a mask that maps to file read. The copies that files directly in `CODEX_HOME` inherited go with it. Every other entry stays, and so does the DACL's protection. Each contract holds `CODEX_HOME\.secretless-launch.lock` shared. A removal takes it exclusively without waiting, so it never runs under an armed process. A contract without the lock refuses protected launches (`CodexHomeLockUnavailable`). The lock file must be a regular file with one link; it cannot be deleted while open and gets its own explicit deny for the sandbox's users. |
> | 2a | #282 | **New threads protected from creation.** In Core and the broker, `restrict_current_process_access` redirects this image's `CreateThread` imports to a wrapper that passes the protected thread descriptor, so every Rust thread (std, tokio) is protected at creation. The broker also sets its token's default DACL to the protected thread DACL, which covers every thread whoever starts it. |
> | 2b | #284 | **The broker is never openable.** Core starts the broker with `spawn_protected`: process and first thread are created with the protected DACLs, suspended, and the token's default DACL is set before the broker runs. It has no console and inherits only its stdout pipe. |

Verbatim, `qa/security-levels/sprints/PF-27-S07/README.md` lines 62-70:

> ## Reviews (Opus 5.5 High)
>
> | Slice | Rounds |
> | --- | --- |
> | 1 | Changes requested (a lock a sandboxed process could hold, fail-open without the lock, cheap check), then approve with lows (an explicit deny on the lock file, retry, hard links), then approve with lows. Waiting on the launch path removed; error message and removal-order test added. |
> | 2a | Changes requested (fail-safe ordering, self-rights, positive controls), then approve. Page protection is now restored in reverse order. |
> | 2b | Changes requested (`=C:` variables), then approve with lows. All fixed: full denial asserted after the broker's hardening, plus a comment on the control and a missing SAFETY comment. |
>
> Texts: [reviews/](../../../qa/security-levels/sprints/PF-27-S07/reviews/).

Verbatim, `qa/security-levels/sprints/PF-27-S07/README.md` lines 72-146:

> ## Real Windows gate run (2026-10-08): PASS
>
> Host: the PF-27-S06 machine (Windows 11 Pro 25H2, build 26200; local admin `User` at the console; same toolchain).
> Candidate: debug build of `origin/main` at `265172beed3d`, which has #281, #282 and #284 and the PF-27-S06 fixes
> #298 and #302. Every run below was in a **normal session**: medium integrity, no `SeDebugPrivilege`, started
> through `explorer.exe`. Config: `[windows] sandbox = "elevated"`, `workspace-write`, `isolated_credential_broker` and
> `secretless_agent_launch` on (off where stated). GLM 5.2 (`zai`) drove the TUI in tmux on a fresh `CODEX_HOME` per run.
>
> The probe (`threadprobe`, C# source in the specs) only opens handles. It tries 11 process rights and 5 thread rights
> (`GET_CONTEXT`, `SET_CONTEXT`, `SUSPEND_RESUME`, `TERMINATE`, `QUERY_INFORMATION`) on every thread. Its control is a
> child it starts itself, which it can always open.
>
> | Check | Result |
> | --- | --- |
> | Core, same-user process (the user's `!` command) | **pass**: only `QUERY_LIMITED` and `SYNCHRONIZE` (the protected DACL's user entry); 0 of 245 thread opens (49 threads) |
> | Core, agent command (sandbox user `CodexSandboxOffline`) | **pass**: no process right; 0 of 255 thread opens |
> | New threads: a same-user watcher polls Core and the broker during an agent turn and opens each new thread as soon as it appears | **pass**: 6 new Core threads and 2 new broker threads, none opened |
> | Broker (started with `spawn_protected`), same-user process | **pass**: only `QUERY_LIMITED` and `SYNCHRONIZE`; 0 of 40 thread opens |
> | Broker, agent command | **pass**: no process right; 0 of 40 thread opens |
> | `CODEX_HOME` deny, SDDL (script): after a flag-off run, then a flag-on run, then a flag-off `corbanu features list` | **pass**: the flag-on run adds `(D;OINPIO;0x80120089;;;<CodexSandboxUsers>)` to `CODEX_HOME`, and files in it (one created before, one after) get `(D;ID;FR;...)`. After the flag-off process, the SDDL of `CODEX_HOME` and of the earlier file match the baseline exactly. A second flag-off process changes nothing |
> | Same in the TUI: flag-off session after a protected `corbanu exec` | **pass**: the entry is gone from `CODEX_HOME` and `config.toml` |
> | A flag-off process while a protected session holds the lock | **pass**: the deny stays |
> | #295 regression: agent commands in a normal session | **pass**: `whoami` runs as `CodexSandboxOffline` |
> | #294 regression: vault under `workspace-write` | **pass**: the vault store, `auth.json`, `config.toml` and the state database are denied, and so is writing `config.toml`; the workspace control file is read |
> | Probe suites, elevated (as CI): process-hardening 18 + 1 (`pf_27_s06_d1`), windows-sandbox 4, broker 24, core 12 | pass |
> | Same suites in a normal session | pass, except 2 core tests that CI also runs only elevated (below) |
>
> The two core tests that need an elevated session:
> - `pf_27_s06_elevated_launch_cannot_read_protected_files_or_core_memory` runs the elevated sandbox's admin setup,
>   which needs a UAC answer (`ShellExecuteExW ... 1223`).
> - `pf_27_s07_armed_lock_refuses_links_and_deletion` can't create its symbolic link without
>   `SeCreateSymbolicLinkPrivilege` (error 1314). The same limit stops a normal-session process from planting the link,
>   unless Developer Mode is on.
>
> Both pass elevated. Neither is a product failure.
>
> **What CI covers and what it doesn't.** `windows-security-probes` (on `windows-2022`) proves two things
> deterministically:
> - a thread can't be opened while it is held in its creation window;
> - the broker can't be opened while it is still suspended.
>
> CI runs all `pf_27_s07` tests elevated, with privileges disabled in the probe. Of the PF-27-S07 work, it runs only the
> #294/#295 probes at medium integrity. It does not cover:
> - the running product's real Core and broker;
> - a medium-integrity session (outside those two probes);
> - the TUI.
>
> This run covers those, but its watcher can't prove it reached the creation window. The deny-removal SDDL check and
> the lock are covered by both. Neither covers a separate non-admin account, or the admin setup from a normal session.
>
> **Videos** ([index](../../../qa/demos/index/PF-27-S07.md); specs `qa/demos/specs/pf27s07-win-*.toml` and the PF-27-S06
> regression specs). Leak scan: no key value and no key-shaped string in any cast, frame, log or published file (the
> recorder's scan, and a second scan on macOS of the run folders and the downloaded release assets).
>
> | Video | Shows |
> | --- | --- |
> | `pf27s07-win-core-threads-unopenable` | Core and its new threads unopenable by the user's other processes and by the sandbox |
> | `pf27s07-win-broker-unopenable` | the same for the broker |
> | `pf27s07-win-codex-home-deny-flag-off` | a flag-off session removes the deny an earlier protected run left |
> | `pf27s07-win-codex-home-deny-kept-while-armed` | a flag-off process leaves it while a protected session runs |
> | `pf27s06-win-normal-session-works` | #295 regression |
> | `pf27s06-win-vault-denied` | #294 regression |
>
> **How it was run.** Harness as for PF-27-S06 (scripts in `.codex-work/workers-20261002/win-gate-s07/`, not in the
> repository). Normal-session runs reuse the result of one elevated setup run just before. The
> `codex-home-deny-flag-off` launch wrapper runs one protected `corbanu exec` in the run's `CODEX_HOME` before the TUI
> starts, and writes what it left to `before.txt`, which the video prints.
>
> Notes:
> - The removal's `info` log line never reaches `codex-tui.log`, because config loads before the file logger starts.
>   The SDDL is the evidence.
> - The seed wrapper's `.sandbox-secrets` deny had silently failed under MSYS2 path conversion, also in the PF-27-S06
>   runs. It's fixed in this harness. The product's own setup was unaffected.
> - `QUERY_LIMITED` and `SYNCHRONIZE` on the broker for the same user are by design (`process_dacl_sddl`); the sprint's
>   "every process right" means the rights its tests probe (`PROCESS_RIGHTS`), which don't include those two.

Verbatim, `qa/security-levels/sprints/PF-27-S07/README.md` lines 50-60:

> ## Known limits
>
> - **Core's threads started by other modules.** Windows thread pools, RPC and injected DLLs still get the protected DACL only in their TLS callback. Loader workers skip that callback and keep the default DACL. Core cannot change its token's default DACL: its child pipes, created without a descriptor and then reopened, and its child processes would inherit it. The broker has no such gap.
> - **Protected threads cannot change their own DACL or impersonate.** Nothing in Core or the broker does either.
> - **Broker stdout inheritance.** The broker's stdout write end is inheritable only for the duration of the `CreateProcessW` call, so a process `std` spawns at the same moment could inherit it. That pipe carries only the public pipe name, and Core checks the server's process ID, so the worst case is a denial of service.
> - **Mixed versions.** A PF-27-S06 build holds no lock, so a PF-27-S07 build with the flag off can remove the deny under it. Restart protected sessions before turning the flag off.
> - **Interrupted removal.** If propagation fails partway, some root files keep an inherited copy. This fails safe: those files stay denied to the sandbox, and nothing retries.
> - **Lock-file creation window.** If an armed contract creates the lock file before the sandbox group exists, or before its deny is applied, a sandboxed command could open it in that moment. Holding it shared only blocks cleanup (fails safe). Holding it exclusively makes protected launches refuse (fails closed).
> - **The lock file stays.** It keeps its own explicit deny after the removal.
> - **Lock path occupied.** A hard link or a symbolic link at the lock path makes protected launches refuse until the link is deleted.
> - **Real-Windows gate.** Done on 2026-10-08; see [Real Windows gate run](../../../qa/security-levels/sprints/PF-27-S07/README.md#real-windows-gate-run-2026-10-08-pass).

## Acceptance and archive (#329)

Verbatim, `docs/sprints/archive/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md` lines 35-50:

> ## Closure — 2026-10-08
>
> Completed. Travis **accepted** PF-27-S06 **with known limits** on 2026-10-08 (in chat with the coordinator): the
> limits are the [Known limits](../../../qa/security-levels/sprints/PF-27-S06/README.md#known-limits) list in the
> evidence README. Received and archived by the P1 integration owner; gate rerun evidence merged in PR #312.
>
> Follow-ups (linked, not blockers):
>
> - #300 (deny-read not enforced on the unelevated tool path): decided, fail closed; being implemented.
> - #301 and #304 (deny entries revoked by a flag-off session / never revoked): decided, rule-driven removal; draft PR #326.
> - #307 (launcher pipe inheritance) fixed by PR #321; #320 (`spawn_protected` stdout pipe) fixed by PR #327.
> - #323 (an armed contract's deny entries vs. another `CODEX_HOME`'s sync): open.
> - [PF-27-S08](../../sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (broker token) and
>   [PF-27-S09](../../sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) (model auth): planned.
> - Unplaced: file tools other than patches under the Windows contract; elevated-sandbox profile reads
>   (`~/.git-credentials`, `.ssh`, `.npmrc`, `.config/gh`); both in the plan's carried-forward table.

Verbatim, `docs/sprints/archive/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md` lines 103-110:

> ## Verification
>
> - [x] `windows-2022` (job 113190329753): process-hardening 9, broker and pipe suite 24, core 5 `pf_27_s06` tests
>   pass. Linux (RTX box): clippy `-D warnings` clean; broker 55, process-hardening 7, core 12 + 23 pass. macOS: same.
> - [x] GLM 5.2 tmux run and videos on real Windows: the first run (2026-10-08) **failed** with two defects; the
>   [rerun](../../../qa/security-levels/sprints/PF-27-S06/README.md#gate-rerun-after-the-fixes-2026-10-08-pass) at
>   `661b5c6a48cd` **passed** in a normal session ([videos](../../../qa/demos/index/PF-27-S06.md)).
> - [x] Gate evidence received by the P1 integration owner (2026-10-08; PR #312).

Verbatim, `docs/sprints/archive/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md` lines 30-43:

> ## Closure — 2026-10-08
>
> Completed. Travis **accepted** PF-27-S07 **with known limits** on 2026-10-08 (in chat with the coordinator): the
> limits are the [Known limits](../../../qa/security-levels/sprints/PF-27-S07/README.md#known-limits) list in the
> evidence README. Received and archived by the P1 integration owner; gate run evidence merged in PR #316.
>
> Follow-ups (linked, not blockers):
>
> - #300 (deny-read not enforced on the unelevated tool path): decided, fail closed; being implemented.
> - #301 and #304 (deny entries revoked by a flag-off session / never revoked): decided, rule-driven removal; draft PR #326.
> - #320 (`spawn_protected` stdout pipe briefly inheritable, the "broker stdout inheritance" limit): fixed by PR #327.
> - #323 (an armed contract's deny entries vs. another `CODEX_HOME`'s sync): open.
> - [PF-27-S08](../../sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) and
>   [PF-27-S09](../../sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md): planned.

Verbatim, `docs/sprints/archive/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md` lines 105-112:

> ## Verification
>
> - [x] `pf_27_s07` on `windows-2022` (jobs 113293258039, 113280283309, 113292935422); PF-27-S06 suites still pass;
>   Linux clippy `-D warnings` clean on the RTX box; macOS suites pass.
> - [x] Opus 5.5 High review per slice: approve (slice 1 after 3 rounds, 2a and 2b after 2).
> - [x] GLM 5.2 tmux run and SOP videos on real Windows at `265172beed3d`, normal session: **passed** (Core, new
>   threads and broker unopenable; deny removed on flag-off, kept while armed; #294/#295 hold) ([gate run](../../../qa/security-levels/sprints/PF-27-S07/README.md#real-windows-gate-run-2026-10-08-pass), [videos](../../../qa/demos/index/PF-27-S07.md)).
> - [x] Gate evidence received by the P1 integration owner (2026-10-08; PR #316).

Follow-up states on 2026-10-09 (from `gh issue view`): #300, #301, #304, #307 and #320 are closed by the post-archive fixes (SECACCT-TN-02); #323 is open.

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#323](https://github.com/CorbanuCore/CorbanuTerminal/issues/323) | OPEN | — | Windows: an armed contract's deny-read entries are not protected from another CODEX_HOME's sync |

## Demo videos

`curl -s -o /dev/null -I -L -w %{http_code}` on every asset URL in [qa/demos/index/PF-27-S06.md](../../../qa/demos/index/PF-27-S06.md) and [qa/demos/index/PF-27-S07.md](../../../qa/demos/index/PF-27-S07.md): 14 and 12 assets, all 200.

PF-27-S06 (the first five at `63ea3d0cbd0c` are the failed run, including the two defect videos; the last two at `661b5c6a48cd` are the rerun):

| Demo | Commit | Length | Asset | HTTP |
| --- | --- | --- | --- | --- |
| `pf27s06-win-baseline-flag-off` | `63ea3d0cbd0c` | 40s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-baseline-flag-off-63ea3d0cbd0c-2026-10-08.mp4 | 200 |
| `pf27s06-win-baseline-flag-off` | `63ea3d0cbd0c` | 40s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-baseline-flag-off-63ea3d0cbd0c-2026-10-08.cast | 200 |
| `pf27s06-win-broker-pipe-foreign-client` | `63ea3d0cbd0c` | 37s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-broker-pipe-foreign-client-63ea3d0cbd0c-2026-10-08.mp4 | 200 |
| `pf27s06-win-broker-pipe-foreign-client` | `63ea3d0cbd0c` | 37s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-broker-pipe-foreign-client-63ea3d0cbd0c-2026-10-08.cast | 200 |
| `pf27s06-win-normal-session-launch-fails` | `63ea3d0cbd0c` | 24s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-normal-session-launch-fails-63ea3d0cbd0c-2026-10-08.mp4 | 200 |
| `pf27s06-win-normal-session-launch-fails` | `63ea3d0cbd0c` | 24s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-normal-session-launch-fails-63ea3d0cbd0c-2026-10-08.cast | 200 |
| `pf27s06-win-protected-files` | `63ea3d0cbd0c` | 40s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-protected-files-63ea3d0cbd0c-2026-10-08.mp4 | 200 |
| `pf27s06-win-protected-files` | `63ea3d0cbd0c` | 40s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-protected-files-63ea3d0cbd0c-2026-10-08.cast | 200 |
| `pf27s06-win-unelevated-refused` | `63ea3d0cbd0c` | 28s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-unelevated-refused-63ea3d0cbd0c-2026-10-08.mp4 | 200 |
| `pf27s06-win-unelevated-refused` | `63ea3d0cbd0c` | 28s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-unelevated-refused-63ea3d0cbd0c-2026-10-08.cast | 200 |
| `pf27s06-win-normal-session-works` | `661b5c6a48cd` | 25s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-normal-session-works-661b5c6a48cd-2026-10-08.mp4 | 200 |
| `pf27s06-win-normal-session-works` | `661b5c6a48cd` | 25s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-normal-session-works-661b5c6a48cd-2026-10-08.cast | 200 |
| `pf27s06-win-vault-denied` | `661b5c6a48cd` | 40s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-vault-denied-661b5c6a48cd-2026-10-08.mp4 | 200 |
| `pf27s06-win-vault-denied` | `661b5c6a48cd` | 40s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s06-pf27s06-win-vault-denied-661b5c6a48cd-2026-10-08.cast | 200 |

PF-27-S07 (gate run at `265172beed3d`, including the #294/#295 regressions):

| Demo | Commit | Length | Asset | HTTP |
| --- | --- | --- | --- | --- |
| `pf27s07-win-core-threads-unopenable` | `265172beed3d` | 45s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-core-threads-unopenable-265172beed3d-2026-10-08.mp4 | 200 |
| `pf27s07-win-core-threads-unopenable` | `265172beed3d` | 45s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-core-threads-unopenable-265172beed3d-2026-10-08.cast | 200 |
| `pf27s07-win-broker-unopenable` | `265172beed3d` | 33s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-broker-unopenable-265172beed3d-2026-10-08.mp4 | 200 |
| `pf27s07-win-broker-unopenable` | `265172beed3d` | 33s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-broker-unopenable-265172beed3d-2026-10-08.cast | 200 |
| `pf27s07-win-codex-home-deny-flag-off` | `265172beed3d` | 18s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-codex-home-deny-flag-off-265172beed3d-2026-10-08.mp4 | 200 |
| `pf27s07-win-codex-home-deny-flag-off` | `265172beed3d` | 18s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-codex-home-deny-flag-off-265172beed3d-2026-10-08.cast | 200 |
| `pf27s07-win-codex-home-deny-kept-while-armed` | `265172beed3d` | 35s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-codex-home-deny-kept-while-armed-265172beed3d-2026-10-08.mp4 | 200 |
| `pf27s07-win-codex-home-deny-kept-while-armed` | `265172beed3d` | 35s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s07-win-codex-home-deny-kept-while-armed-265172beed3d-2026-10-08.cast | 200 |
| `pf27s06-win-normal-session-works` | `265172beed3d` | 23s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s06-win-normal-session-works-265172beed3d-2026-10-08.mp4 | 200 |
| `pf27s06-win-normal-session-works` | `265172beed3d` | 23s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s06-win-normal-session-works-265172beed3d-2026-10-08.cast | 200 |
| `pf27s06-win-vault-denied` | `265172beed3d` | 41s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s06-win-vault-denied-265172beed3d-2026-10-08.mp4 | 200 |
| `pf27s06-win-vault-denied` | `265172beed3d` | 41s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s07-pf27s06-win-vault-denied-265172beed3d-2026-10-08.cast | 200 |

## Open, not done

- #323 (open): an armed contract's deny-read entries are not protected from another `CODEX_HOME`'s sync.
- PF-27-S08 (broker restricted token, in progress) and PF-27-S09 (Windows model auth, planned).
- CI does not cover a separate non-administrator account or the elevated setup from a normal session (stated in both READMEs above).
- Milestone code-blind VM run, human sign-off and flag removal.
