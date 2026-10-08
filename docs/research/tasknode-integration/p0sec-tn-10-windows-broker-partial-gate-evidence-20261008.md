# Task Node evidence record: P0SEC-TN-10

Compile the P0SEC-TN-10 Windows Broker Partial-Gate Evidence Record. Task `task_1330fc0d84a8ee77e131023399bcb2aa`, request `req_615bf0ee9fe71e06864e807dc008f6ce6d1a31eeaae71ec402d8d97064f96435`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `ad96c55cb5` (2026-10-08). Quoted blocks are copied verbatim from the records named above them (relative links re-pointed to this file's location).

**The per-sprint gate is partial and is not claimed complete.** Done: `pf_27_s06` tests on GitHub `windows-2022` runners, Linux clippy and the Linux/macOS suites, and one Opus 5.5 High review per slice ending APPROVE. **Not done:** the GLM 5.2 tmux functional run and the SOP demo videos (both need a real Windows machine; none was available), Travis's acceptance of the documented limits, and archiving the record. **No demo videos exist yet for PF-27-S06.** Also not claimed: the milestone code-blind VM run, human sign-off and flag removal. PF-27-S06 belongs to the P1 security hardening plan; its sprint record stays `status: draft` until that plan is activated.

Sprint record: [docs/sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md](../../sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md). Evidence: [qa/security-levels/sprints/PF-27-S06/README.md](../../../qa/security-levels/sprints/PF-27-S06/README.md). Flags: `isolated_credential_broker`, `secretless_agent_launch` (default off).

| PR | Merge commit on main | What |
| --- | --- | --- |
| #267 | `36e342b9fb` | Slice 1: Windows process access hardening and measured probes |
| #269 | `a71eb2ad71` | Slice 2a: Windows named-pipe broker transport |
| #270 | `6cc330436b` | Slice 2b: isolated credential broker on Windows |
| #272 | `cce641e322` | Slice 3: secretless launch contract on Windows (elevated sandbox) |
| #277 | `ad96c55cb5` | Sprint record and evidence |

PR links:

- #267: https://github.com/CorbanuCore/CorbanuTerminal/pull/267
- #269: https://github.com/CorbanuCore/CorbanuTerminal/pull/269
- #270: https://github.com/CorbanuCore/CorbanuTerminal/pull/270
- #272: https://github.com/CorbanuCore/CorbanuTerminal/pull/272
- #277: https://github.com/CorbanuCore/CorbanuTerminal/pull/277

## CI job check

`gh api repos/CorbanuCore/CorbanuTerminal/actions/jobs/<id>` for each job id the README names (all `pf_27_s06 probes (windows-2022)`, conclusion `success`):

| Job | Head | Completed |
| --- | --- | --- |
| 113190329753 | `2a6b26ba49` (PR #272 merged head) | 2026-10-08T07:18:34Z |
| 113152390863 | `b1ed3bc603` | 2026-10-08T04:47:34Z |
| 113167492899 | `33c33ffc38` | 2026-10-08T05:46:08Z |
| 113167757386 | `a88b0df223` | 2026-10-08T05:47:28Z |
| 113179399269 | `00a85072a4` | 2026-10-08T06:40:35Z |

`windows-security-probes` runs on each PR's merged head (`gh run list --commit <head>`), all `success`: #267 `feb0c07b64` run 37731473462; #269 `d1dd4532e9` run 37735587727; #270 `a9589f72b4` run 37735591935; #272 `2a6b26ba49` run 37740627114.

## Evidence README

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 1-9:

> # PF-27-S06 evidence: Windows broker and secretless launch
>
> Flags: `isolated_credential_broker` and `secretless_agent_launch` (both default off). Permissive is unchanged, and
> nothing changes on macOS or Linux beyond a refactor of the broker's transport code (Linux and macOS suites rerun).
>
> No Windows host was available. Every Windows result below is measured on GitHub's `windows-2022` runners by the
> `windows-security-probes` workflow, which runs on every PR that touches this code. The runners are elevated
> administrators with `SeDebugPrivilege` enabled, so probes that stand for an ordinary process disable their own
> privileges first (an enabled debug privilege opens any process whatever its DACL; that is a documented limit).

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 11-18:

> ## What shipped
>
> | Slice | PR | What |
> | --- | --- | --- |
> | 1 | #267 | `restrict_current_process_access()`: protected process DACL (user: query-limited + synchronize; SYSTEM; OWNER RIGHTS: read-control) and the same for every thread, existing ones at once and new ones from a TLS callback. Hardening fails closed if the callback is not in the image's TLS directory. |
> | 2a | #269 | Named-pipe transport: random first-instance names, protected DACL for the user, remote clients refused, no inheritable handles, peer process id checked before any byte, identification-level impersonation, an overlapped control pipe for the controller. |
> | 2b | #270 | The isolated broker on Windows over those pipes; the broker contains itself with the DACL plus a job (no child processes, no desktop/clipboard/atoms/other USER handles); it holds its controller's process handle and exits with it. |
> | 3 | #272 | The secretless launch contract passes on Windows under the elevated sandbox (a separate sandbox user) and refuses the unelevated restricted-token sandbox with a stated reason. A protected launch first gives `CODEX_HOME` an inherit-only, files-only read deny for the sandbox's users, so files created or replaced there later stay denied. |

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 20-43:

> ## Measured on windows-2022
>
> | Probe | Result |
> | --- | --- |
> | Command under the unelevated sandbox's restricted token vs an unhardened process (positive control) | memory and environment canary read |
> | Same token, and a same-user process without privileges, vs a hardened process | `PROCESS_VM_READ`, `QUERY_INFORMATION`, `DUP_HANDLE`, `WRITE_DAC`, `WRITE_OWNER`, `VM_WRITE`, `VM_OPERATION`, `CREATE_THREAD`, `SUSPEND_RESUME`, `SET_INFORMATION` all denied; every thread (one started after hardening) denied `GET_CONTEXT`, `SET_CONTEXT`, `SUSPEND_RESUME`; environment unreadable |
> | Same-user process vs an unhardened process (positive control) | every right granted |
> | Broker pipe: same-user foreign client | connects (the DACL grants the user) and is dropped before any byte |
> | Broker pipe: restricted-token client | read+write open denied; read-only open connects and gets nothing |
> | Pipe handles | none inheritable; a child that scans its own handle table finds no broker pipe (an inheritable duplicate is found: positive control) |
> | Wrong server pid, look-alike name | refused by the controller |
> | Contained broker | containment `dacl+job`; it cannot start a process; another same-user process cannot open it to read memory |
> | Broker suite over named pipes | the PF-27-S04/PF-28-S02/PF-33-S02 suite, 24 tests, passes (substitution only inside the broker, wrong peer, forged/replayed frames, revocation, crash, bounds, scrubbing, pinning) |
> | Controller killed outright | the broker exits within 10 s |
> | Contract decisions | unelevated restricted token refused; elevated accepted (also through `env_for`) |
> | Elevated sandbox, base profile (positive control) | vault, `auth.json`, `config.toml`, `*.sqlite` and an unprotected file all readable |
> | Elevated sandbox, contract's profile | vault, `auth.json`, `config.toml`, `*.sqlite` denied; unprotected file readable; `CODEX_HOME` not writable |
> | File created in a protected directory during a run | denied (inherited deny) |
> | `auth.json` replaced by rename, and a new `state_9.sqlite-wal`, during a run | denied (the files-only deny on `CODEX_HOME`); before that deny existed the replaced file was measured readable |
> | Unprotected file in `CODEX_HOME\skills` under the contract | readable (control) |
> | Elevated sandbox probe vs hardened and unhardened Core stand-ins | both denied: the separate sandbox user is already the boundary |
>
> CI jobs (`windows-security-probes`): merged slice 3 head 113190329753 (all 38 tests); slice 1 113152390863; slice 2a 113167492899; slice 2b 113167757386 (broker
> suite); slice 3 113179399269 (new-file deny). The merged heads were rerun green before each merge.

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 45-52:

> ## Found and fixed on the way (all measured)
>
> - The CI runner's `SeDebugPrivilege` opened the hardened target in the first run; probes now drop privileges.
> - A synchronous pipe handle serializes all I/O on its file object: the controller's first write waited behind its
>   own reader thread and the broker never received Hello. The controller's control pipe is now overlapped.
> - The elevated runner re-quotes argv, so quoted paths in `cmd.exe` probes became invalid; the base-profile control
>   caught it.
> - The handle scan hung on inherited CI pipes with pending synchronous I/O; each query now has a deadline.

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 54-73:

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

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 75-80:

> ## Reviews (Opus 5.5 High)
>
> Slice 1: changes requested (threads), changes requested (TLS linkage, window), approve. Slice 2a: changes requested
> (read-only restricted connect, busy retry, timeouts), approve. Slice 2b: approve with findings, approve (overlapped
> pipe and parent handle; its low finding was fixed in 2a). Slice 3: changes requested (replaced files, apply_patch,
> vacuous control), changes requested (replaced files still open), approve. Texts: [reviews/](../../../qa/security-levels/sprints/PF-27-S06/reviews/).

Verdict line of each review text (`rg -m1 VERDICT`): slice1-review1 CHANGES REQUESTED, slice1-review2 CHANGES REQUESTED, slice1-review3 APPROVE; slice2a-review1 CHANGES REQUESTED, slice2a-review2 APPROVE; slice2b-review1 APPROVE, slice2b-review2 APPROVE; slice3-review1 CHANGES REQUESTED, slice3-review2 CHANGES REQUESTED, slice3-review3 APPROVE. Texts: [reviews/](../../../qa/security-levels/sprints/PF-27-S06/reviews/).

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 82-85:

> ## Linux and macOS
>
> Linux clippy (`-D warnings`) on the RTX box is clean for every slice; Linux `credential_broker` 55, process-hardening
> 7, core `launch_contract` 12 and `pf_27_s0` 23 tests pass. macOS: the same suites pass.

Verbatim, `qa/security-levels/sprints/PF-27-S06/README.md` lines 87-102:

> ## Windows machine needed for the remaining gate
>
> Still open: the decision 5 tmux run (GLM 5.2 driving the real TUI) and the SOP videos.
>
> - Windows 11 Pro/Enterprise 23H2 or later (or Windows Server 2022/2025 with Desktop Experience), x64, a real machine
>   or full VM (not Windows Sandbox or a container): the elevated sandbox creates local users and a group.
> - One local administrator account for the one-time elevated sandbox setup (UAC prompt), then a normal session of
>   that user; 8+ cores, 16 GB RAM, 60 GB free.
> - Remote access: OpenSSH Server for driving builds and tmux, plus RDP to look at the TUI.
> - Tools: Git for Windows, Visual Studio 2022 Build Tools (C++ workload, Windows 11 SDK), rustup with Rust 1.95.0
>   MSVC, Python 3.12+, MSYS2 with tmux (native console apps run through ConPTY), asciinema 3.x, agg, ffmpeg, gh.
>   The SOP recorder has only been used on macOS/Linux; a short feasibility check of tmux + asciinema under MSYS2
>   comes first, with Windows Terminal plus ffmpeg screen capture as the fallback.
> - Network: github.com, crates.io, and the Z.AI API for GLM 5.2.
> - Credentials: a Z.AI key reachable on that machine (vault label `zai` in an installed signed `corbanu`, or a scoped
>   key Travis provides) and a GitHub token only if videos are published from there.

## Sprint record

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md` lines 79-85:

> ## Remaining
>
> - [ ] Decision 5 tmux run (GLM 5.2 driving the TUI) and SOP videos on a real Windows machine ([requirements](../../../qa/security-levels/sprints/PF-27-S06/README.md#windows-machine-needed-for-the-remaining-gate)).
> - [ ] Travis's acceptance of the documented limits (evidence README), including the new-thread DACL window.
> - [ ] Follow-ups for the plan worker: a restricted or AppContainer token for the broker; Windows model auth
>   (PF-27-S05); file tools other than patches under the contract on Windows; the elevated sandbox's read of
>   `~/.git-credentials`, `.ssh`, `.npmrc`, `.config/gh` if profile reads are ever granted (setup excludes most).

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md` lines 87-92:

> ## Verification
>
> - [x] On `windows-2022` (job 113190329753, the merged slice 3 head): process-hardening 9, network-proxy broker and
>   pipe suite 24, core 5 `pf_27_s06` tests pass. Linux (RTX box): clippy `-D warnings` clean; broker 55,
>   process-hardening 7, core 12 + 23 tests pass. macOS: the same suites pass.
> - [ ] GLM 5.2 tmux run and SOP videos on a Windows host.

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md` lines 94-97:

> ## Exit evidence
>
> - [x] Outputs under `qa/security-levels/sprints/PF-27-S06/` ([evidence](../../../qa/security-levels/sprints/PF-27-S06/README.md)).
> - [ ] Record archived (after the Windows-host gate items).

## Open, not done

- GLM 5.2 tmux functional run (sec-common decision 5) on a real Windows machine.
- SOP demo videos: none recorded; `qa/demos/index/` has no PF-27-S06 entry.
- Travis's acceptance of the documented limits, including the new-thread DACL window.
- Record archiving (after the Windows-host items) and P1 plan activation (record stays `draft`).
- Milestone code-blind VM run, human sign-off, flag removal.
