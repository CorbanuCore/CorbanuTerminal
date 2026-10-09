# PF-27-S08 evidence: the Windows broker's own token

Third of the four PF-27-S06 limits Travis approved fixing (2026-10-08). Behind the existing default-off flags
(`isolated_credential_broker`, `secretless_agent_launch`); Permissive, macOS and Linux are unchanged.

## What shipped (PR #333)

- **The broker token.** Core starts the broker under a restricted copy of its own token, so no privilege is needed:
  - write-restricted to a fresh random capability SID, the logon SID and Everyone. A write then also needs an entry
    for one of them; the broker's pipes name the capability SID (low-labeled objects that grant Everyone or the
    logon SID are a known limit below);
  - low integrity, so everything of the user's at medium or above refuses its writes, and other processes and
    threads refuse it everything beyond query-limited information and terminate (measured);
  - every privilege but `SeChangeNotifyPrivilege` removed; Administrators and INTERACTIVE deny-only (INTERACTIVE
    is what COM's default launch permission grants);
  - default DACL: the PF-27-S07 protected thread DACL, as before.
- **Reported and required.** The broker checks its own token and reports `token+dacl+job`. Core refuses a Windows
  broker without `token`. Broker pipes grant the capability SID, so the broker can add instances; only Core and
  the broker can use them.
- **Probes** (`pf_27_s08`, process-hardening and network-proxy) on `windows-2022` (elevated and normal session),
  each with a positive control: the same child started the PF-27-S07 way (same token as the parent).

## Why a restricted token, not an AppContainer

Measured, the restricted token meets acceptance 2–5 except the limits below. An AppContainer would refuse loopback
(brokered upstreams on `localhost`, `allow_local_binding`) without an administrator exemption, needs a registered
profile (persistent per-user state), and cannot read the vault or the user's profile without new ACL grants.

Variants tried and dropped: write restriction to the capability SID alone, and untrusted integrity: the broker does
not start (`0xC0000142`, the window station and desktop refuse it).

## Measured

| Criterion | Probe | Broker token | Control (PF-27-S07 broker) |
| --- | --- | --- | --- |
| 1 | token check, containment string | low, write-restricted, 1 capability, no privileges; `token+dacl+job` | `dacl+job`, refused by Core |
| 2 | create, overwrite, append, attributes, rename, delete, mkdir in `%TEMP%` | all denied | all succeed |
| 2 | same in `%USERPROFILE%\AppData\LocalLow` (low integrity) | all denied **except delete** | all succeed |
| 3 | an ordinary process of the user: `VM_READ`, `VM_WRITE`, `VM_OPERATION`, `DUP_HANDLE`, `CREATE_THREAD`, `SUSPEND_RESUME`, `SET_INFORMATION`, `WRITE_DAC`, `WRITE_OWNER`, `PROCESS_CREATE_PROCESS`, `SET_QUOTA`, `QUERY_INFORMATION`; its threads: get/set context, suspend, terminate, impersonate | all denied except **query-limited information and process terminate**; environment unreadable | all granted |
| 4 | `Win32_Process.Create` (WMI) | `80041003` access denied | starts a process |
| 4 | `Schedule.Service` register a task | refused (`80070003`, the root folder is not shown) | registered |
| 4 | `MMC20.Application` activation (out-of-process COM, starts `mmc.exe`) | `800A0046` permission denied | starts `mmc.exe` (elevated; in a normal session on CI too, not on the real machine) |
| 5 | broker suite over pipes (PF-27-S04/PF-28-S02/PF-33-S02/PF-27-S06, 25 tests) | pass, elevated and normal session | — |
| 5 | DNS and TCP to `github.com:443` | work | work |
| 5 | Credential Manager read of a synthetic generic credential | **readable** | readable |
| 5 | Credential Manager: create a new synthetic generic credential (its cleanup delete is not checked) | **created** | created |
| 5 | Credential Manager: delete a credential that does not exist | `1168` not found (not access denied) | same |

Real Windows 11 machine, 2026-10-09, at commit `dbf8be6df5` (`C:\CorbanuQA\s08`): process-hardening `pf_27_s08`
7/7 elevated and in a normal (medium-integrity) session, network-proxy `credential_broker::isolated::` 25/25 both.

GLM 5.2 tmux SOP demo videos (leak-scanned, uploaded to the `demos` release, indexed in `qa/demos/index/PF-27-S08.md`):
`pf27s08-win-broker-token` (the TUI logs `containment=token+dacl+job` and an agent command runs with the broker up)
and `pf27s08-win-broker-token-probes` (every pf_27_s08 probe passes in a normal session).

## Launch mechanism and parent-process spoofing

Kept the PF-27-S07/#320 start: the broker is created from a suspended holder process, which now carries the broker
token, so Windows reports the (exited) holder as the broker's parent. Security software that compares the creating
process with the reported parent may flag this as parent-process spoofing. A direct start
(`CreateProcessAsUserW` from Core, no inherited handles, Core as parent) was built and measured: the broker could
not open its own token (its token object gets Core's default DACL, which in an elevated Core grants Administrators,
deny-only in the broker token) and so could not bind its pipes; Core could not change that token object's DACL after
creation either. Fixing it needs a different token-object descriptor at creation and was left out of this sprint.

## Decision: how the vault key reaches the broker (Travis, 2026-10-09)

**(c): the broker reads Credential Manager itself**, under its own token, the same as the macOS and Linux brokers
read the OS keyring (PF-27-S05); PF-27-S09 ports S05. Acceptance 5 now reads that way; the original
"the broker's token reading Credential Manager is denied" was dropped. S08 decides and measures the path (the token
can read Credential Manager); the Windows broker reads no stored keys yet (`server.rs`). The reader is PF-27-S09's,
with one Windows addition to S05: every vault read opens `secrets/.vault.lock` for writing, which the broker token
cannot do on a medium-labeled file, so S09 must grant it (capability-SID entry and low label, the Windows analogue
of S05's Landlock exception) or hand the broker an opened lock. The alternatives, a helper or Core handing the
key over, would also have needed the broker kept out of Credential Manager, which this token type cannot do (it
would need an AppContainer or new ACL grants). The accepted consequence is the first known limit below.

## Merge and current main

#333 was merged with five long-running checks still pending (stalled runners). They passed afterwards on main: at the
merge commit `d0544c1c91`, `windows-security-probes` and `corbanu-terminal-ci` are green. (Its `postmerge-ci` failed
on an unrelated `app-server-protocol` schema fixture; postmerge is green again from `d9e851b380`.)

Rechecked 2026-10-09 on main at `2f3e04201f` (the latest `windows-security-probes` run; nothing under
`process-hardening/` or `network-proxy/` changed after it up to `c0e242c1a9`): run 37967366555, job 113944960637,
`pf_27_s08` 7/7 elevated and 7/7 in a normal session, the broker suite over pipes 25/25 both, with the same
measurements as the table above.

## Known limits

- **Credential Manager (accepted with decision (c)).** A compromised broker can read the user's other generic
  credentials in Credential Manager, not only the vault key, and create new ones (measured on CI and the real
  machine, elevated and normal session). Overwriting or deleting an existing credential (including the vault key)
  was not probed; Credential Manager has no per-credential access control, so assume it can. Mitigated by its
  other confinement: it cannot write the user's files, open their processes beyond query-limited and terminate,
  start child processes (job), or start work through WMI, Task Scheduler or COM. Its network is not restricted, so a compromised broker could still send what it reads anywhere.
- **Terminate.** Low integrity leaves `PROCESS_TERMINATE` (and query-limited) on the user's ordinary processes, so
  the broker can kill them (the terminal, an editor). Core is not affected: its protected DACL grants only
  query-limited and synchronize.
- **Not probed (review, non-blocking).** Low-labeled objects that grant Everyone or the logon SID write (likely the
  session's named objects, new pipe names, mailslots) could be created or squatted by the broker; `WRITE_DAC` and
  `WRITE_OWNER` on low-labeled files, and `HKCU\Software\AppDataLow`; Core checks the broker's own report of its
  token rather than opening the token itself.
- **Delete in low-integrity folders.** Write restriction does not cover `FILE_DELETE_CHILD`, so in a folder at low
  integrity that grants the user full control (`LocalLow`, `Temp\Low`, and likely `%LOCALAPPDATA%\Packages\*`
  and `INetCache\Low`) the broker can delete a file or tree bottom-up, though not create, change or rename one (the
  same gap as the sandbox's write-restricted token, #158). Medium-labeled files inside a low folder can also be
  deleted through `FILE_DELETE_CHILD`.
- **COM servers callable by low-integrity clients.** The main barrier to out-of-process COM is the broker's low
  integrity level: COM checks the caller's integrity against the server's launch and access permissions, so a
  low-integrity client is refused unless those permissions carry a low label. A server configured to allow
  low-integrity callers and to run as the interactive user, a service or a named account starts under that identity
  (not the broker's token), and the broker can then call its methods — the classic way out of a low-integrity sandbox,
  and outside both the token and the job. Already-running servers (checked by access permissions, not launch
  permissions) and the general RPC/ALPC surface carry the same risk. Deny-only INTERACTIVE is a secondary close for
  servers that keep COM's default launch permission (which grants INTERACTIVE); a server whose AppID grants Everyone,
  Users or Authenticated Users launch rights would bypass it.
- **Task Scheduler** refuses by hiding its folders from a low-integrity caller, not with access denied. In CI's
  normal session the control could not register a task either (`Permission denied`), so there that row has no
  positive control; the elevated run has one.
- **Normal-session COM control.** On the real machine `MMC20.Application` needs elevation in a normal session, so
  there the COM row's positive control is the elevated run; CI's normal-session control did start it.
- **Parent-process spoofing signal** (above).
