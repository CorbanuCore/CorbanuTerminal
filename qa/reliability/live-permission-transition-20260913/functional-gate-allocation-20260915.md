# PF-83-S01 independent functional execution allocation

## Status, authority and ownership

**Design packet only; execution is blocked on the prerequisites below.** This
record does not close the functional gate or attest an available native runner.
Action `pf83-gate-design-01`; worker `gpt-6-astra`, effort `high`; Fable is the
receiving manager. Allocation digest
`a7835b1aa5c494b91b9537b8e4e1ab684b1262ff5fdca77a0dd3d40b2f867932`;
claim `4cc8b4ac-48d8-461e-a0f7-72d760eb885f`. Brief SHA-256 verified with
`shasum -a 256`:
`9cb7759d1f8ae5c1ee212fdefde0a2b0f8150a280e552ce2464ee7b489e78419`.

Class: routine QA/process design supporting the active
[permission plan](../../../docs/plans/active/p0-security-levels.md), feature PF-83,
[sprint PF-83-S01](../../../docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md)
(`in_progress`), Remaining item 3 and its Verification/Exit evidence lists.
Product heading **Permission selection confirmation — TO BUILD** in
[the specification](../../../docs/corbanu-product-spec.md): “A submitted selection
is not a confirmed change.” Also: “Existing active-turn approval/sandbox snapshots
and pending approvals are not retroactively changed.” The recorded September 14
user amendment authorizes the next-turn command boundary, not global quiescence.

This worker read implementation findings and harness code as assigned. Therefore
this is a source-informed execution/infrastructure allocation, **not another
independent code-blind test design**. Preserve the original independent F01–F11
verbatim, including ambiguities; their file is
`/Volumes/CorbanuDrive/Corbanu/.codex-work/permission-transition.eocbhB/frozen-functional-cases.md`,
SHA-256 reverified `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.
Keep original bytes plus the explicit scope amendment in the frozen packet.
The additional G cases below operationalize coverage; they cannot replace or
silently waive an original case. Manager freezes the normalized manifest and
literal actor inputs before comparing results. Original-design provenance stays
separate from this action's identity.

The future executor must be a different fresh context from designer, implementer
and harness author. The evidence reviewer must differ from implementer, harness
executor and every acceptance executor; the original blind designer may review.
This worker cannot serve as the blind evidence reviewer after these readings.
Use the existing review ledger: integration-handoff.md recorded 4/5 used with an
evidence check reserved. That is historical usage, not a current allowance
assertion. Fable records current usage and any integrator-authorized extension;
execution sessions/costs are recorded separately. No agent was dispatched here.

## Candidate and reproducible package pin

The assigned source candidate is exactly
`e3bd579bf4e0c7c58ad863af2a9c6098e2297f98`, Git tree
`dd4ea6fc1584a10064aa15b38f50bd7eae35c259`, read from the initially clean worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-pf83-gate-design-20260915`, branch
`bootstrap/pf83-gate-design-20260915`. The document commit adds no product code.
Build in a separately allocated clean detached checkout of this source commit;
do not build a moving integration branch or label an old package with this pin.
A later source change requires a new explicit candidate and affected replays.

Target: standalone canonical Corbanu package, `aarch64-apple-darwin`, `dev-small`,
expected workspace version `0.1.42` (verify from this candidate during packaging).
The version alone is not identity. Reuse the package workflow established by
[preparation-receipt-vwQlK8.md](preparation-receipt-vwQlK8.md):

```sh
python3 -B scripts/build_codex_package.py \
  --target aarch64-apple-darwin --variant corbanu --cargo-profile dev-small \
  --cargo "$PF83_BUILD_ROOT/cargo-locked" \
  --package-dir "$PF83_BUILD_ROOT/package" \
  --archive-output "$PF83_BUILD_ROOT/corbanu-terminal-package-aarch64-apple-darwin.tar.gz"
```

The owner assigns an empty absolute `PF83_BUILD_ROOT`. Execute with `env -i` and
an explicitly constructed build environment: pinned Rust 1.95.0/toolchain PATH,
private HOME/TMPDIR/CARGO_HOME/CARGO_TARGET_DIR/XDG_CACHE_HOME, no inherited
profile aliases or inference credentials. `cargo-locked` is a hashed wrapper
around that pinned Cargo adding `--locked --offline`; it performs builds, never
raw Cargo tests. Record its exact bytes, argv, tool versions and build exit code.
Seed only dependency caches, never profile/auth/config files. Record verified
V8/rg/zsh artifact inputs and any packaging downloads separately from the offline
Cargo build. A missing dependency blocks packaging; no unrecorded fallback.

Measure Git commit/tree and tracked/untracked source diff before and after;
require the selected source tree unchanged. Retain full build stdout/stderr,
source-file manifest, builder/wrapper hashes and target/profile/signing identity.
Hash the archive and every package file with SHA-256; record mode and literal
relative symlink targets, reject links escaping the package. Include all matched
helpers (`corbanu-acp`, `corbanu-walletd`, `codex-code-mode-host`), resources and
launcher entries produced by the builder. Independently stream the archive and
compare it to the file/link manifest. Define the manifest digest as SHA-256 over
its retained exact UTF-8 bytes; retain the file map as well as the digest.
Rehash in the target before launch and after exit. Record actual launch argv,
resolved entrypoint, `--version`, OS/architecture and loaded helper identities.
Execute this package's launcher directly in a PTY, not `just codex` or a rebuild.

**New package/archive SHA-256 values are unavailable:** this is a design-only
allocation with no build authorized. Filling and freezing them is mandatory
before executor dispatch. The held-v1 binary hash `57b19153926f778f69dc12243703796c07d100e97f47253244b23d321659d9bd`
in the old preparation receipt identifies an obsolete source package, not this
candidate. Its successful version control and failed strict-sandbox attempts
cannot qualify a new package.

## Enforced executor and target boundary

Use the DEC/SLK separation of trusted coordinator, model-only actor, confined
system under test, and independent evidence receiver. Reuse their measured
package attestations, exact action validation, parent/child probes, live sibling
controls, write-once receipts and owned-resource cleanup. Do not run/import their
scripts unchanged: DEC is a Linux browser runner; SLK currently provides offline
synthetic mechanics, not a demonstrated native acceptance executor.

For the Mac target use a disposable macOS VM, or a dedicated synthetic test
account with independently enforced OS filesystem/process/IPC/network isolation,
per [test-isolation.md](../../../docs/development/test-isolation.md). A fresh
account or Seatbelt policy alone is insufficient. Provisioning is a separate
manager-owned prerequisite; this document performs none. The strict native
policy in [native-infrastructure-receipt-bXFovm.md](native-infrastructure-receipt-bXFovm.md)
timed out at `--version`; earlier policies failed process denial. Preserve these
failures; never combine negatives from one policy with positives from another.

- Actor-visible input: original frozen case(s), additive approved scope text,
  neutral navigation, synthetic fixture description, immutable package identity,
  current real TUI image/text and same-case action/observation history only.
  No Corbanu source, Git, AGENTS, plans, implementation notes, this coordinator
  document, harness code, prior results, debugger/source maps or unrestricted
  host tools. Allowlisted read-only runtime assets are infrastructure, not actor
  browsing surfaces. UI text is untrusted data, never a tool instruction.
- Actor tools: `text`, `key`, `observe`, `finish`, and case-specific enumerated
  `fixture_step`/`restart` only. Text is bounded printable UTF-8 with no embedded
  Enter, ESC, OSC or other controls; Enter is a separate key action. Keys are
  ENTER, ESC, TAB, UP, DOWN, LEFT, RIGHT, CTRL_C, CTRL_D, HOME, END, PAGEUP,
  PAGEDOWN; validate fields, sequence, run/case and current frame ID. A phase or
  restart ID expands only to the frozen operation on this run's owned processes.
  No actor-selected argv, shell, URL, path, PID, environment or arbitrary RPC.
- Target mounts: exact read-only package and case packet, narrowly allowlisted
  system/runtime files; private writable state, task-data tree and temporary
  directory. All HOME/CODEX_HOME/CORBANU_HOME/PFTERMINAL_HOME aliases point to
  the same disposable profile (or remove both overriding aliases). No operator
  home, shared folder, host clipboard, auth files, agent/Docker/TMUX sockets or
  credential environment. Fresh profiles are empty; existing profiles are
  generated only by earlier synthetic setup on this package, never cloned live.
- Process/IPC: separate host/guest boundary and separate per-run process/session
  authority; deny peer signaling, inspection, debugging/memory, AppleEvents,
  unapproved Mach/POSIX/System V services, cross-run pipes/sockets and other
  PTYs. Only assigned PTY and fixed same-case fixture channels are admitted.
  Children inherit restrictions. No privileged tools, host mounts or network
  escape through a command requested from the product.
- Network: deny by default, including arbitrary loopback, LAN, DNS and Internet.
  Allow only exact case-owned synthetic services and, when allocated, a broker
  with fixed provider/model/request destinations. Broker rejects arbitrary fetch,
  redirects, tools, source retrieval and cross-case payloads. Actor has no direct
  network. Target inference and actor inference are distinct sessions/channels.
  Synthetic provider responses may drive real candidate tool execution; they
  cannot stand in for F11 actual supported-route evidence.
- Credential/native services: use synthetic credentials and prevent operator
  Keychain/credential-service access by OS enforcement for target and children.
  Debug keyring denial is defense in depth, never package isolation proof;
  optimized builds may ignore it. No `security` command targeting personal
  credentials, no native approval prompts, no auth/token capture. Any broker
  credential exception stays outside actor/target readability and is explicitly
  attested; DEC's transport auth-read/443 exception is not a target permission.
- Resources: pin runtime/image/toolchain and per-run CPU/RAM/PID/storage limits
  in the manifest after a package positive control. Do not blindly import DEC's
  128-PID/512-MiB browser bounds for native Corbanu. Preserve the chosen limits
  across all controls and acceptance cases; resource failure remains a blocker.

Before every case or explicitly frozen group, run negative probes from the
actual actor runtime/bridge process, target and their children. Kernel policy
queries on the actual model transport PID supplement, not replace, attempted
syscalls in confined runtimes. Use existing synthetic denied source/history/auth
canaries and live owned peers, never read real credentials. Probe read/list,
symlink escape, package write, peer inspect/memory/signal, other PTY, live Unix
socket/pipe/Mach service, host agent/Docker socket, unapproved TCP/DNS/HTTP and
child exec escape. Record errno/return value, target identity, policy/mount/tool
inventory and namespace/account identity. Nonexistent targets and a dead peer
are invalid denial controls. Keep peer positives before and after the negatives.

Under the **same** effective policy/package prove packet reads, exact package
version and real TUI launch, text and separate Enter, required child command,
allowed fixture IPC and mediated inference. Product permission R/F controls must
both work inside the outer boundary. All required controls must pass before case
input/inference admission. A denied tool remains denied. Freeze policy hashes
and record fresh evidence on replacement processes/windows; no silent relaxation.

## Common case fixtures and literal action vocabulary

Create a per-case synthetic tree `/state/pf83` (native path mapping may differ;
substitute once and freeze literal resolved paths before dispatch). It contains
`workspace/input.txt` = `permission-test`, sibling `outside/`, and append-only
`events.txt`. Set candidate cwd to `workspace`; the outer sandbox permits both
folders, while the product's restricted workspace policy does not permit an
unapproved sibling write. No real secrets, money, network or unrelated files.

R = visible **Ask for approval**, reviewer User, workspace restrictions;
F = visible **Full Access**. Named permission-profile variants use their literal
frozen equivalent labels. Do not assume a protected product mode is enabled.

- `SELECT(R)`: type `/permissions`, ENTER; use UP/DOWN to highlight the visible
  `Ask for approval` row, ENTER; dismiss an informational picker with ESC if
  needed to return to composer. `SELECT(F)`: same command, choose `Full Access`,
  ENTER; in `Enable full access?`, choose `Yes, continue anyway`, ENTER.
  Capture each actual key; never assume initial focus or use unseen coordinates.
- `PROBE(tag)`: type exactly `Run this one shell command once; request approval
  if needed; do not substitute another operation: printf '%s\n' 'tag' >>
  /state/pf83/outside/events.txt` as a single line with `tag` replaced by the
  frozen case/phase ID; then ENTER separately. Freeze the resulting complete
  string in each case packet. Count actual appended lines externally, not from
  model prose. Never pre-create a pass marker or ask the model to claim success.
- `SLOW(tag)`: submit `Run once: printf '%s\n' 'tag-started' >>
  /state/pf83/workspace/events.txt; sleep 30; printf '%s\n' 'tag-finished' >>
  /state/pf83/workspace/events.txt` as one line, then ENTER. A same-turn probe
  uses a predeclared synthetic inference barrier after `tag-started` and before
  the next command response. It must not be implemented by a second user turn.
- `NEW`: type `/new`, ENTER separately. `RESUME`: coordinator's frozen restart
  action launches the exact package `resume <this-case-session-id>`; it never
  accepts an actor-provided session/path. Capture resulting visible context.
- `DENY`/`ALLOW_ONCE`: navigate the actual pending command approval to the visible
  deny or one-time allow option and ENTER; freeze exact displayed labels after
  neutral package-navigation preparation. Never choose session-wide approval.
- `WAIT(checkpoint)`: repeated observations up to the declared time cap, waiting
  for positive current output, not a vanished spinner or an old transcript cell.
  External marker/process/request counters supply independently timed evidence.

Each case includes a disjoint negative-control phase with unique markers. First
calibrate the identical probe in R (pending approval, no marker; then deny) and
F (one marker, no command approval). If this difference cannot be demonstrated,
behavioral authority assertions are blocked, not inferred from permission labels.
A fixture must never weaken the product sandbox to establish the probe.
For active/in-flight cases, record selection between the started and finished
checkpoints; if work finishes before selection, retain that attempt as blocked
on timing and replay. Never relabel an idle selection as active-turn evidence.

## Cases: preconditions, actions, outcomes and refuting controls

Every G case below is required; directions, outcomes and variants are separate
IDs in the manifest, not success-only substeps. P0 unless explicitly P1. Budgets
are **per expanded case ID**, including its control, not for the whole suite.

| ID / original mapping | Preconditions and literal actions | Observable expected outcome | Negative control | Calls / minutes |
| --- | --- | --- | --- | --- |
| G01 / F01 | Idle calibrated R. SELECT(F); observe requested and next-turn completion; PROBE(G01-after). | Confirmed next-turn F; exactly one write without stale command approval. | Same probe in initial R must await approval and write nothing when denied. | 64 / 25 |
| G02 / F02 | Idle calibrated F; fixture holds confirmation before completion. SELECT(R); while visibly pending submit PROBE(G02-held); release only declared confirmation barrier; after completion PROBE(G02-after). | No over-privileged new-turn dispatch while pending; post-boundary probe cannot write without approval; held draft/submission disposition explicit. | F-before writes once; withhold approval after R and verify zero G02 protected effects. | 96 / 40 |
| G03-RF / F03,F09 | R active SLOW(G03); SELECT(F) before same-turn probe barrier; release that barrier; deny old-turn probe if asked; finish; PROBE(G03-new). | Current turn retains R; UI explains finish/stop and next-turn F; fresh probe uses F. | Old-turn probe denied gives zero marker, new-turn probe gives one. | 96 / 40 |
| G03-FR / F04,F09 | F active SLOW(G03); SELECT(R); release predeclared same-turn probe after selection; withhold any approvals; finish; PROBE(G03-new). | Already-active turn retains its snapshot; new-turn R gates effects; UI does not claim retroactive restriction. | Distinct old/new tags prove old-turn authority versus zero unapproved new-turn effect. | 96 / 40 |
| G04-RF, G04-FR / F06 | Begin SLOW(G04) under starting mode, approve it explicitly if requested; observe started marker. SELECT(opposite) while process runs; wait finish; PROBE(G04-new). | Running work is not automatically interrupted, rolled back or replayed; separate next turn uses selected mode. | Started/finished counters distinguish actual in-flight work; restricted fresh probe withheld yields no protected write. | 96 / 40 |
| G05-deny, G05-allow / F05 | R, PROBE(G05-pending) visibly awaits approval and marker absent. SELECT(F) without resolving it; observe pending; in separate repetitions DENY or ALLOW_ONCE; then PROBE(G05-next). | Selection never approves/reissues old request; deny writes zero, allow writes exactly once; distinct next-turn probe follows F. | Unresolved dwell shows zero old marker; deny and allow branches have different measured effects. | 96 / 40 |
| G06-RFR, G06-FRF / F07 | Active harmless turn, first confirmation held. Select the three modes in ID order before settlement; try conflict while full-access consent is open; release accepted request; PROBE(G06-final) on new turn. | Accepted/refused/cancelled selections distinguishable; final accepted state matches probe; older completion cannot overwrite it. | Cancel latest offered consent or observe explicit busy refusal; it must not count as accepted final selection. | 128 / 55 |
| G07-R, G07-F / F08 | From R/F, open picker and cancel with ESC; from R also select F then choose Cancel at full-access consent. PROBE(G07-after). | No change claimed for cancellation; former behavior remains. R transition may have no post-submission cancel: record this explicitly. | Reopen and actually confirm a transition in disjoint phase; observe the contrasting next-turn behavior. | 64 / 25 |
| G08-failed, G08-unsupported-empty, G08-unsupported-method, G08-uncertain / F08,F10 | From each starting mode, frozen external fixture respectively supplies known rejection, acceptance-only {}, method unsupported, or loses completion/disconnects after submission. SELECT(opposite); WAIT(terminal outcome); PROBE(G08-after) only after safe recovery. | Failure/unsupported/uncertainty never displays confirmed success or retries blindly; known rejection preserves old authority; uncertain actual effect is reconciled, not assumed rolled back. | Pass-through real-server phase gives actual confirmation; count one request in fault phase and no implicit fallback/retry. | 96 / 40 |
| G09-RF, G09-FR / F09 | Complete a uniquely tagged workspace write, begin active SLOW, SELECT(opposite), press CTRL_C through the visible stop path; after stopped checkpoint submit PROBE(G09-next). | Stop is explicit, context/partial work visible; next-turn permission correct; completed work not duplicated. | Observe before CTRL_C that selection did not stop work; marker/request counts detect replay afterward. | 96 / 40 |
| G10-RF-new, G10-FR-new / F09 + P2 | Existing synthetic disk default is the starting mode. SELECT(opposite); WAIT(confirmed); NEW; PROBE(G10-new). | Confirmed full profile/reviewer survives /new despite conflicting disk default; probe matches. | A separate cancelled selection + NEW retains starting default; this can expose stale-config reset. | 96 / 40 |
| G11-RF-start, G11-FR-start / F09 + P2 | Fresh process has no conversation yet and an initial prompt held through a confirmed startup selection, using the qualified public startup route. Confirm opposite of disk default; permit first thread creation; PROBE(G11-first). | First native thread uses confirmed full permission bundle; initial input proceeds once. | Identical startup with selection cancelled/unsuccessful cannot adopt the requested bundle or execute held prompt. | 96 / 40 |
| G12-applied / P2,F02,F09 | Initial held text `PF83-G12: read input.txt once and report its content.`; selection pending with no active/queued turn. Release matching real confirmation/observation in each order (reply-first, observation-first). | Held input submits exactly once after matching success; one inference turn and one file read, no orphaned draft. | Keep barrier held first: zero submissions; duplicate/stale response afterward must not cause a second submission. | 96 / 40 |
| G13-failed, G13-unsupported, G13-uncertain, G13-superseded / P2,F07,F08 | Same held text plus synthetic local/remote image, mention and pending-paste variants where public UI supports them; drive named unsuccessful outcome. | Exact held input restored, attachments/bindings preserved, zero automatic submission/queue drain; user can explicitly edit/resubmit after reconciliation. | Paired real success submits once; an explicit subsequent ENTER on restored draft is distinguishable from automatic dispatch. | 128 / 55 |
| G14-active, G14-queued / P2,F09 | Held initial input exists but active turn or queued user work is also present when matching success arrives. Confirm; observe without Enter/stop; then explicitly follow continuation. | Success does not inject held text as steering or drain queue; input restored and continuation remains operator-controlled. | Idle/no-queue G12 actually submits; tagged active/queued inputs detect accidental dispatch or duplication. | 128 / 55 |
| G15-R, G15-F, G15-pending-RF, G15-pending-FR, G15-approval / F10 | Establish each state with unique markers; ordinary exit/restart/RESUME; pending variants also use owned-process interruption at declared barrier. Inspect before PROBE(G15-new); never accept restored approval implicitly. | Restored or reset authority intelligible; no silent increase, inferred completion, automatic approval/replay; retained work identifiable. No cross-process persistence default is invented. | Pair confirmed versus pending restart; zero old-approval effect while withheld and request counts distinguish replay. | 128 / 55 |
| G16-R, G16-F / product no-op contract | Effective R/F. SELECT(same); WAIT(explicit outcome); PROBE(G16-next). | Unchanged selection still has request-specific applied/no-op confirmation; no indefinite wait on deduplicated notifications. | Withhold matching outcome in paired phase: existing matching label alone must not produce confirmation. | 64 / 25 |
| G17-old-id, G17-other-thread / F07,F10 | Old response held; use actual supported new-thread/navigation route to make it obsolete, or establish newer accepted observed state. Deliver old response; PROBE(G17-current). | Stale completion cannot alter active-thread state, release unrelated held input or overwrite newer selection. | Deliver current matching response in paired phase; only it can complete current request. | 128 / 55 |
| G18-route-A, G18-route-B, G18-route-switch / F11 | Same calibrated fixture on two actually exposed supported routes; repeat G03-RF, G03-FR and G05 deny/allow as separately budgeted child IDs. If supported, switch route with change pending, then new probe. | Permission boundary/pending approvals remain safe across routes; no grant from switching. Record unsupported routes/switching explicitly. | R/F behavioral control on every route; reject a route claim lacking actual route identity and tool execution. | 96 / 40 per child |
| G19-RF, G19-FR / unchanged MCP | Active old-turn command plus case-owned real MCP service. Establish MCP elicitation/reviewer behavior, SELECT(opposite); trigger a new MCP request after refresh while the old command turn remains; then new command probe. | Shared MCP policy refresh follows latest runtime authority while old command keeps its snapshot; no false statement that all services wait for next turn. | Paired no-change baseline and blocked/released refresh distinguish actual updated binding; external MCP request/reviewer receipts required. | 128 / 55 |

G08 and G13 expand both starting directions; G12 expands both event orders;
G13 attachment variants and G18 child replays remain explicit IDs. Do not count
an unexecuted variant as passed because another row passed. F01→G01;
F02→G02/G12/G13; F03→G03-RF; F04→G03-FR; F05→G05;
F06→G04; F07→G06/G17; F08→G07/G08; F09→G03/G09–G14;
F10→G08/G15/G17; F11→G18. G16/G19 retain additional product acceptance.
Original ambiguities, impossible navigation and unaccepted scope dispositions
remain visible in the original-case ledger.

## Platform, task-data and fixture prerequisites

Run the native Mac matrix on both fresh and representative synthetic existing
profiles; named-profile persistence is an additional G10/G11 variant. Run the
permission workflows in **both TensorCash and Isometric Game** as required by
the plan. Coordinator resolves canonical origin, exact local path and base
commit, creates disposable worktrees, and exports bounded task-data snapshots
without Corbanu source/history or operator configuration into isolation. Record
snapshot and synthetic overlay hashes. At minimum G01/G02/G05/G09/G10 exercise
both live-repository task contexts; other cases need explicit applicability
mapping and retained repository variant IDs, not an assumed blanket exemption.
No live-repository path/base was resolved or worktree created by this action.

Fixture preparation is not acceptance execution. Freeze exact literal keys,
configuration, synthetic inference responses, barrier IDs and fault schedule.
A fixture may delay/drop/reorder only messages on an already supported external
transport, or stop/restart this run's owned processes. Success phases must use
the real candidate backend and actual command effects; never manufacture
Applied, edit persisted state to force success, patch product event handlers or
replace candidate authorization with a stub. Compatibility faults supplied by
a scripted server are protocol-fixture evidence, not old-binary qualification.
A deterministic fake provider supplies tool requests only; the real product
must authorize and execute them. Keep actual supported-provider F11 separate.

The following requirements cannot yet be made executable from supplied evidence:

1. **Native boundary:** no policy/environment in the supplied receipts passes
   all denials and exact-package launch together. Disposable-account/VM and
   mediated actor/target inference positive controls must be supplied.
2. **Pending/held/stale fault seams:** the checkpoint proves event-dispatcher
   tests, not an external native-TUI way to hold confirmation, cause all four
   unsuccessful outcomes, seed held startup/queued inputs, or reorder matching
   observations. The inspected TUI CLI exposes an initial PROMPT but no direct
   remote-server selector. A transport seam and literal startup navigation must
   be demonstrated on the immutable package before G02/G06/G08/G11–G14/G17
   dispatch. An unreachable branch is blocked; direct test-function invocation
   is not acceptance. The original F08 bars invented hidden failure mechanisms;
   synthetic fault variants supplement it and require an additive scope record,
   never silently replace the exposed-failure branch.
3. **MCP observable:** source-findings.md names existing
   `refreshed_mcp_binding_captures_current_approval_authority` and
   `mcp_elicitation_reviewer_uses_latest_runtime_authority` support, but supplies
   no exact native synthetic-server elicitation recipe or visible reviewer
   control. Freeze its concrete method/payload, baseline behavior and literal
   trigger before G19. Do not assert that choosing F automatically accepts MCP
   elicitation, or that a tools/list screenshot proves refreshed authority.
4. **Navigation/coverage:** obtain neutral package screenshots and exact approval,
   startup/resume and route-switch labels. Resolve two supported brokered routes,
   live-repository bases, and applicable OS targets. Windows/Linux are unexecuted;
   Mac-only success cannot qualify them. Record exact old-server binary identity
   if claiming old/new binary compatibility; scripted replies are narrower.
5. **Package/final-tree evidence:** new hashes/build remain absent. P2's focused
   nine tests passed, but p2-fixes-20260914.md records three broader unchanged
   failures and two leaky cases. This allocation neither resolves nor waives
   them, reconciles plan/sprint checkout coordinates, nor accepts human testing.

These are concrete implementation/preparation return items for Fable, not new
permission questions or claimed approvals. Freeze amendments before affected
execution, preserving every earlier packet and attempt. An integrator may accept
only an appropriately recorded limitation; no worker may delete blocked cases.

## Per-case action and time budgets

DEC `run.py` currently defaults to 19 actor calls (with bounded per-case override),
checks a 200-second next-call cutoff, and its guest has a separate lifetime.
The brief reports four cases lost to the roughly 20-action budget. Avoid both
call and wall-time starvation; raising only the outer loop is insufficient.

Use the table's 64/96/128-call budgets (25/40/55 minutes) per expanded case,
including controls; reserve the final 8 calls for recovery observations and an
honest verdict. No shared 20-action suite cap. Account text, Enter, each key,
observation, phase control and finish separately. An allowlisted deterministic
key sequence can avoid inference on every arrow, but each actual key remains
journaled and counted; cap total PTY/control operations at 4× the call budget.
Preflight/build actions have separate limits and cannot consume case allowance.
Show remaining calls, actions and real wall time to the actor after every step.

Bound text/key writes to 5 seconds, framed exchange to 10 seconds, observation
waits to 5 seconds, confirmation observation to 30 seconds, slow-command fixture
to 60 seconds and individual model transport calls to at most 120 seconds.
These are supervisor bounds, not altered product deadlines. Retain server/TUI
10/15/20-second deadline distinctions from the implementation evidence; never
sleep through them and infer success. Host wait/poll calls stay <=60 seconds.
Keep the same target alive across the case where required; if the mediator uses
450-second transport windows, replace only that stateless window, reattest its
boundary, and retain the same case/target/PTY/history. Restarting the target to
renew a harness lease invalidates a continuity case unless the case requests it.

At 75% budget consumption record a checkpoint and remaining mandatory actions.
At exhaustion preserve all artifacts, record `blocked: budget_exhausted` unless
a functional failure was already observed, stop/reap only owned processes and
seal the attempt. No inferred pass, silent extension, automatic uncertain-change
retry or continuation with leaked context. Manager may allocate a larger frozen
budget and a fresh-context replay; retain previous attempt and costs separately.
Use <=128 KiB model input with all observation identities retained and explicit
uniform truncation of old summaries; never omit a conflicting result. Limit each
frame to 4 MiB, raw PTY stream to 64 MiB and case evidence to 256 MiB. Reaching
any cap blocks completion and retains the prefix/overflow receipt, not fake EOF.

## Required artifacts and classifications

Retain an immutable directory for **each attempt and expanded case ID**, including
setup failures, interrupted cases and corrected fresh replays:

1. `intent.json`, frozen original/amendment/case packet and `manifest.json`:
   action/claim, run/case/phase, actual role/session/model/effort identities,
   candidate commit/tree/package hashes, OS/profile/task repository bases,
   fixture/transport/runtime/policy versions, exact budgets and allowed effects.
2. `build-attestation.json`, full package file/link manifest and archive hash;
   target before/after measurements, exact executable/launcher/helper identities.
   Coordinator-only source attestations never enter actor context.
3. `isolation.json` in schema 2 plus effective policy, mounts/account/process/tool
   inventory, actual parent/child negative syscall results, live-peer controls,
   package/PTY/inference positives and cleanup. Bind agent/run/case/design and
   candidate SHA-256 exactly; booleans come from these observations only.
4. Append-only `actions.jsonl`: monotonic and UTC times, actor-requested and
   actually executed text/keys, sequence/frame/phase IDs, acknowledgments, action
   result, exit status/signal/timeout and remaining budgets. Preserve failed
   requests and denied tools, not only successful actions.
5. Raw PTY bytes, timestamped pane captures/rendered text and PNGs at startup,
   picker/consent, requested, pending submission/approval, terminal outcome,
   new/resumed thread, probe before/after and recovery. Record geometry, capture
   command and truncation. Do not replace real output with expected screenshots.
6. Case-owned marker-file before/after bytes/hashes and process start/finish
   receipts; sanitized synthetic provider/MCP/wire counters with thread/turn,
   request/response and approval IDs, exact fault activation/release timestamps.
   Prove zero/one dispatch independently of transcript cells. Keep necessary
   diagnostic logs reviewer-only; actor judges visible behavior, not internals.
7. Every actor input/output/verdict and model transport exit/provenance, including
   failed calls. Record credential-broker exception, measured sandbox and cleanup;
   do not claim raw upstream HTTP/SSE capture when transport does not expose it.
   No auth headers/tokens or live logs enter captures. Any accidental credential
   material stays sealed; publish only a reviewed redacted export and its hash.
8. `receipt.json`: distinct raw functional verdict, harness outcome, contamination
   flag, cleanup/reconciliation, reason and measured actual spend. Verify removal
   by owned immutable process/container/VM identity and successful absence checks;
   daemon/query errors do not prove absence. Keep evidence before cleanup.
9. `results.json` maps all originals and expanded IDs to artifacts/dispositions;
   SHA-256 every retained artifact, including terminal receipts and packet bytes.
   Verify hashes independently before the evidence reviewer receives the set.

| Classification | Rule |
| --- | --- |
| passed (provisional until review) | Every case phase/control observed, exact candidate and isolation verified, required effects/counts correct, no unresolved contradictory evidence; successful harness exit alone is insufficient. |
| failed | Valid setup reaches an observable violation, e.g. false applied label, unapproved marker, lost input or duplicate dispatch; preserve it even if a later timeout also occurs. |
| blocked | Missing package/boundary/route/seam, unavailable navigation, invalid fixture, inference failure, timeout or budget prevents assessment. No missing case becomes passed. |
| contaminated | Native credential prompt, live-profile read, secret exposure or boundary breach invalidates acceptance use. Set campaign-wide circuit breaker; stop successor/retry dispatch, stop/reap only owned processes, preserve private evidence and investigate. Never authorize the prompt or kill unrelated SecurityAgent/securityd. |

The existing results schema accepts passed/failed/blocked/out_of_scope, not a new
contaminated enum: keep `contaminated: true` in raw receipt and map to blocked
with the contamination reason for the checker; retain any observed functional
failure in its separate field. Out-of-scope requires actual named product
acceptance plus reason/artifact; this action supplies none. Independent reviewer
checks original-case completeness, native/synthetic distinctions, identity,
negative controls and raw attempts, then the manager runs the existing schema-2
handoff checker against the **actual hashed package**. Checker success proves
traceability, not truth. No human acceptance is fabricated.

## Follow-on harness bounds and handoff limits

Proposed separate allocation: one new private `pf83-functional.<random>` harness,
maximum seven authored files and 1,900 nonblank lines total:
`run_pf83.py` <=450, `native_guest.py` <=350, `fixtures.py` <=350,
`cases.json` <=300, `boundary-policy.txt` <=150, `checks.py` <=200,
`README.md` <=100. Count additions/deletions in reused adaptations; full copied
pinned dependencies and generated package/evidence bytes are inventoried
separately. Reuse audited DEC/SLK helpers by exact hash, not mutable imports.
VM/account provisioning, credential mediation changes and product seams are
separate prerequisites, not hidden in the estimate. Stop for an integrator scope
amendment if correct enforcement cannot fit; do not compress away controls.

Order: package/fixture and boundary preparation; offline harness controls and
negative-verdict/parser checks; freeze packet/seams; independent real-key case
execution; independent evidence check/schema-2 validation; manager reconciles
sprint Exit evidence. No source/harness edits, Rust tests, build, inference,
acceptance run or push were performed in this design action.

This gate does **not** establish global quiescence, hot-swapped current-turn
permissions, revocation of authorized work, automatic approval, protected-mode
or PF27 broker activation, all-service next-turn semantics, production Keychain
behavior, installed application replacement, signed/notarized release identity,
real model-route coverage from synthetic providers, old-binary compatibility
from mock replies, Windows/Linux support from Mac runs, full-repository test
success, security-review closure, human acceptance, benchmarks or release
qualification. Those claims retain their own evidence and authority requirements.

## Read-only provenance and document validation

Read the sprint, specification/plan contract, original cases, implementation and
P2/checkpoint/integration/native preparation evidence, test-isolation and
code-blind contracts, package README, DEC README/run.py and SLK README/PROPOSAL/
run_slk.py. Source-informed navigation checked `permission_popups.rs`, picker
labels and `tui/src/cli.rs`; none is provided to the future blind actor.
DEC observed `run.py` SHA-256:
`c59d73f5d01df78cf5bec15431a05dd9ce34939588ed0ed36066c6d565d6f3b2`;
SLK observed `run_slk.py` SHA-256:
`f3b13b43cc6c4f21dc1d68956af01c432d4e8ff09dcda9d7e38dd492ff3251b1`.
These are read-time provenance, not permission to substitute them for internal
historical pins or claims of a newly executed harness.

Document-only validation: `python3 docs/plans/check.py` exited 0
(`plans: active 3/3; available slots 0`); `python3 docs/sprints/check.py` exited 0
(`sprints: current 116; archived 126`). No Rust tests were run. These governance
checks do not establish package/functional readiness. Stage only this document,
check its staged diff with `git diff --cached --check`, and commit in the assigned
worktree; record the actual commit and final line count in the return.
