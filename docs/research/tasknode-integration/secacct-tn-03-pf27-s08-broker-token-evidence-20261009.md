# Task Node evidence record: SECACCT-TN-03

Compile the SECACCT-TN-03 PF-27-S08 Broker Token Evidence Record. Task `task_ee52fe14f4a911a695cf1849a3d1d6e9`, request `req_4703b65bc4e8503889f1603a5e3c42eddc2036158c623dd81ffc0ece796fb8d3`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: PF-27-S08 is `in_progress`; it is not complete and not accepted.** PR #333 (slice 1) is merged behind the default-off flags. Acceptance criterion 5 is **not met**: the broker token can read, write and delete the user's Credential Manager generic credentials (measured), and how the vault key reaches the broker is an **open product decision for Travis** (options below; (c) recommended). The sprint record's verification line reads REQUEST_CHANGES (2 blocking) then APPROVE after fixes.

Records: [docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md](../../sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (status `in_progress`), [qa/security-levels/sprints/PF-27-S08/README.md](../../../qa/security-levels/sprints/PF-27-S08/README.md), [qa/demos/index/PF-27-S08.md](../../../qa/demos/index/PF-27-S08.md).

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#333](https://github.com/CorbanuCore/CorbanuTerminal/pull/333) | `d0544c1c91` | 2026-10-09T10:27:16Z | PF-27-S08 slice 1: Windows broker under its own restricted token |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#333 MERGED base=main merge=d0544c1c91 checks: SKIPPED=10 SUCCESS=28; git merge-base --is-ancestor d0544c1c91 3254a302fd -> exit 0
```

## Evidence README

Verbatim, `qa/security-levels/sprints/PF-27-S08/README.md` lines 6-20:

> ## What shipped (PR #333)
>
> - **The broker token.** Core starts the broker under a restricted copy of its own token, so no privilege is needed:
>   - write-restricted to a fresh random capability SID, the logon SID and Everyone. A write then also needs an entry
>     for one of them; the broker's pipes name the capability SID, nothing of the user's does;
>   - low integrity, so everything of the user's at medium or above refuses its writes, and other processes and
>     threads refuse it everything beyond query-limited access;
>   - every privilege but `SeChangeNotifyPrivilege` removed; Administrators and INTERACTIVE deny-only (INTERACTIVE
>     is what COM's default launch permission grants);
>   - default DACL: the PF-27-S07 protected thread DACL, as before.
> - **Reported and required.** The broker checks its own token and reports `token+dacl+job`. Core refuses a Windows
>   broker without `token`. Broker pipes grant the capability SID, so the broker can add instances; only Core and
>   the broker can use them.
> - **Probes** (`pf_27_s08`, process-hardening and network-proxy) on `windows-2022` (elevated and normal session),
>   each with a positive control: the same child started the PF-27-S07 way (same token as the parent).

Verbatim, `qa/security-levels/sprints/PF-27-S08/README.md` lines 22-29:

> ## Why a restricted token, not an AppContainer
>
> Measured, the restricted token meets acceptance 2–5 except the limits below. An AppContainer would refuse loopback
> (brokered upstreams on `localhost`, `allow_local_binding`) without an administrator exemption, needs a registered
> profile (persistent per-user state), and cannot read the vault or the user's profile without new ACL grants.
>
> Variants tried and dropped: write restriction to the capability SID alone, and untrusted integrity: the broker does
> not start (`0xC0000142`, the window station and desktop refuse it).

Verbatim, `qa/security-levels/sprints/PF-27-S08/README.md` lines 31-52:

> ## Measured
>
> | Criterion | Probe | Broker token | Control (PF-27-S07 broker) |
> | --- | --- | --- | --- |
> | 1 | token check, containment string | low, write-restricted, 1 capability, no privileges; `token+dacl+job` | `dacl+job`, refused by Core |
> | 2 | create, overwrite, append, attributes, rename, delete, mkdir in `%TEMP%` | all denied | all succeed |
> | 2 | same in `%USERPROFILE%\AppData\LocalLow` (low integrity) | all denied **except delete** | all succeed |
> | 3 | an ordinary process of the user: `VM_READ`, `VM_WRITE`, `VM_OPERATION`, `DUP_HANDLE`, `CREATE_THREAD`, `SUSPEND_RESUME`, `SET_INFORMATION`, `WRITE_DAC`, `WRITE_OWNER`, `PROCESS_CREATE_PROCESS`, `SET_QUOTA`, `QUERY_INFORMATION`; its threads: get/set context, suspend, terminate, impersonate | all denied except **query-limited information and process terminate**; environment unreadable | all granted |
> | 4 | `Win32_Process.Create` (WMI) | `80041003` access denied | starts a process |
> | 4 | `Schedule.Service` register a task | refused (`80070003`, the root folder is not shown) | registered |
> | 4 | `MMC20.Application` activation (out-of-process COM, starts `mmc.exe`) | `800A0046` permission denied | starts `mmc.exe` (elevated; a normal session needs elevation for it) |
> | 5 | broker suite over pipes (PF-27-S04/PF-28-S02/PF-33-S02/PF-27-S06, 25 tests) | pass, elevated and normal session | — |
> | 5 | DNS and TCP to `github.com:443` | work | work |
> | 5 | Credential Manager read of a synthetic generic credential | **readable** | readable |
> | 5 | Credential Manager write and delete of a synthetic generic credential | **writable, deletable** | writable, deletable |
>
> Real Windows 11 machine, 2026-10-09, at commit `dbf8be6df5` (`C:\CorbanuQA\s08`): process-hardening `pf_27_s08`
> 7/7 elevated and in a normal (medium-integrity) session, network-proxy `credential_broker::isolated::` 25/25 both.
>
> GLM 5.2 tmux SOP demo videos (leak-scanned, uploaded to the `demos` release, indexed in `qa/demos/index/PF-27-S08.md`):
> `pf27s08-win-broker-token` (the TUI logs `containment=token+dacl+job` and an agent command runs with the broker up)
> and `pf27s08-win-broker-token-probes` (every pf_27_s08 probe passes in a normal session).

Verbatim, `qa/security-levels/sprints/PF-27-S08/README.md` lines 54-62:

> ## Launch mechanism and parent-process spoofing
>
> Kept the PF-27-S07/#320 start: the broker is created from a suspended holder process, which now carries the broker
> token, so Windows reports the (exited) holder as the broker's parent. Security software that compares the creating
> process with the reported parent may flag this as parent-process spoofing. A direct start
> (`CreateProcessAsUserW` from Core, no inherited handles, Core as parent) was built and measured: the broker could
> not open its own token (its token object gets Core's default DACL, which in an elevated Core grants Administrators,
> deny-only in the broker token) and so could not bind its pipes; Core could not change that token object's DACL after
> creation either. Fixing it needs a different token-object descriptor at creation and was left out of this sprint.

Verbatim, `qa/security-levels/sprints/PF-27-S08/README.md` lines 64-78:

> ## Open decision for Travis: how the vault key reaches the broker
>
> The record assumed the broker's token could not read Credential Manager. Measured: it can read, write and delete
> the user's generic credentials (CI, and the real machine elevated and in a normal session). The write and delete
> probes were added from the review (item 4): if the broker can write to Credential Manager it can also overwrite or
> delete the vault key. Options:
>
> 1. **(c) The broker reads Credential Manager itself** (recommended): the same as the macOS and Linux brokers, which
>    read the OS keyring (PF-27-S05); PF-27-S09 ports S05 unchanged. Cost: a compromised broker can read the user's
>    other generic credentials too.
> 2. **(a)/(b) A helper or Core hands the key over**: then the broker's token must also be kept out of Credential
>    Manager, which this token type cannot do without making the user SID deny-only, after which the broker cannot read
>    its own executable or the vault in the user's profile. That needs an AppContainer or new ACL grants: new work.
>
> Acceptance 5's "the broker's token reading Credential Manager is denied" is not met until this is decided.

Verbatim, `qa/security-levels/sprints/PF-27-S08/README.md` lines 80-99:

> ## Known limits
>
> - **Delete in low-integrity folders.** Write restriction does not cover `FILE_DELETE_CHILD`, so in a folder at low
>   integrity that grants the user full control (`LocalLow`, `Temp\Low`, and likely `%LOCALAPPDATA%\Packages\*`
>   and `INetCache\Low`) the broker can delete a file or tree bottom-up, though not create, change or rename one (the
>   same gap as the sandbox's write-restricted token, #158). Medium-labeled files inside a low folder can also be
>   deleted through `FILE_DELETE_CHILD`.
> - **COM servers callable by low-integrity clients.** The main barrier to out-of-process COM is the broker's low
>   integrity level: COM checks the caller's integrity against the server's launch and access permissions, so a
>   low-integrity client is refused unless those permissions carry a low label. A server configured to allow
>   low-integrity callers and to run as the interactive user, a service or a named account starts under that identity
>   (not the broker's token), and the broker can then call its methods — the classic way out of a low-integrity sandbox,
>   and outside both the token and the job. Already-running servers (checked by access permissions, not launch
>   permissions) and the general RPC/ALPC surface carry the same risk. Deny-only INTERACTIVE is a secondary close for
>   servers that keep COM's default launch permission (which grants INTERACTIVE); a server whose AppID grants Everyone,
>   Users or Authenticated Users launch rights would bypass it.
> - **Task Scheduler** refuses by hiding its folders from a low-integrity caller, not with access denied.
> - **Normal-session COM control.** `MMC20.Application` needs elevation there, so the COM row's positive control is the
>   elevated run.
> - **Parent-process spoofing signal** (above).

## Sprint record

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md` lines 50-54:

> ## Preconditions
>
> - [x] PF-27-S07 merged; a real Windows machine (used 2026-10-09).
> - [ ] Product decision: how the vault key reaches the broker: (a) one-shot helper, (b) Core sends it,
>   (c) broker reads Credential Manager itself. Measured: (c) works; (a)/(b) need a stronger token.

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md` lines 75-79:

> ## Decisions
>
> - Token: write-restricted, low-integrity restricted token, not an AppContainer; launch: the PF-27-S07 holder start.
>   Reasons, measurements and limits: the evidence README.
> - Key path: **open, for Travis.** The broker token can read Credential Manager (measured), so (c) is what ships.

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md` lines 81-85:

> ## Done
>
> - [x] Planned (2026-10-08).
> - [x] Criteria 1–4 and 5 except the key path, with positive controls, on `windows-2022` and the real machine (PR #333).
> - [x] Real-Windows GLM 5.2 tmux run and SOP videos (2026-10-09).

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md` lines 87-89:

> ## Remaining
>
> - [ ] 5 (part): key-path decision (Credential Manager is readable by the broker token).

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md` lines 91-95:

> ## Verification
>
> - [x] Probes and suite on `windows-2022` and the real Windows machine (elevated and normal session), Credential
>   Manager measured on both: done 2026-10-09.
> - [ ] Independent review verdict: REQUEST_CHANGES (2 blocking: B1, B2), then APPROVE after fixes.

Verbatim, `docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md` lines 97-100:

> ## Exit evidence
>
> - [x] Outputs under `qa/security-levels/sprints/PF-27-S08/` (probes, gate run and videos).
> - [ ] Key-path decision for Travis: how the vault key reaches the broker (Credential Manager is readable by the broker token).

## Demo videos

2 assets, all HTTP 200:

| Demo | Commit | Length | Asset | HTTP |
| --- | --- | --- | --- | --- |
| `pf27s08-win-broker-token` | `dbf8be6df5` | 31s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s08-pf27s08-win-broker-token-dbf8be6df581-2026-10-09.mp4 | 200 |
| `pf27s08-win-broker-token-probes` | `dbf8be6df5` | 29s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s08-pf27s08-win-broker-token-probes-dbf8be6df581-2026-10-09.mp4 | 200 |

## Open, not done

- Key-path decision (Travis): (c) broker reads Credential Manager itself, or (a)/(b) a helper or Core hands the key over (needs an AppContainer or new ACL grants).
- Acceptance criterion 5 (part): "the broker's token reading Credential Manager is denied" is not met.
- Sprint completion, archive and Travis's acceptance; PF-27-S09; milestone gates.
