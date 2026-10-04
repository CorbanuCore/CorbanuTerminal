# PF83 native infrastructure correction — bXFovm

2026-09-14. Routine, private infrastructure preparation only. This worker is
neither product implementer nor acceptance executor/reviewer. Root AGENTS,
code-blind-functional README/isolated-execution contract, preparation receipt
`preparation-receipt-vwQlK8.md`, and the Corbanu development skill were read.
The skill routed this work to the isolation gate; no plan/sprint changes apply
to this bounded, non-product preparation. PF83 functional acceptance remains open.

**Outcome: blocked for acceptance use.** The final deny-by-default policy proves
the listed synthetic denials, including children, and bounded PTY input works.
Held-v1 `corbanu --version` times out under that same strict policy. Earlier
policies that launch it successfully fail the process-inspection negative.
Neither result may be combined into a passing environment. No denies were
relaxed after the strict failure to manufacture a successful positive control.

## Private artifacts and exact binding

New private directory, mode 0700:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-native-deny.bXFovm`.
Every execution created another fresh `attempt-*` child. Previous attempts and
the old preparation/native-01 roots were never rerun or edited.

- [One-off preflight script](/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-native-deny.bXFovm/preflight.py): 264 authored lines, including synthetic probes, generated-policy literal and local bridge. SHA256 `7e7152f887a0b167d992a2b854212f6e74a6fc070a96ad58aac5b1aa09bb7c25`.
- [Final exact effective Seatbelt policy](/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-native-deny.bXFovm/attempt-38p0ulnc/sandbox.sb): SHA256 `febfe5e40033a2399d7f4818fa3894b310ae5aaf083ab1ee0f6eaa3022376893`. No imports or allow-default rule. All literal paths and runtime exceptions are recorded verbatim there and in the observations JSON.
- [Final raw observations](/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-native-deny.bXFovm/attempt-38p0ulnc/observations.json): SHA256 `e26fcc61f99172bfcfcb146ac1510c8249d64c383a8d72044734a178f57ec9aa`. Includes exact argv, environment, process IDs, policy text/hash, package hashes, raw stdout/stderr, timeouts, bridge requests/results, cleanup and per-probe checks.

Final run started `2026-09-14T07:17:10Z`, macOS 26.6.2 / 25G83, arm64 host.
The final generated policy has 26 lines; script plus rendered policy is below
the 400-line allocation. No general runner, installed component or service was
created. Patterns reused: old preflight's owned synthetic canaries, environment
reset, Seatbelt wrapper and PTY I/O; the existing `functional-execution.ikteKU` pattern
of coordinator-owned validation of model actions. No old script was imported.

Each attempt APFS-copied only the held v1 package into its own directory for
infrastructure controls. Original package:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-preparation.vwQlK8/package`.
Copied `bin/corbanu` SHA256 remains
`57b19153926f778f69dc12243703796c07d100e97f47253244b23d321659d9bd`.
Final before/after package file-content hashes match. Held v1 failed Fable review;
it is not accepted, and the other worker owns review fixes and final rebuild.
This worker did not launch a product session or run any acceptance case.

## Effective authority

The linked policy is the authoritative exact text, invoked as
`/usr/bin/sandbox-exec -f <this-attempt>/sandbox.sb <fixed-preflight-argv>`.
Its base is `(deny default)`, with these explicit allowances only:

- Read/map installed system framework and dylib subtrees, exact Cryptex dyld locations, Python 3.14.4's versioned Cellar tree, and this attempt's package. No whole `/usr`, `/opt/homebrew`, `/private/var/db` or user-home read grant.
- Execute the exact Python launcher and underlying Python app executable, `/bin/bash`, `/bin/cat`, `/bin/ls`, `/usr/bin/printf`, and the copied `bin/corbanu`. Other packaged helpers are readable but not executable in this one-task preflight.
- Read this attempt's packet; read/write only its private state subtree. Read/write `/dev/null` and the exact coordinator-allocated PTY `/dev/ttys013`; ioctl only that PTY. Read `/dev/random` and `/dev/urandom`. No PTY wildcard, TMUX socket or host-terminal session access.
- Read the literal filesystem root and two Cryptex roots for runtime startup; read ancestor metadata only for the explicitly selected runtime/package paths. This exposes some root-directory names/path metadata, not a claim of zero host metadata visibility.
- Read exact named hardware/kernel runtime sysctls, including page-size compatibility and dyld/ignition inputs; fork and signal self. The policy file enumerates every sysctl. No environment is inherited beyond the literal constructed environment in the JSON; HOME, CODEX_HOME, CORBANU_HOME, PFTERMINAL_HOME, TMPDIR and cache point to this run's private state.

Explicit denials retain networking in its entirety, Mach lookup/registration,
POSIX/System V IPC, all `process-info*`, cross-process signals, and AppleEvents.
The final policy also denies `SYS_proc_info`, `SYS_proc_info_extended_id` and
`SYS_ptrace`. The extended proc-info syscall is present in the installed SDK.
The earlier process-info rule plus old proc-info syscall denial was insufficient
for the observed `proc_pidpath` call. The final change tightened both the
process-info scope and syscall set, so this experiment does not isolate which
individual added restriction causes the held-v1 startup timeout.

There is **no network exception**: no synthetic inference transport was required
for offline preflight. The unapproved TCP listener was `127.0.0.1:54857`, the
owned Unix socket `/private/tmp/pf83-deny-cqbk7rty/s`, and the ephemeral Mach
canary `local.pf83.canary.attempt-38p0ulnc.60776`. These were created solely for
this experiment, not real host sessions or persistent/provisioned services.

## Final observed controls

These are actual syscall/API outcomes. JSON probe processes generally exit 0
after reporting the operation's error: exit status alone is not the verdict.
Every file canary existed and was positively read/listed by the coordinator.
Endpoint, process and other-PTY controls succeeded outside confinement both
before and after their confined probes. Both TCP and Unix listeners accepted
exactly those two positive-control connections, and no confined connection.

| Control | Direct and child observations under final policy |
| --- | --- |
| Synthetic source, history, auth, prior-results read/list | All return `EPERM` / errno 1 against existing files/directories. No real credentials or sessions were probed. |
| Symlink from writable task state to source canary | Both return `EPERM`. |
| Copied package executable modification | Both append-open attempts return `EPERM`; package file-content hashes unchanged. |
| Live unapproved loopback TCP and cross-run Unix socket | Both direct/child connection attempts return `EPERM`; peers stayed live. |
| Live cross-run Mach lookup | Both return 1100 (`BOOTSTRAP_NOT_PRIVILEGED`); coordinator lookup returns 0 before and after. Setup allocate/insert/register each returned 0. |
| Inspect owned live process PID 60929 with `proc_pidpath` | Both return 0 bytes / errno 1; coordinator returns 124 bytes before and after. |
| Signal owned live process with handled SIGUSR1 | Both return `EPERM`; positive sends succeed; process remains alive and emits its synthetic signal marker. |
| Open another owned live PTY | Both return `EPERM`; coordinator opens it successfully before and after. |
| Host automation executable | Direct `/usr/bin/osascript` launch denied with exit 71; child bash launch denied with exit 126 and `Operation not permitted`. Only `return 1` was requested; no real app/session was targeted. |
| Python/runtime, packet and private state | Exit 0: `RUNTIME_OK 16384`, `PF83_SYNTHETIC_PACKET_ONLY`, `PRIVATE_OK`. |
| Bounded actual PTY text then Enter | Separate bridge calls produce `PTY_RECEIVED:PF83_SYNTHETIC_KEY`; a subsequent child `cat` visibly returns `Operation not permitted`. |
| Held-v1 `--version`, captured pipes | **Failed positive:** no output within 8 seconds; owned process group killed/reaped, exit -9. |
| Held-v1 `--version` after PTY key control | **Failed positive:** no version output before observation/action bound; owned process group killed/reaped, exit -9. |

The final JSON has 58 command records: 57 match their locally checked control
expectation; `held-v1-version` does not. The separate PTY record also explicitly
records its version timeout. These counts are not functional acceptance results.
Child enforcement was exercised by fresh Python subprocess execs, plus bash/cat
under the same inherited policy. Final probe outputs include PID/PPID identity.

## Preserved unsuccessful attempts

All names below are children of the new private root and retain their own exact
`sandbox.sb` and raw `observations.json`. None was overwritten by a replay.

| Attempt, in execution order | Actual result/disposition |
| --- | --- |
| `qbd__ldf`, `ignzsuzz` | Runtime/dyld SIGABRT (-6), including packet and package controls. Denial probes not established. Owned startup crash reports identify dyld CacheFinder/ignition; no acceptance evidence. |
| `8cixo_rh` | Same confined startup aborts; additionally a misplaced startup-check edit made the synthetic probe text invalid. Unconfined syntax failures are preserved and not positives. |
| `1wqy7icz` | Corrected probe text; early startup control still -6, later probes not reached. |
| `2l39p7mg`, `4zxosrt_` | Runtime advanced to Python launcher, but its underlying app executable was not yet allowlisted; spawn failure, later probes not reached. |
| `hw8d_tlp` | Basic runtime/packet/state work; probe ctypes import fails at uname, shell selector path is denied, package guard-page initialization aborts. These are setup failures, not targeted denials. |
| `y2lu_uh7` | Named runtime sysctls and explicit bash produce package version and real PTY output; direct/child `proc_pidpath` still succeeds despite intended denial. Disqualified. |
| `wi27e3g3` | Adding only the old proc-info syscall denial still permits direct/child `proc_pidpath`; package/PTY positives work. Disqualified. |
| `jbilwiao` | Full process-info and extended syscall restrictions deny inspected process paths, but package version times out in pipe and PTY controls. Not qualified. |
| `38p0ulnc` | Final strict replay above, with explicit own-PTY ioctl, bounded process-group cleanup, before/after endpoint positives, PID binding and exact outcome checks. Still blocked by version timeout. |

## Tool boundary, remaining gaps and stop condition

The private script contains a coordinator-owned `Bridge` with only `text`,
`key`, and `observe`. Exact fields are validated; text is printable ASCII,
1–512 characters with no embedded Enter/control sequences; allowed keys are
ENTER/CTRL_C/ESC. Limit: 32 valid actions, 4096 bytes per observation, 65536 bytes
total observation data, with bounded reads. Output is returned as terminal text,
not interpreted as host commands or terminal automation.

Eight synthetic requests for exec/read/fetch, extra observation fields, embedded
newline, oversized input, unknown key and an OSC sequence were all rejected.
The demonstrated interface cannot choose argv, host paths, endpoints or another
PTY. The coordinator's arbitrary-command probe function is preflight-only and
must never become an executor tool. No acceptance agent was launched: this is
local interface validation, not a demonstrated external model tool inventory.
Future executor actions must be decoded solely into that bounded key/observation
interface, with no host tools. This script/receipt and its findings are
coordinator-only, not part of any future code-blind input packet.

Mediated inference is still absent for both the model-only executor and any
target inference needed by the frozen cases. No credential broker or exact
synthetic inference endpoint was wired or tested; no live inference or
credential access occurred. A future permitted mediator must hold credentials
outside target/executor readability and expose no arbitrary fetch/proxy/source
or host-tool capability. No external agent/session was represented as isolated.

Direct AppleEvent API sends, every Mach-right/IPC variant, debugger/memory access,
all kernel process-discovery routes, resource exhaustion, native Keychain/browser/
launcher flows and actual product TUI behavior are **not** established by these
controls. Default/explicit denies are policy declarations for unexercised routes,
not an all-encompassing sandbox proof. The entire runtime and helpers needed by
future acceptance are also not yet qualified by a held-v1 version control.

Stop here: repairing product source or provisioning an account/VM/privileged
service exceeds this allocation, and weakening the process boundary for a
positive control is forbidden. The immediate infrastructure blocker is the
strict-policy version timeout; final candidate rebuild and mediated executor
wiring remain independent gates. No product source, plan, staging, commit,
merge, install, launcher or live profile was written. This is the single
infrastructure-authored QA receipt in the worktree.

Owned TCP/Unix listeners and PTYs were closed; each short private socket directory
was removed, ephemeral Mach receive rights destroyed (final subsequent lookup
1102, service unknown), and canary processes exited. Timeout process groups were
killed/reaped in the final run. A final exact-prefix process check found no
remaining owned package `--version` process. Only disposable socket paths were
removed; all attempt evidence and package copies remain recoverable in the
private root. Host-generated startup crash reports were not deleted. Script and
embedded probe syntax were checked without executing them again.
