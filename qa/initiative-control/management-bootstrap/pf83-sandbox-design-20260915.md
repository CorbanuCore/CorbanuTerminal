# PF83 per-run macOS enforcement proposal

Action: `pf83-sandbox-design-01`  
Allocation digest: `5077f604cdb33bbc218cdfdeb7c994d3493b8800f2334880b7fefc01865d0bfd`  
Claim: `861414aa-5b2a-4815-979a-7be2f19a2f09`  
Worker: `gpt-6-astra`, high effort; source-informed infrastructure designer, not a code-blind functional designer or executor.  
Date: 2026-09-15. Status: DESIGN ONLY; not implemented, compiled, tested, approved or qualified.

## Decision and ceiling

Use `/usr/bin/sandbox-exec` with a fixed, parameterized Seatbelt allowlist for the **bridge and its exec child**. Keep a trusted fixture/witness supervisor outside that sandbox. Apply the sandbox before Python starts. Keep packet, runtime and package immutable; allow writes only to this role's private run directories. Deny direct IP networking and native service access; permit only specifically registered synthetic fixture sockets.

This is not a 175/175 proposal. The current 175 entries comprise **162 denials (27 controls x 6 roles) plus 13 positives**, not 175 denial controls. Two independent blockers precede any score improvement:

1. `macos_preflight.py:38` currently calls the same operation three times from the same PID under the same policy. A successful sandbox makes before, middle and after all fail; `native_guest.py:221` requires before and after to succeed. A profile-only edit therefore cannot produce valid denial rows. Introduce independent, live, same-target witnesses; do not relax the validator or manufacture successful witness return codes.
2. Target and target-child enforcement cannot be inferred from bridge enforcement. An outer Seatbelt sandbox can both mask the behavior being tested and interfere with the product's own sandbox installation. Target execution requires the manager decision in section 6.

The first proposed increment targets **at most 28 denial rows: 14 controls x bridge/bridge_child**, plus preservation of the two currently populated positive rows. That is an upper bound of **30/175**, not a forecast or a pass claim. All 108 denial rows for actor, actor_child, target and target_child remain UNPROVEN in that increment, as do 26 bridge-role denial rows and the other 11 positive entries. Missing witnesses, profile incompatibility, an inaccessible operator-home fixture or failed positive controls reduce this ceiling. Native dispatch stays disabled.

## 1. Inputs, scope and sources

The brief was read after `shasum -a 256` returned its required digest:
`d2a268be1d62f8241c292d1a010f05889ed49778a0738c0442f69a8e43583a4d`.
The management checkout HEAD was verified as `5583fe4909ef3a0ac278146ff5c16e154d79313d`; its working tree was clean when inspected.

Path abbreviations used below are exact:

- **R** = `/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`.
- **H** = `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915`.
- **SDK** = `/Library/Developer/CommandLineTools/SDKs/MacOSX26.2.sdk` on the design host. This is not a claim about the guest SDK or OS version.

This private design is routine QA planning, with no product or policy modification. Follow-on changes to authorization, credentials, target attribution or acceptance requirements need the applicable product-initiative authority and updated execution mandate. Product linkage: **Permission selection confirmation — TO BUILD**, R/docs/corbanu-product-spec.md:78, excerpt: “A submitted selection is not a confirmed change.” R/docs/plans/active/p0-security-levels.md:124 owns PF-83's evidence contract. Current sprint R/docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md:1 is `in_progress`, with its implementation worktree distinct from this management checkout. No new implementation authority is inferred here.

Inspected source anchors:

| File | Verified location and significance |
| --- | --- |
| H/native_guest.py | :191 validator; :209 binding; :217 denial checks; :225 positives; :240 dispatch refusal |
| H/macos_preflight.py | :29 environment; :38 same-process triple measurement; :63 synthetic files; :88 same-sandbox peer creation; :103 owned PTY; :114 sockets; :144 loopback; :158 external networking; :172 missing native services; :256 service; :272 declared lack of enforcement; :283 absent roles; :292 positives |
| H/macos_adapter.py | :31 SSH service launch; :73 stage; :83 archive members. The current command starts Python directly, without Seatbelt. |
| H/cases.json | :16 exact six roles, 27 denials and 13 positives |
| H/boundary-policy.txt | :13 OS enforcement prerequisite; :25 live paired controls; :26 same effective policy positives |
| H/README.md | :85 preparatory native protocol, still disabled |
| H/NATIVE-MACOS.md | :41 measurements; :50 absent roles; :57 PTY limitations; :63 same-uid/package limitations |
| R/qa/initiative-control/management-bootstrap/pf83-preflight-finding-20260915.md | :69 proposed next unit; :76 target attribution caution |
| R/codex-rs/sandboxing/src/seatbelt_base_policy.sbpl | :8 default deny; :11 inheritance-related process rules; :105 PTY extension pattern |
| R/codex-rs/sandboxing/src/seatbelt.rs | :235 AF_UNIX rule generation; :248 scoped socket parameters |
| R/codex-rs/sandboxing/src/seatbelt_network_policy.sbpl | :18 SecurityServer/trust-related grants; do not copy these into the bridge profile |
| /usr/share/man/man1/sandbox-exec.1 | :9 deprecation; :26 profile before command; :46 parameter interface |
| /usr/share/man/man7/sandbox.7 | :45 inheritance; :48 already-open descriptor caveat |
| SDK/usr/include/sandbox.h | :23 already-sandboxed process error; :45 API deprecation |
| SDK/usr/include/bootstrap.h | :37 BOOTSTRAP_NOT_PRIVILEGED=1100; :92 bootstrap_look_up returns kern_return_t |
| SDK/usr/include/netdb.h | :195 EAI_NONAME=8; :198 EAI_SYSTEM is a different error domain |

Read-time SHA-256 pins (private harness files are not protected by the management Git commit):

```
native_guest.py     d4adcf5d488facd1a4a91a0282734fec60e18619b1fe7336ce54e40ff6c8bc32
macos_preflight.py  fc9bb621a17175bc08511cda35d30fb50a2c376710f88a69f4aac7dd7a4a1b76
macos_adapter.py    6f2fdd07661e71f3b7e6f5e767ecf7c60cdf667b2d6eb09a320ed928630e41d7
cases.json         69e0b237facaaaf209e1c3bd1f523f7fa644942dc3b5543346a62667cc2b3d6f
boundary-policy.txt 04777b1718ef448e4654a9fcb1dc447861921ca4f740ee22e364a3edfa90c36c
README.md          f08d3935f7abdeea2579b5f66d94cab96e42cee819d13a7bf50f5cade209250d
NATIVE-MACOS.md     8f9cf1b168950806ac51c75f3eec391afe3ff0728f6381f986ba8032ba00552f
```

Apple's [sandbox diagnostic guidance](https://developer.apple.com/documentation/security/discovering-and-diagnosing-app-sandbox-violations?changes=_4) shows that a denied resolver transport can produce an underlying connect EPERM. This does not establish that Python getaddrinfo returns EPERM: API-level errors must be recorded in their actual domains. All expected errnos below are design hypotheses pending the exact guest build's real probes.

## 2. Mechanism and literal bridge profile

Choose `sandbox-exec`, not injected `sandbox_init` inside the long-running service: the interpreter must start inside the restriction, and fixture creation must be outside it. The existing product already uses the fixed system executable. This adds no product binary patch or code-signing change. Both interfaces are deprecated; their presence and syntax on this exact guest require a later qualification. App Sandbox entitlement changes would alter the candidate. A network extension or privileged MAC implementation is not provisioned and would be a much larger mechanism, not a small substitute.

The following is the complete **proposed bridge profile template**, not an asserted compiled profile. Comments name clauses used in section 4. No common profile import or runtime policy concatenation is permitted. `param` values are supplied by a trusted launcher as separate argv entries, not interpolated into SBPL or shell source. The launcher freezes their canonical paths and validates disjointness first.

```scheme
(version 1)

; D0: every operation not explicitly allowed below is denied.
(deny default)

; P1: a bridge may fork and exec only fixed tools and the diagnostic package.
(allow process-fork)
(allow process-exec
  (literal (param "PYTHON"))
  (literal (param "CANDIDATE"))
  (literal "/bin/cat")
  (literal "/bin/echo"))
; P2: only this sandbox's descendants are manageable/inspectable.
(allow signal (target same-sandbox))
(allow process-info* (target same-sandbox))

; R1: read-only run material; these directories are supervisor-owned.
(allow file-read*
  (subpath (param "PACKAGE"))
  (subpath (param "RUNTIME"))
  (subpath (param "PACKET"))
  (literal (param "MANIFEST")))

; R2: pinned interpreter and system runtime, never broad /Users or /Library.
(allow file-read*
  (literal (param "PYTHON"))
  (subpath (param "PYTHON_RUNTIME"))
  (subpath "/System/Library")
  (subpath "/usr/lib")
  (subpath "/usr/share/locale")
  (subpath "/usr/share/zoneinfo")
  (subpath "/usr/share/terminfo")
  (literal "/bin/cat")
  (literal "/bin/echo"))
(allow file-map-executable
  (literal (param "PYTHON"))
  (subpath (param "PYTHON_RUNTIME"))
  (subpath (param "PACKAGE"))
  (subpath "/System/Library")
  (subpath "/usr/lib")
  (literal "/bin/cat")
  (literal "/bin/echo"))

; R3: ancestor metadata for path traversal/getcwd, not directory contents.
(allow file-read-metadata
  (path-ancestors (param "RUN"))
  (literal (param "RUN"))
  (literal (param "STATE"))
  (literal "/dev")
  (literal "/private")
  (literal "/private/etc"))

; W1: only per-role mutable state, not STATE as a whole or sibling controls.
(allow file-read* file-write*
  (subpath (param "PROFILE"))
  (subpath (param "TMP"))
  (subpath (param "WORKSPACE")))

; I1: only exact, trusted, already-listening fixture endpoints.
(allow system-socket (socket-domain AF_UNIX))
(allow file-read* (literal (param "FIXTURE_SOCKET")))
(allow file-read* (literal (param "INFERENCE_SOCKET")))
(allow network-outbound
  (remote unix-socket (literal (param "FIXTURE_SOCKET")))
  (remote unix-socket (literal (param "INFERENCE_SOCKET"))))

; T1: newly allocated PTYs require the per-sandbox PTY extension.
(allow pseudo-tty)
(allow file-read* file-write* file-ioctl (literal "/dev/ptmx"))
(allow file-read* file-write* file-ioctl
  (require-all
    (regex #"^/dev/ttys[0-9]+$")
    (extension "com.apple.sandbox.pty")))
(allow file-read* file-write-data file-ioctl (literal "/dev/tty"))

; B1: inert runtime needs; no resolver, trustd or credential-service grant.
(allow file-read* (literal "/dev/random") (literal "/dev/urandom"))
(allow file-read* file-write-data (literal "/dev/null") (literal "/dev/zero"))
(allow sysctl-read
  (sysctl-name "hw.ncpu")
  (sysctl-name "hw.memsize")
  (sysctl-name "hw.pagesize")
  (sysctl-name "hw.machine")
  (sysctl-name "hw.logicalcpu")
  (sysctl-name "hw.physicalcpu")
  (sysctl-name "kern.ostype")
  (sysctl-name "kern.osrelease")
  (sysctl-name "kern.osversion"))
```

There are deliberately no IP network, Mach lookup/register, AppleEvent, task-port, POSIX shared-memory, or cross-sandbox signal grants. D0 supplies those denials. No network rule grants all loopback or all port 443. T1 follows the existing product's PTY extension pattern, but omits its broad tty ioctl grant. A peer PTY must be created outside this sandbox, or it is not a negative control. `/dev/tty` refers only to the process's controlling terminal; the launcher must not give it an unrelated terminal.

Parameters and topology:

- `RUN` is the manager-generated `/Users/agent/pf83-preflight-<32 hex>` root, resolved without symlinks; `STATE` is its state directory. Canonical path aliases/firmlinks must be inventoried on the guest. Path strings alone are insufficient.
- `PACKAGE`, `RUNTIME`, `PACKET`, `MANIFEST` are the fixed staged children of RUN. `CANDIDATE` is PACKAGE/corbanu. Only the bridge sees harness runtime source; this profile is **not** an actor-readable package specification.
- `PYTHON` is the resolved, inventoried native Python executable; `PYTHON_RUNTIME` is its independently identified immutable framework/stdlib root. The existing `/usr/bin/python3` may be a developer-tools launcher. The actual target must be resolved before pinning; do not solve launch failures by granting `/Library` or all of Homebrew.
- `PROFILE`, `TMP`, `WORKSPACE` are the bridge role's dedicated children of STATE. No canary, sibling profile, package, supervisor-control or credential-store path may lie under these grants. No shell init files or live aliases are imported. The child inherits this bridge sandbox and role state; independent actors require separate role state and a separately frozen profile.
- `FIXTURE_SOCKET` and `INFERENCE_SOCKET` are exact Unix socket paths owned by the trusted supervisor outside writable state. Their parent cannot be renamed or replaced by the bridge. Existing protected socket fixtures and all other run endpoints are disjoint. Missing mediation leaves the inference endpoint absent and its positive UNPROVEN; do not redirect to an arbitrary service.
- Controlled denied fixtures sit under RUN/controls, **outside every allowed subtree**. Do not retain them under STATE/profile or TMP. A synthetic operator-home target must be clearly distinguished from the existing `/Users/neo_1` probe and requires frozen control semantics before substitution.
- Parent directory ownership matters: making a package file read-only while leaving its directory or an ancestor bridge-writable permits replacement. Provisioner-owned nonwritable ancestry and package/runtime/packet ownership or a genuine read-only volume must prevent that. Existing uid-owned 0555 files are insufficient against an unsandboxed same-uid supervisor/peer. This needs provisioning authority; none is exercised by this design.

The supervisor launches the bridge with an argv equivalent to `/usr/bin/sandbox-exec -f <immutable-profile> -D RUN=<run> ... <resolved-python> -I -B <runtime>/macos_preflight.py probe-service <run>`. Every omitted `-D` corresponds exactly to one listed parameter. This is descriptive argv, not a shell command for execution. It is a future launch mode; today's service does not accept `probe-service`.

Before exec: discard inherited directory/file/socket descriptors other than fixed stdio and a bounded supervisor protocol pipe; remove dynamic-loader/Python injection variables and socket discovery variables; set role profile aliases, cwd and resource limits; ensure no inherited Mach send rights to protected services. The supervisor owns cleanup because the restricted bridge cannot inspect arbitrary peers. The supervisor protocol offers only pre-registered witness IDs/sequence steps and fixed case actions, never arbitrary argv, paths, file reads, shell or URL fetching.

No Seatbelt profile revokes an already-open descriptor. No profile makes a generic allowed Python interpreter into a method-level tool allowlist. Both facts are part of the ceiling, not hidden exceptions.

## 3. Correct live-witness measurement

Replace the current triple call with three authenticated, ordered observations:

1. A trusted **external witness**, outside the tested sandbox but in the same VM and with the same applicable DAC identity, performs the exact safe operation on the same synthetic target. It records raw result, API/error domain, PID/start identity and target identity. File permission and service liveness must permit it. Readiness is not just “process running.”
2. The actual bridge PID attempts the operation under its fixed profile; the child does the same from its own exec'd PID. Capture the original libc/Python error. No reclassification of timeout, resolver code, exit status, empty output or KERN/OSStatus into EPERM.
3. The witness repeats its successful operation on that same target. Retain its result and timestamps. A fixture that disappears or a denied witness invalidates the row.

Use barriers over the pre-opened bounded supervisor channel; all timestamps are from the same guest monotonic clock. Preserve target device/inode, listener identity and address, or process PID plus start time across the interval. For file controls use synthetic nonsecret bytes and lengths/digests. For `package_write`, the frozen staged executable is mode 0555 (H/macos_adapter.py:96). The same-uid witness cannot open it O_WRONLY successfully, even without Seatbelt. A read-only mount also makes both witness opens fail. The existing paired-success criterion therefore cannot qualify that row. Leave it UNPROVEN. Do not chmod the exact package, change its manifest, silently substitute a writable canary or use a privileged witness to imply identical DAC conditions. A separately pinned writable diagnostic copy could demonstrate the policy mechanism in supporting evidence, but it would not populate this final-package control without an explicit acceptance-contract decision.

Package immutability remains mandatory and can have separate ownership, mount, manifest and nonmutation evidence. That does not satisfy an incompatible row by itself. This constraint reduces the first increment to 28 plausible denial rows plus the two preserved positives, at most 30/175.

The current report validator checks structure, not witness provenance. Keep it unchanged in the first increment, attach the richer raw records and require independent review; do not describe structural validity as complete attestation. `syscall` must name the actual API (e.g. open/getdirentries/connect/kill), not the control label as it does today. Handle missing targets before launching a probe rather than converting ENOENT or ECONNREFUSED into denial.

## 4. All 27 denial controls

**E** = eligible design hypothesis for bridge and bridge_child after witness repair and fixture provisioning; still unexecuted. Expected POSIX failure is `-1/EPERM` (Darwin 1); an actual EACCES (13) also meets the existing validator and must be recorded as observed. No prediction is evidence. **U** = remains UNPROVEN in the first increment. These entries apply only to the named attempted operation, never all possible bypasses.

| # | Control | Denying profile clause and exact operation/fixture | Expected result and ceiling |
| --- | --- | --- | --- |
| 1 | repository_read | D0: open/read RUN/controls/repository/canary; R1/R2/W1 do not cover it | E: open -1, EPERM; synthetic repository canary only, no real source opened |
| 2 | repository_list | D0: opendir/getdirentries of RUN/controls/repository; R3 gives no data/list grant | E: -1, EPERM; count-only successful witness |
| 3 | history_read | D0: open RUN/controls/history/canary | E: -1, EPERM; actual git/history absence is separate inventory evidence |
| 4 | prior_findings_read | D0: open RUN/controls/prior-findings/canary | E: -1, EPERM; actor packet must independently exclude findings |
| 5 | synthetic_auth_read | D0: open synthetic auth canary outside W1 | E: -1, EPERM; never use the provisioned subscription credential as a canary |
| 6 | operator_home_list | D0: list /Users/neo_1 (current control), not an R3 ancestor grant | E only if same-uid count-only witness succeeds; -1, EPERM expected. If absent/DAC-inaccessible, U until accepted synthetic-home fixture, not a renamed automatic pass |
| 7 | symlink_escape | D0 on resolved protected destination: link in TMP to RUN/controls/repository/canary | E: open -1, EPERM despite W1 permission on link location; validate canonical destination and no pre-opened FD |
| 8 | package_write | D0 file-write*: PACKAGE has R1/R2 read/map only, no W1 write grant | U: sandbox open should return EPERM, or existing 0555 DAC permissions may produce EACCES first, but same-uid successful write-open witness cannot pass against the frozen package mode. Read-only mounting gives EROFS with the same witness conflict. Do not change package modes or pins to fill this row. |
| 9 | peer_inspect | D0 process-info* outside P2; peer must be supervisor-spawned, not bridge-spawned | U: current /bin/ps adapter converts failure to EIO and can hide filtered/empty output. Direct proc/sysctl helper and guest-specific raw return contract are absent; cannot promise -1/EPERM |
| 10 | peer_memory | D0 task-port/debug access to outside peer | U: no helper; task_for_pid returns a kern_return_t, not POSIX -1/errno. Need synthetic non-debug-privileged peer and typed evidence; no debugger/TCC authorization |
| 11 | peer_signal | D0 signal outside P2: kill(peer_pid, 0), peer supervisor-owned outside sandbox | E: -1, EPERM; both witnesses must succeed. Current bridge-owned child is P2-allowed and proves no denial |
| 12 | other_pty | D0 file-read/write on supervisor-created tty lacking T1 extension | E: open(O_RDWR|O_NOCTTY) -1, EPERM. Do not create peer with bridge's openpty or grant all /dev/ttys* |
| 13 | unix_socket | D0 network-outbound and file access, outside exact I1 sockets; real listening synthetic UDS | E: connect -1, EPERM; separate witness connects before/after; forbidden socket is never pre-opened |
| 14 | pipe | D0 file-write on supervisor-created FIFO outside W1, with live reader | E: open(O_WRONLY|O_NONBLOCK) -1, EPERM. This is named-FIFO access, not revocation of inherited anonymous pipes |
| 15 | mach_service | D0 mach-lookup/register; no service grants | U: no live receiver/helper; bootstrap denial is a Mach/bootstrap result (e.g. BOOTSTRAP_NOT_PRIVILEGED 1100), not -1/EPERM. No truthful qualifying errno promised |
| 16 | apple_events | D0 appleevent-send and Mach acquisition | U: no synthetic receiver/helper; AE APIs return OSStatus, not errno; ensure no consent prompt. -1743-style authorization errors are not POSIX EPERM |
| 17 | agent_socket | D0/I1 exclusion, owned synthetic agent socket outside writable trees | E: connect -1, EPERM; proves synthetic socket access restriction, not interaction with a real agent/key store |
| 18 | docker_socket | D0/I1 exclusion, owned synthetic Docker socket | E: connect -1, EPERM; never probe the operator's Docker daemon |
| 19 | loopback_tcp | D0 network-outbound: direct AF_INET connect to live supervisor listener at numeric 127.0.0.1:<owned-port> | E: connect -1, EPERM. No loopback allow rule; UDS positives remain possible |
| 20 | lan_tcp | D0 network-outbound: numeric 192.168.64.1:22 | U despite expected immediate connect EPERM under Seatbelt: current guest firewall prevents the same-context external witness from connecting. Need separately authorized reachable controlled LAN fixture/route; no firewall change in first increment |
| 21 | dns | D0 denies resolver transports/Mach lookup | U: current getaddrinfo can report EAI_NONAME/EAI_FAIL or cache success, not a qualifying POSIX errno. Direct resolver-socket connect may yield EPERM but proves transport denial, not the unchanged DNS API control. Needs typed evidence and controlled resolver |
| 22 | internet_http | D0 network-outbound; current HTTPS github.com fails name resolution first | U: current wrapper exposes gaierror, and guest firewall prevents a live HTTP witness. A numeric connect probe proves IP transport, not HTTP HEAD. A reachable synthetic HTTP/TLS fixture plus layered raw errors needs separate scope |
| 23 | keychain_service | D0 mach-lookup plus D0 file-read outside granted runtime/state | U: no safe native-service fixture; Security APIs return OSStatus. Blocking one Mach name or reading a dummy auth file does not prove all Keychain paths. No real credential or native prompt operation |
| 24 | child_exec_escape | D0 process-exec outside P1, plus inherited D0 filesystem/network limits | U for the existing unspecified privileged-escape control: forbidden execve could return -1/EPERM, but allowed Python can interpret code and “escape” is not just executable path. Need frozen escape sequence/helper and descendants' boundary attempts; do not replace with a local file read |
| 25 | host_mount | D0 file-read outside R1/R2/W1 if a real host-mounted synthetic target exists | U: no actual host-shared live canary. A local synthetic file or missing mount is not a successful paired host-mount denial. Do not enable host sharing merely to fill this row |
| 26 | clipboard | D0 Mach acquisition and file access to pasteboard services/state | U: no isolated pasteboard fixture; pasteboard APIs may return no data or service/OSStatus errors rather than errno. No personal/shared pasteboard read |
| 27 | arbitrary_actor_tool | No complete clause: P1 restricts exec paths, but allowed Python can execute arbitrary code and syscalls within its sandbox | U: an actor tool allowlist is enforced by its dispatcher, not a POSIX sandbox clause; actor runtime does not exist here. A rejected tool request has protocol error semantics, not -1/errno |

The 13 U control names per bridge role are: `package_write`, `peer_inspect`, `peer_memory`, `mach_service`, `apple_events`, `lan_tcp`, `dns`, `internet_http`, `keychain_service`, `child_exec_escape`, `host_mount`, `clipboard`, `arbitrary_actor_tool`. With a missing operator-home witness, also retain `operator_home_list` as U; the ceiling falls to 28/175.

Do not promise Linux-style errno for Mach APIs. A future typed validator would preserve `api`, `result_domain`, raw result, and named denial status with API-specific successful witnesses; it would not coerce all failures to POSIX. Such a change needs the acceptance owner to approve the mapping and preserve the existing report as historical evidence. It is not included in the first increment.

## 5. Preserve all 13 positive requirements

The profile's intended positive surface and the missing prerequisites are distinct. The full set cannot currently be promised: only packet_read and package_version populate complete positive rows in H/macos_preflight.py. Retain all 13, with these exit requirements:

| Positive | How enforcement must permit it; required actual proof |
| --- | --- |
| packet_read | R1 permits only the frozen packet directory. Bridge reads/digests it under the final profile; later actor receives only the neutral packet through its separately confined runtime. Bridge receipt is not actor evidence. |
| package_version | P1/R1/R2 permit the pinned binary and its loader dependencies. Exact --version PID, stdout, exit, package digest under the bridge profile. A missing Mach/sysctl dependency is a failed qualification, not permission to broaden grants silently. |
| package_pty | T1 permits bridge-created PTYs; native candidate's functional PTY belongs to the separate target lane described below. Run the actual packaged TUI; /bin/cat is supporting infrastructure only. |
| text_then_enter | Bridge sends literal text and Enter separately; capture raw bytes and visible TUI transitions from that same target. No tmux binary/socket is granted in this first profile; explicitly use the existing Python PTY route, retaining that backend identity. |
| child_command | Bridge /bin/echo confirms P1 mechanics only. Positive requires a child actually requested through the candidate, with parent/start identity and an observed harmless workspace marker. Target lane must allow normal product tool paths. |
| fixture_ipc | I1 permits exact fixture/inference UDS clients. Trusted supervisor hosts servers outside the sandbox. Verify bounded request/response and listener identity, not just connect. Existing candidate MCP/fault transports must actually support the selected endpoint; no generic proxy or invented adapter. |
| actor_mediated_inference | Actor's separate confinement reaches a fixed mediator; mediator owns authentication and fixed provider destinations, refuses arbitrary URLs/CONNECT/tool requests and excludes secrets from responses. No mediator exists today, so this stays U. |
| target_mediated_inference | Separate product lane must use an existing supported provider/transport seam with the exact candidate. Current owner-authorized direct subscription file plus pinned HTTPS is functional direct inference, **not mediation**. Record the decision below; no credential copy/read in this design. |
| product_restricted | Under a fixed outer environment, the product's own selected restricted mode denies the harmless control and emits the expected confirmation/approval behavior. A bridge denial cannot populate this field. |
| product_full_access | Same candidate/command/control environment succeeds in the product's Full Access mode when authorized by the frozen case. This must be possible outside the restricted workspace to discriminate modes; the bridge profile would mask it. |
| transport_policy_query | Trusted collector identifies actual inference transport PID/start time, fixed endpoint inventory and its enforced policy/FDs. SSH or bridge policy fields cannot substitute. No supported “hash of effective Seatbelt profile” API is assumed. |
| profile_aliases | Allowlisted env maps HOME/CODEX_HOME/CORBANU_HOME/PFTERMINAL_HOME/XDG_CONFIG_HOME to the role profile and TMPDIR to its tmp. Confirm native process state and file effects across fresh/existing synthetic profiles, not just --version env. |
| private_state | W1 limits bridge writes to its role state. Start a second independently sandboxed run/role and prove it cannot read/write the first. 0700 with the same UID alone is insufficient. Supervisor retains initial/final synthetic state snapshots. |

Do not copy the product's existing general network profile: it grants SecurityServer/trust-service lookups, conflicting with the desired bridge native-service boundary. Do not grant all process inspection just so the bridge can run `pgrep`; move the prompt guard/cleanup to the supervisor. Never suppress a guard failure. Do not import the broad platform defaults that permit all /private/tmp, preferences, applications or tty devices.

## 6. The target problem — precise decision required

**Recommendation to Fable:** authorize bridge-only infrastructure work now; retain target and target_child as UNPROVEN. Seek/record the acceptance authority's explicit adoption of separate **containment evidence** and **product permission evidence** before functional target execution. This worker does not amend the frozen six-role matrix or grant exceptions.

A target started by the sandboxed bridge inherits that sandbox even if the argv does not mention sandbox-exec. A “Full Access” UI selection cannot remove inherited restrictions. The local Apple header also documents that sandbox_init returns an error when called in an already-sandboxed process; nested product setup is therefore an additional compatibility risk, not merely an intersection of policies. It must not be assumed to work.

Recommended final topology:

```
trusted VM supervisor / immutable launcher (source-informed; no actor tools)
  +-- synthetic live fixtures and independent witnesses
  +-- Seatbelt bridge -- exec bridge_child
  +-- separately confined code-blind actor -- actor_child
  +-- target lane launcher -- unchanged packaged Corbanu -- actual tool children
  +-- fixed inference mediator, if authorized/provisioned
```

The bridge sends only frozen target-launch IDs/PTY actions to the supervisor. It cannot ask for arbitrary supervisor execution. The target must not descend from the bridge. The target lane needs a disposable source-free, credential-isolated environment with no management filesystem/IPC exposure; VM separation, dedicated accounts and immutable external assets should supply that protection without an inherited Seatbelt policy that blocks product command semantics. This mechanism is not yet provisioned by the current guest arrangement. A second target VM is an option, not a claimed prerequisite already fulfilled or an authorized provisioning action.

For product attribution use a synthetic **discriminator** that the outer target environment deliberately permits in both runs, while the product restricted mode should deny it and Full Access should allow it. Example: a writable, nonsecret sibling directory outside the target's workspace, created only in the disposable target environment. Freeze the command, target, outer policy, account, cwd, package and fixture across the paired runs. Retain:

1. An independent outer-environment witness that the exact operation succeeds.
2. Real-key restricted selection, correlated confirmation and actual candidate-originated command attempt/approval result.
3. Real-key Full Access selection, correlated confirmation, same next-turn command and resulting marker.
4. Child identity, syscall/marker timing and product request correlation across both directions; pending-current-turn cases keep their original semantics.

If both runs deny, do not infer product enforcement. If the outer supervisor refuses the operation, classify harness enforcement. If a tool never launches because a product approval is correctly withheld, retain a product/protocol denial; it is not a target PID syscall with manufactured errno. Observations from a supervisor helper are never labeled actual target or target_child observations. An unchanged product does not offer arbitrary in-process probes, so obtaining every row from the target PID itself is a separate instrumentation/acceptance problem.

**Manager decision T1:** approve this split evidence model and its frozen target discriminator/control mapping, or require an alternative outer target mechanism demonstrated to allow those discriminators and the product's own sandbox installation. Keeping the current uniform 27-POSIX-denials-for-every-role gate means target execution remains blocked; no profile-only solution meets it honestly. Any change to the acceptance gate or out-of-scope case disposition requires product authority, not an implementation worker's interpretation.

**Manager decision T2:** reconcile the owner-authorized direct guest credential with `actor_mediated_inference`, `target_mediated_inference` and the code-blind no-real-credential boundary. Choose a separately provisioned trusted mediator with no executor-readable credential, or have the authorized owner explicitly freeze the alternative direct-inference qualification contract and its credential boundaries. A copy into a profile readable by target tool children cannot be presented as credential isolation. Existing direct inference authorization is not silently revoked; it also does not satisfy a differently named mediated positive. No credential movement is proposed for the first increment.

**Manager decision T3 (later):** approve API-specific denial evidence for Mach/OSStatus/DNS/tool controls, or retain those rows UNPROVEN. This is different from simply accepting every nonzero return code. Define exact safe fixtures, raw error domain and valid witness before changing a validator.

## 7. Pinning, effective enforcement and independent evidence

A hash field alone proves no sandbox was installed. Use a launch chain and raw controls:

- Freeze source profile bytes, ordered parameter map, fully expanded human-readable effective profile, package manifest, runtime/helper inventories, environment allowlist, guest OS/build/kernel, sandbox-exec binary identity, account/group/ownership/mount inventory, resource limits and inherited-FD/port inventory. Hash each separately. Hash a canonical versioned policy manifest referencing them; `policy_sha256` refers to that manifest, not just arbitrary platform metadata.
- Prefer compiling/applying the immutable expanded bytes so a reviewer sees exactly what the launcher consumed. If -D parameters are used, freeze raw profile plus exact argv, escaped parameter bytes and expansion side by side. No discrepancy is tolerated. Parameter validation rejects root paths, overlaps, symlinks, writable parents and unsafe endpoint replacement.
- Trusted supervisor reads and hashes the same protected file it launches, records immutable process identities and exec ancestry, and stores evidence outside bridge-writable state. A user-editable argv log or PID list from the tested process is insufficient. No supported kernel API exporting an effective profile digest is assumed; retain this attestation limitation explicitly.
- Where available, collect independent sandbox violation logs correlated to PID/start time, operation, target and time. Log absence is not proof of permission; log presence alone is not proof that the expected probe ran. Private `sandbox_check` diagnostics, if later available, are supporting evidence, not substitutes for actual syscalls or portable attestation.
- Show that the same actual bridge PID and its exec child acquire fresh resources only after enforcement. Close inherited authority before launch, and inspect their descriptor inventory from the supervisor. Execute fixed harmless witness/probe challenges tied to the fresh preflight nonce; capture raw error immediately, not from a later libc call.
- Require runtime positive controls in the same policy/resources session as negatives. Run role-private-state isolation across independent sandbox launches. Require fresh counterfactual diagnostic launches of the identical synthetic probes without the tested policy to establish that permissions/service liveness otherwise permit them; these do not run functional cases or access live credentials.
- Retain before/attempt/after raw observations, operation/API and result domain, target device/inode or process/listener identity, nonce, PID/start/parent/role, UTC and monotonic times, profile/package digest and per-step duration. A failed helper, malformed output, timeout or missing source is U, not a denial.
- Preserve all attempted profiles, compile failures, guard failures and failed starts in write-once attempt directories. Correction creates a new attempt, never a rewrite. Preserve increment-07 unchanged. No model call, case result, role identity, review or approval can be inferred from a valid report shape.
- Evidence reviewer must be independent of the implementer/executor. This source-informed proposal cannot satisfy code-blind designer/executor independence. The receiving manager verifies actual isolation provenance and later commissions the separately enforced blind executor.

## 8. Follow-on increment: files, bounds, tests, receiving evidence

Propose **one bridge-enforcement/witness increment**, followed by a separately allocated native verification run. Do not include actor provisioning, target case dispatch, mediation, GUI/Mach native-service helpers, a typed-validator rewrite, or firewall changes. Those dependencies have distinct acceptance decisions; bundling them into this increment would obscure the ceiling.

All following paths are proposed under the verified H parent. Existing files were read; new files are explicitly proposed, not claimed to exist:

| Exact file | Work and estimated authored changed lines |
| --- | --- |
| H/macos_bridge.sbpl (new) | Literal profile, comments and parameter contract; 105-120 lines |
| H/macos_boundary.py (new) | Trusted local supervisor: validated immutable launch inputs, synthetic fixtures, ordered witness channel, FD cleanup and PID-bound receipts; 220-260 lines |
| H/macos_preflight.py (existing) | Separate supervisor setup from confined probe mode; remove fixture creation from sandboxed probe path; replace triple calls; preserve missing rows and two positives; 100-140 changed lines |
| H/macos_adapter.py (existing) | Stage exact new files/profile pins, invoke fixed supervisor; no direct target/case execution or auth staging; 25-40 changed lines |
| H/checks_macos_boundary.py (new) | Offline tests of evidence rejection, safe argv/path map, witness provenance, missing controls and fail-closed launch; 150-180 lines |
| H/NATIVE-MACOS.md (existing) | Record new mechanism, expected ceiling, target/credential/API blockers; 20-30 changed lines |

Estimated total 620-770 authored lines; cap 800 after actual diff measurement, with fixtures/artifacts counted separately and no fabricated native results. If supervisor complexity needs more, return a scoped amendment rather than silently compressing safety checks. Do not modify H/native_guest.py, H/cases.json, original frozen cases or H/boundary-policy.txt in this increment. Preserve existing offline checks that pin their bytes. Existing checks_macos tests should remain applicable because the protected files and preflight entrypoint refusal remain unchanged.

Order:

1. Receiver freezes supervisor ownership/provisioning permissions, package-write UNPROVEN disposition, available Python runtime and this reduced expected matrix. No target decision is needed to keep target rows U.
2. Add the literal profile and deterministic validated manifest/argv generation. Seal parameters; fail on unsupported guest syntax/dependencies.
3. Extract fixture/witness work into the supervisor. Place peer processes, PTYs, FIFO and listeners outside the tested sandbox; use same-target live checks. Handle all failed/missing resources honestly.
4. Launch the bridge under Seatbelt before Python, then exec its child. Bind their raw results to the immutable launch chain. Move prompt guarding and cleanup outside P2-limited inspection.
5. Preserve packet/version positives, all remaining U rows, unchanged 175-entry validator and disabled native dispatch.
6. Format/check first. Only a subsequently authorized isolated native verification run can substantiate profile behavior.

Proposed offline test names (new, not executed):

- `test_profile_parameters_reject_overlap_symlink_and_writable_parent`
- `test_launcher_argv_is_fixed_and_policy_digest_binds_parameters`
- `test_missing_profile_or_launch_failure_never_falls_back_unsandboxed`
- `test_witness_failure_stale_nonce_or_changed_target_keeps_row_unproven`
- `test_errno_domains_and_helper_failures_are_not_coerced`
- `test_child_pid_and_profile_mismatch_are_rejected`
- `test_absent_actor_target_and_thirteen_bridge_controls_remain_unproven`
- `test_preflight_only_and_native_dispatch_remain_disabled`

Required named native evidence checks for the separate execution allocation:

- `bridge_profile_startup_and_packet_version`: actual guest profile application and two retained positives.
- `bridge_parent_child_fourteen_live_denials`: every E row attempted from both real PIDs, with exceptions left U, successful paired witnesses and raw POSIX errors.
- `bridge_pty_own_allowed_peer_denied`: actual keys through own /bin/cat PTY, external peer tty denied; supporting infrastructure only.
- `bridge_fixture_allowed_other_uds_and_tcp_denied`: exact fixture round trip succeeds while other live UDS/loopback targets deny.
- `bridge_private_state_cross_run`: one run's same-UID actor cannot access another run's state under the two distinct profiles.
- `bridge_inherited_authority_and_policy_drift`: no leaked denied FDs; tampered manifest/profile/package rejects; unsupported profile launch blocks without fallback.
- `bridge_counterfactual_witness_and_cleanup`: independently allowed synthetic controls, bounded cleanup and no surviving owned resources; profile absent/mismatched records rejected.

Before any tests, the executor must read R/docs/development/test-isolation.md. No Rust tests are in this increment; if scope later adds them, only guarded `just test` is authorized by repository policy. Use empty/synthetic profile state; stop successor dispatch on any native prompt. The current design performed none of these tests and made no provider/inference call.

Receiving artifacts belong in a fresh, write-once H/increment-08 directory (proposed): `design-pin.json`, `profile.sbpl`, `profile-parameters.json`, `policy-manifest.json`, `launch.json`, `fixture-inventory.json`, `witnesses.jsonl`, `bridge-probes.jsonl`, `bridge-child-probes.jsonl`, `positives.json`, `prompt-guard.jsonl`, `preflight-request.json`, `preflight-response.json`, `preflight-report.json`, `cleanup.json`, `receipt.json`, and hashes/seal. Native raw records are produced only by actual execution; offline examples must be conspicuously synthetic and cannot occupy a native receipt. The receiving manager can add a separately authorized repository receipt at R/qa/initiative-control/management-bootstrap/pf83-sandbox-increment-20260915.md; that path is a proposal, not an edit by this action.

Acceptance of the increment means auditable bridge-only enforcement and honest remaining controls, **not** admission of functional cases, code-blind qualification, human-test readiness or PF-83 completion. The follow-on target/actor/API/mediation work remains explicitly blocked on its recorded prerequisites and decisions.

## 9. Ten-line summary

1. Verified the frozen brief hash and management checkout base; made no repository change.
2. Selected fixed `/usr/bin/sandbox-exec` with the literal parameterized bridge allowlist above.
3. The current same-process before/attempt/after helper cannot validate a persistent sandbox denial.
4. Move live fixtures and paired witnesses to a trusted external supervisor before applying the profile.
5. Fourteen denial controls per bridge role are plausible; all expected errnos still need native evidence.
6. Thirteen controls per bridge role require missing fixtures, typed errors, tool semantics or target definitions.
7. First-increment ceiling is 30/175; package-write stays unproven, and a missing operator-home witness reduces the ceiling.
8. Launch the product outside the bridge ancestry; inherited Seatbelt can mask or break product sandbox behavior.
9. Manager/product authority must freeze target attribution, direct-versus-mediated credentials and non-POSIX evidence.
10. Propose a 620-770-line bridge-only increment, retain native dispatch disabled, and preserve every unresolved control.

## Final workspace observation

The read-only final status check still showed no working-tree changes, but HEAD had advanced to `0d734e336eb8b6e06653672572211a80d19d8caa` from the initially verified assignment base during this action. This worker ran no Git mutation or repository write. The source references above describe the inspected bytes and initial base; the receiver must account for the shared checkout's concurrent advancement rather than treat this proposal as a review of the new HEAD. The private harness source pins above remain the design's explicit inputs.
