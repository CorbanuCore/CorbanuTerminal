# PF-27-S08 evidence: the Windows broker's own token

Third of the four PF-27-S06 limits Travis approved fixing (2026-10-08). Behind the existing default-off flags
(`isolated_credential_broker`, `secretless_agent_launch`); Permissive, macOS and Linux are unchanged.

## What shipped (PR #333)

- **The broker token.** Core starts the broker under a restricted copy of its own token, so no privilege is needed:
  - write-restricted to a fresh random capability SID, the logon SID and Everyone. A write then also needs an entry
    for one of them; the broker's pipes name the capability SID, nothing of the user's does;
  - low integrity, so everything of the user's at medium or above refuses its writes, and other processes and
    threads refuse it everything beyond query-limited access;
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
| 4 | `MMC20.Application` activation (out-of-process COM, starts `mmc.exe`) | `800A0046` permission denied | starts `mmc.exe` (elevated; a normal session needs elevation for it) |
| 5 | broker suite over pipes (PF-27-S04/PF-28-S02/PF-33-S02/PF-27-S06, 25 tests) | pass, elevated and normal session | — |
| 5 | DNS and TCP to `github.com:443` | work | work |
| 5 | Credential Manager read of a synthetic generic credential | **readable** | readable |
| 5 | Credential Manager write and delete of a synthetic generic credential | **writable, deletable** | writable, deletable |

Real Windows 11 machine, 2026-10-09, at commit `06a150a3ab` (`C:\CorbanuQA\s08`): process-hardening `pf_27_s08`
7/7 elevated and in a normal session (interactive logon), network-proxy `credential_broker::isolated::` 25/25 both.

## Launch mechanism and parent-process spoofing

Kept the PF-27-S07/#320 start: the broker is created from a suspended holder process, which now carries the broker
token, so Windows reports the (exited) holder as the broker's parent. Security software that compares the creating
process with the reported parent may flag this as parent-process spoofing. A direct start
(`CreateProcessAsUserW` from Core, no inherited handles, Core as parent) was built and measured: the broker could
not open its own token (its token object gets Core's default DACL, which in an elevated Core grants Administrators,
deny-only in the broker token) and so could not bind its pipes; Core could not change that token object's DACL after
creation either. Fixing it needs a different token-object descriptor at creation and was left out of this sprint.

## Open decision for Travis: how the vault key reaches the broker

The record assumed the broker's token could not read Credential Manager. Measured: it can read, write and delete
the user's generic credentials (CI, and the real machine elevated and in a normal session). The write and delete
probes were added from the review (item 4): if the broker can write to Credential Manager it can also overwrite or
delete the vault key. Options:

1. **(c) The broker reads Credential Manager itself** (recommended): the same as the macOS and Linux brokers, which
   read the OS keyring (PF-27-S05); PF-27-S09 ports S05 unchanged. Cost: a compromised broker can read the user's
   other generic credentials too.
2. **(a)/(b) A helper or Core hands the key over**: then the broker's token must also be kept out of Credential
   Manager, which this token type cannot do without making the user SID deny-only, after which the broker cannot read
   its own executable or the vault in the user's profile. That needs an AppContainer or new ACL grants: new work.

Acceptance 5's "the broker's token reading Credential Manager is denied" is not met until this is decided.

## Known limits

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
- **Task Scheduler** refuses by hiding its folders from a low-integrity caller, not with access denied.
- **Normal-session COM control.** `MMC20.Application` needs elevation there, so the COM row's positive control is the
  elevated run.
- **Parent-process spoofing signal** (above).
