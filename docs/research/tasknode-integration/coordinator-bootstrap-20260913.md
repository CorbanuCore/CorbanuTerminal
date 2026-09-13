# PF-80-S01 — management bootstrap allocation

Product initiative under **Internal delivery control — TO BUILD**, September13
explicit five-part goal; [full coordinator contract](../../../coordinatorInstructions.md).
This allocation preserves the entire objective, not a replacement MVP. Parent
coordinates startup until the Fable manager is qualified. Other product work and
old automations remain paused; no main/release or live Task Node bulk operation.

## Ownership and code boundaries

Base eb01bf006eacbeef5be07174a3f37ad124abc74c. Parent canonical worktree/branch:
`worktrees/management-workstreams-20260911`, `integrate/management-workstreams-20260911`.
Launcher worker: `worktrees/fable-launcher-20260913`,
`bootstrap/fable-launcher-20260913`, same base. Paths are relative to Corbanu root.

- Fresh Astra High launcher worker: ONLY `scripts/initiative_control/fable_launcher.py`,
  `scripts/initiative_control/test_fable_launcher.py`,
  `qa/initiative-control/management-bootstrap/fable-launcher.md`.
  Own no credentials, shared registration, source state, service or policy files.
  Target 1200 total lines, hard 1500; stop for reallocation if required.
- Parent: `scripts/initiative_control/coordinator.py`, `coordinator_cli.py`,
  `integration.py`, `test_coordinator.py`, `test_integration.py`, plus existing
  `control.py`, `decision_alerts.py`, `decision_manager.py`, `decision_feed.py`,
  `slack_transport.py` and their exact associated tests as required for linkage.
  Parent owns product/plan/sprint updates, approved private state, live Slack/HTTPS
  configuration, mainline registration and rehearsal evidence. First durable
  coordination/integration unit target 1600 total/900 non-test; hard 2000/1200.
  Later UI/live bridges get an explicit scope update before implementation.

No Rust or dependency-cache campaign. Same-sprint disjoint workers are not extra
initiatives. Shared edits and receiving tests remain serialized.

## Sequential core owner-controls allocation

### Passive wait recording, after the actual driver rehearsal

Parent implements the next small internal unit in `worktrees/coordinator-wait-20260913`,
branch `bootstrap/coordinator-wait-20260913`, base
`9bafae46ba615768d9fff3e72c6beddf54771f95`. Exact four paths:
`scripts/initiative_control/coordinator.py`, `coordinator_cli.py`,
`test_coordinator.py`, and `qa/initiative-control/management-bootstrap/wait-recording.md`.
Target200 changed lines/100 non-test, hard300/150. One Fable High material review
plus necessary scoped correction, preserving previous units' ledgers.

Add an owner-only revision-bound operation to record an already accepted manager's
prepared `wait` proposal as durably observed. Retain its rationale, frozen allocation
and evidence; do not invent a worker, claim, ACK or completed product task. It may
record an intentional wait while dispatch is paused, without changing pause state,
sprint lifecycle, human decisions or dependencies. Reject every other action kind,
already claimed/terminal waits, stale revisions and changed allocations. No new
meaningful event for this passive bookkeeping: otherwise wait completion would
continually reawaken Fable. Keep its audit/evidence and existing terminal-history
retention. Tests cover CLI/restart, duplicate/stale denial, all other kinds, active
ownership and more than48 successive waits without a growing pending queue. Parent
owns receiving integration and actual existing-wait recording after review; no
recurrence, dispatch, broader passive side effects or completion gate is authorized.

After accepted core84f16cceec87292296ff94a9d431a86d23a9fe52, a fresh Astra High
worker owns only `scripts/initiative_control/coordinator.py`, `coordinator_cli.py`,
`test_coordinator.py` and `qa/initiative-control/management-bootstrap/owner-controls.md`
in `worktrees/coordinator-bridge-20260913`, branch
`bootstrap/coordinator-bridge-20260913`, base84f16cceec87292296ff94a9d431a86d23a9fe52.
Parent relinquishes those four files until handoff. New unit target600 changed
lines/350 non-test, hard950/550; this is a new authorized bootstrap unit, not
unbounded extension of the completed first review. No live state/credentials,
launcher, integration.py, dashboard, Slack, policy or scheduler writes.

Deliver supported revision-bound owner controls to add/replace exact frozen
allocations and enable/pause an individual stream without reinitializing SQLite;
do not silently mutate a claimed allocation or leave prepared claims executable
against changed scope. Add explicit completion/archive and successor transition
operations that require verified receiving/action evidence and owner-checked
mandatory gate records; dependency predicates alone are not completion. Preserve
the three-reservation limit, owner-only authority, pause precedence, audit history,
wrong-revision/duplicate/restart refusal and the distinction between claims and
acceptance. Tests must prove these paths and denials through the CLI. No actual
sprint is completed/archived by this implementation or its synthetic tests.
The parent performs Fable material review and receiving integration, then owns
actual native tool/rehearsal wiring in a subsequent explicitly allocated unit.

## Bounded actual native handoff rehearsal

After owner-controls receiving `fc7656ee370e33277f093144ba1ccf773fb2afc4`, the
parent may exercise one actual fresh-manager decision and one fresh Astra High
acknowledgment/reconciliation worker using the existing owner CLI and private
operator artifacts. The worker has zero write scope: inspect only this canonical
checkout's branch/HEAD/clean status and confirm the launcher/owner-control entry
points exist. No product work, reviews, builds, source edits, services, credentials,
network, document archives, Task Node sends or additional workers. Bind its ACK
to the exact claim/allocation digest and its actual native identity before work.
Capture actual return and independent owner comparison, close the worker, retain
pending/history records and return global/delivery dispatch to paused afterward.
Both other streams remain paused throughout. This is one operator-driven bridge
rehearsal, not proof of a deployed automatic event controller or recurring loop.

## Read-only Slack reply collection allocation

Sequential bootstrap unit from receiving `327eade129e186a5c66a3bacbf60c2342771de86`:
fresh Astra High worker in `worktrees/slack-reply-poll-20260913`, branch
`bootstrap/slack-reply-poll-20260913`, owns only
`scripts/initiative_control/slack_reply_poll.py`, `test_slack_reply_poll.py` in
the same directory, and `qa/initiative-control/management-bootstrap/slack-reply-poll.md`.
Target 650 total/350 non-test lines, hard 850/450. Parent retains all existing
modules, registrations, credentials, live state and scheduling. One Fable High
material review plus necessary in-scope corrections; prior review history remains.

Deliver one bounded owner-invoked collector using an injected authenticated Slack
WebClient and the existing durable Coordinator event API. Exact owner-pinned
team/bot/human/channel and tracked thread-to-decision mapping, never discovered
from untrusted replies. Read replies only; no Slack sends, canonical resolutions,
worker dispatch, service/automation enablement or credential loading in the worker.
Persist original bounded reply evidence with stable deduplication across repeated
polls/restarts and distinct edited revisions. Pagination must not silently skip
answers: bounded partial scans expose continuation/coverage, not a false caught-up
claim. Failure/429 and unknown/deleted/malformed evidence must stay explicit;
polling cannot prove unseen deletions or replace qualified Socket Mode ingress.
No approvals inferred from text and no message text in diagnostic stdout. Recording
events is allowed with product dispatch paused. Tests cover real API-shaped
fixtures, identity/thread/author rejection, duplication/edit/restart, pagination,
partial results, failure and persistence-before-observed semantics. No live API
or credential access by this worker. Parent owns actual bounded replay and later
monitor activation after the relevant launch gates; this is not recurring enablement.

## Event-to-manager driver allocation

Next bounded unit after collector receiving and actual read/dedup, base
`bca6485a2e60803393bbca0ee56d98953022ed12`: fresh Astra High worker in
`worktrees/manager-cycle-20260913`, branch `bootstrap/manager-cycle-20260913`,
owns only `scripts/initiative_control/manager_cycle.py`, its adjacent
`test_manager_cycle.py`, and `qa/initiative-control/management-bootstrap/manager-cycle.md`.
Target800 total/450 non-test lines; hard1050/600. Parent retains coordinator,
launcher, native tools, private credentials/state, policy and scheduling. One
Fable High material review plus necessary scoped corrections, retaining history.

Build the minimal owner-invoked one-cycle driver on existing Coordinator and
`fable_launcher.run_launcher`, not a new daemon/framework. No native tool API is
callable from this Python module: return accepted prepared actions for the actual
host coordinator; never synthesize spawn/ACK/return or integrate anything here.
Observe enabled/owned/pending state before inference; refuse duplicate cycles.
Claim the existing manager token, freeze an owner-only briefing/artifact record,
include the three workstreams and their last three actions plus original selected
event/result evidence, and call the actual fresh launcher once. Enforce64KiB
before inference, expose missing/oversized evidence as an owner hold without
silently truncating it or consuming events. Trusted owner context is bounded data,
not model-authored policy. Describe the seed as historical and allow separately
dated owner observations; no invented current facts. Prompt short rationales under
300 bytes while preserving the core's hard1000-byte limit and exact frozen inputs.

Bind successful receipt to this manager token, exact briefing digest, binary and
actual model/provider/high/session/turn/complete-response/shutdown evidence before
calling `accept_decision`. Retain unsuccessful/stale/invalid attempts and report
ownership/reconciliation requirements; never retry the model implicitly or clear
uncertain ownership based on elapsed time, a caller boolean or PID guess. An owner
uses existing reconciliation after actual shutdown inspection. No credential load,
network, tools or state writes at import/help/OFF; execution uses the existing
explicit private auth path only through the launcher. Test injected launcher
receipts, empty/paused/owned, evidence overflow/missing, wrong identity/digest,
stale revision, failed shutdown, crash boundaries and successful accepted action.
Those fixtures are not live-manager or native-dispatch qualification. Parent owns
actual fresh-manager replay, native bridge, future supervision and recurrence.

### Actual driver and uncertain-dispatch rehearsal

After corrected driver review and verified receiving integration, the parent may
run one fresh Fable cycle through that actual driver and allocate one fresh Astra
High worker for an ACK-only native handoff. Its scope is zero writes, zero product
execution, no credentials/network/builds/reviews, and no descendants. Bind the
actual native receipt and exact allocation digest; do not manufacture tool results.
Exercise the crash window between actual spawn and durable `dispatched` recording:
retain the host receipt, inspect the actual agent, let the real claim deadline
expire, run the watchdog in a new process, and verify uncertain/duplicate denial.
Reconcile the known native identity, record its genuine ACK, close it through the
actual tool, and reconcile shutdown as failed/cancelled rehearsal work, not success.
Use a fresh manager to observe that terminal result. Security/accounting remain
paused; only this explicit delivery allocation may run, with global/delivery pause
restored in finally/recovery. Preserve all attempts, including any unexpected hold.
No sprint completion, successor implementation or recurring enablement is implied.

## Isolated model transport enforcement allocation

Stage A review02 is clean, while its four failed capability probes remain failed.
Parent authorizes a fresh Astra High worker to implement the next wrapper-owned
transport unit, not to relabel the diagnostic as accepted. Exactly four authored
files under `.codex-work/model-transport-b.I8BDWp/`: `model_only_transport.py`,
`model-only.json`, `test_model_only_transport.py`, `README.md`. Generated private
evidence is retained beneath that root. Target850 total/450 non-test lines,
hard1000/550; one Fable High material review plus necessary scoped corrections.
PF-80-S01 and its canonical worktree remain the governing allocation. No product
worker, Rust edit, shared script, service, real credential or remote change.

Read the frozen Stage A source/evidence without modifying it. Reuse its pinned
0.1.41 package and exact Astra High model-only catalog. Enforce startup-input
exclusions at the OS boundary even when synthetic AGENTS/skills/hooks/plugins
canaries exist; a newly empty home or disabled-config assertion alone is not proof.
Keep the fixed owner command builder closed to guest argv/model/effort overrides.
Set supported request/stream retry limits and verify actual request counts for
HTTP failure and dropped streams. Preserve wall/output bounds, no helper tools,
no external network/Keychain/home/source access, immutable catalog/owner evidence,
child confinement, cleanup and positive package/text controls. Actual negative
read probes must demonstrate denial, not only marker absence or nonexistent files.

Tests use only synthetic auth and a bounded loopback HTTP fixture, preserving all
failed attempts and wire identities. This may qualify the constrained local
transport profile, not an entire reasoning executor: credential mediation, target
PTY/browser actions, independent frozen-case execution and evidence review remain
explicit later gates. Do not substitute arbitrary HTTP proxying or a native host
tool for that mediator. Report exact files, counts, hashes, tested effective policy,
supported override evidence and any remaining failed gate; stop at the hard scope
limit. Parent receives the result and owns live qualification/allocation afterward.

### Transport B owner correction: supported private provider alias

Feynman returned788/361 lines and closed: the OS/text diagnostic passes, but the
normal profile rejects reserved built-in `model_providers.openai` before inference.
Original four-file source is retained in `evidence-02y437g5/four-file.diff` with
hash manifest and failed replay `evidence-es59ei5y`. Do not alter that evidence.
Parent verified an existing supported custom-provider route, not a source bypass:
fixed alias `isolated-openai-transport`, display name `Isolated OpenAI transport`,
Responses wire, `requires_openai_auth=true`, zero request/stream retries and the
same synthetic loopback service/model/catalog. Actual pinned0.1.41 normal output,
503 and dropped stream each sent exactly one Astra High/Responses Lite request
and cleaned up. Preserve `evidence-parent-alias-3tdui4rw` (bad dictionary/TOML probe)
and corrected `evidence-parent-alias-4qiiy100` (supported scalar fields).

Parent owns this same four-file correction, inside the unchanged850/450 target
and1000/550 hard scope. Replace only the rejected built-in provider configuration
with that fixed alias, qualify every ordinary test through the actual execute path,
retain explicit default-policy diagnostics and all original failures, then obtain
the already-allocated Fable High material review. This is no model substitution,
new endpoint/auth store or permission expansion. It does not authorize live auth,
a public proxy, Rust validation changes or acceptance of functional product tests.

## Supported live model-transport preflight

After transport B review01 and its clean documentation correction, parent owns
one bounded native-auth preflight under `.codex-work/transport-live.ZBeaq9/`.
Exactly one operator script `probe.py` plus generated private attempts/receipts;
target200/hard300 authored lines. This uses the goal's bounded live qualification
authority, not product execution. Frozen B source/fixtures remain unchanged.

Pin the same0.1.41 binary, catalog, model-only flags, fixed custom provider alias,
Astra High and zero request/stream retries. Send only a fixed nonsensitive text
probe through Corbanu's supported ChatGPT authentication and native endpoint
selection. Parent verified `/Users/Neo/.codex/auth.json` is0600 with ChatGPT-mode
credentials and an access expiry September15; do not print, copy to an executor,
export, refresh or rewrite credentials. The trusted model transport may read that
exact existing auth file; it is not the future confined action executor. Its empty
environment and read-only auth access must exclude host config, instructions,
skills, history, source, Keychain and unrelated credentials. Capture only validated
assistant output in the public receipt; raw operator diagnostics stay private.

A preliminary Seatbelt domain-filter syntax check rejected `chatgpt.com:443`:
the filter accepts only `localhost` or `*`, not a hostname. Use HTTPS-port egress
in the trusted model transport plus the fixed native default ChatGPT endpoint,
normal TLS verification and no caller-supplied URL/headers/argv or executable tools.
This is not domain-specific OS filtering or guest network authority; disclose the
layered bound. Keep fork/helper execution denied,90s wall and256KiB output caps,
private fresh state, exact process-group cleanup and immutable source-auth hashes.
Preflight/failed connection attempts are retained, no silent retry or model fallback.
An auth expiry/refresh failure holds live admission and uses supported operator
reauthentication outside this read-only probe; it is not permission to clone more
credentials or relax their protection. No recurring operation or product test pass.

First attempt `attempt-rk_95g3z` failed in-process app-server initialization before
a session because native state was pointed at the read-only real auth home. Keep
that denial and unchanged-auth receipt. Use a fresh private home with one
owner-created auth.json symlink to the same exact read-only source file; the
explicit source-file read allowance is unchanged. Do not grant real-home writes,
copy tokens or expose this link to an executor. This separates native working
state from the already-authorized supported auth read without broadening access.

Second attempt `attempt-a5uobjaw` started a fresh native session but failed sending
the HTTPS request; source auth remained unchanged. A separate unauthenticated
curl diagnostic reproduced DNS denial and then reached the native endpoint
(HTTP401, as expected without auth) after allowing the exact macOS resolver
socket `/private/var/run/mDNSResponder`. Authorize that socket only for the trusted
transport, in addition to HTTPS443, not general UDP/Unix-socket or guest access.
Preserve both failures and a new per-run policy receipt. Keep the fixed profile's
two exact web-search deprecation warnings separately identified; they are not
runtime failures. All other error items and failed/error events still fail the
preflight, regardless of whether expected text was also emitted.

DNS-only corrected attempt `attempt-zii29ueb` still failed HTTPS without changing
auth. The inspected shared `http-client/src/custom_ca.rs` supports `SSL_CERT_FILE`
and selects Rustls for an explicit PEM bundle, avoiding a grant to user Keychain
services for native TLS. Authorize reading only the existing root-owned public
system bundle `/private/etc/ssl/cert.pem` via this supported environment setting;
record its hash and immutability. Do not disable certificate/hostname verification,
add a private CA, change host trust settings or allow Keychain access. This is an
explicit separately recorded preflight correction, not a claim about frozen B.

## Isolated-executor transport falsification allocation

### Local Docker boundary feasibility, parallel to fixed-transport extraction

Read-only host inventory found Docker Desktop27.5.1 running Linux6.12.5-linuxkit,
ARM64, with cached `python:3.12-slim` image
`sha256:944a73bd27d7b5fad206369e77bb7a6da51e073236b9dece9ff387df433de7bb`
(repository digest `python@sha256:57cd7c3a7a273101a6485ba99423ee568157882804b1124b4dd04266317710de`).
A cached ARM64 Scrapling browser image also exists, but no browser runtime has
been launched or qualified. UTM lists two stopped macOS VMs; no VM was started.
VMware's read-only inventory command stalled and only that command was terminated.
Docker reports its default seccomp profile as unconfined: do not inherit that as
an accepted sandbox. This is a potential local Linux route, not current isolation
proof or a replacement for private remote dashboard access.

Parent owns a separate three-file engineering probe in
`.codex-work/docker-boundary.WcQbTX/`: `probe.py`, `guest.py`, `seccomp.json`,
target300/hard450 authored lines plus generated private receipts. No overlap with
Hilbert's five canonical files. Use only the pinned cached Python image, no pull
or installation, and owned disposable containers. Non-root, read-only root,
no capabilities/new privileges, private PID/IPC/network namespaces, networknone,
explicit seccomp denying namespace/mount/ptrace/host-control operations, finite
memory/CPU/PID/tmpfs limits. No Docker socket, credential/home/source/state mount;
the only host bind may be this exact synthetic guest script, read-only.

Probe actual parent/child source/history/credential namespace exclusion, symlink
escape, package/root writes, forbidden syscalls and external networking, plus
private scratch/interpreter/loopback positives and effective resource/config
records. Distinguish namespace invisibility from EPERM and kernel limits from
actual exhaustion. Bound owner Docker calls/output and retain failed attempts;
remove only validated, labelled containers created by this probe, after capturing
terminal state. No host Docker-daemon/VM/security-policy changes or existing
container stops. One scoped Fable review after engineering proof plus necessary
correction, preserving prior reviews. This does not qualify a browser, product
binary or independent reasoning executor; later bounded stdio mediation may avoid
host-network/socket exposure but needs its own explicit allocation and proof.

### Canonical fixed-transport extraction, after the live preflight

The successful native preflight closes the model-connection uncertainty only.
Authorize one fresh Astra High implementation worker in the separately recorded
`worktrees/isolated-model-transport-20260913`, branch
`bootstrap/isolated-model-transport-20260913`, from the commit containing this
allocation (record its exact40-character base in the dispatch receipt).
It owns exactly five new canonical files: `scripts/initiative_control/isolated_model_transport.py`,
`test_isolated_model_transport.py` and `isolated-model-catalog.json` beside it,
plus `qa/initiative-control/management-bootstrap/isolated-model-transport.md`
and its own `transport-allocation.json` in that QA directory. Target900 total/450
non-test, hard1200/600; one Fable High material review plus necessary corrections.
This is one sequential PF-80 bootstrap unit, not a new initiative. Parent owns
all existing modules, private auth/state, live requests, integration and activation.

Extract the already-proved fixed trusted model transport into an import-safe,
testable host library; preserve the B profile/catalog/package pins and explicit
DNS/public-system-CA corrections. An immutable owner configuration selects exact
verified binary/auth/run-root paths, never guest data. The only per-inference
content is bounded UTF8 text; no guest-selected argv, model, effort, providers,
endpoint, headers, files or tools. Fixed Astra High, native auth/endpoint, zero
retries, fresh private state, source-auth read-only link, original policy denials,
90s wall and256KiB aggregate output. A packet may be up to64KiB, so replace B's
4KiB pipe-safe fixture shortcut with genuinely bounded nonblocking input/output.
Preserve attempts/receipts and exact error-versus-known-warning classification.
Return only validated text and safe identity/status receipts; raw diagnostics
remain owner-private. Import/help/OFF must not read auth or invoke a process.

Tests use synthetic credentials and test-only injected loopback/child fixtures,
never real auth/network. Cover same actual public execution path for input/output
bounds, timeout/cleanup, malformed/unknown/error events, unchanged auth/catalog,
path/symlink/package refusal, fresh-session identity and no fallback/retry. Retain
the private B negatives as historical supporting evidence, not tests of changed
code. Parent performs actual exact-candidate OS/live requalification afterward.
No socket mediator, browser/PTY executor, HTTP proxy, daemon, worker dispatch,
Slack call or source profile rewriting is part of this extraction. Those remain
explicit next integration gates, not excuses to expose host capabilities.

Parent inspected the read-only Hypatia audit and verified the Astra catalog's
`code_mode_only` precedence in source. Its inspected default standalone0.1.36
is not our current0.1.41 candidate; preserve that limitation. RTX SSH timed out,
so no remote packaging, forwarding or runtime claim is established. The existing
646-line/12-test five-file boundary runner and all old evidence remain frozen.

September13 host clarification: Travis confirms RTX belongs to a different
tailnet; Alex's RPC uses direct SSH. The parent verified direct
`pfrpc@178.156.143.199` access independently of Tailscale. RPC has bubblewrap0.9.0,
Python3.12.3 and systemd255.4; these are not the RTX runner's versions. A bounded
rootless namespace/`true` probe failed with `bwrap: loopback: Failed RTM_NEWADDR:
Operation not permitted`. RPC is not yet an accepted replacement executor. No
host policy, packages, tailnet or network isolation were changed to bypass it.
Private browser access is a separate route qualification, not an SSH/publication
prerequisite. Stage A remains local/offline and does not wait on either tailnet.

Authorize only Stage A locally now: fresh Astra High worker may create exactly
four private files under `.codex-work/executor-route-audit.Nerv9u/model-only-v1/`:
`model_only_transport.py`, `model-only.json`, `test_model_only_transport.py`,
`README.md`. Target650 total/400 non-test; hard850/550. This is manager-owned
infrastructure within PF-80-S01, not PF81 activation or a fourth initiative.
No canonical source edits or second storage/proxy/daemon framework. Parent owns
the one Fable High boundary review and necessary corrections. The audit's
eight-file full executor proposal is not simultaneously allocated.

Use exact existing macos-candidate-final9 Corbanu0.1.41
SHA256 `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`.
Build the smallest fixed model-only command/profile on supported CLI/catalog
configuration, with a neutral direct-mode Astra entry preserving required wire
metadata. Explicit Astra/High, no fallback/session reuse, no operational tools,
hooks, host docs/memories or guest-selected paths/providers/commands. First test
only synthetic owner-private homes, file auth and loopback fixture servers; never
read/use real auth, Keychain, models or external network. Bound processes, output
and cleanup; inspect captured real request/response and hostile unadvertised tool
calls for shell/file/image/MCP/code-mode/collaboration/extension capabilities.
Exercise synthetic hook/notify/AGENTS/memory/plugin/config canaries and ordinary
text success/invalid configuration failure. Distinguish wire-content proof,
unsupported-tool responses, marker absence and filesystem-denial proof honestly.

Return exact package/profile/catalog/argv hashes, actual tests and failures,
remaining limitations and frozen four-file diff. If supported configuration
cannot remove a required capability, identify the exact remaining registration
and stop that admission path for a separately scoped source correction. No
prompt-only isolation, live credentials/inference, remote SSH/forwarding, namespace
changes, full mediator/action loop or product/browser/Slack acceptance is authorized
by Stage A. Parent chooses and allocates subsequent enforced executor work after
examining this falsification evidence and current machine reachability.

## Isolated browser prerequisite allocation — September13 receiving8665ff1c9

The accepted text transport and corrected Docker boundary do not yet provide
visual observations or an action executor. Authorize two disjoint bounded
prerequisites, preserving their original evidence and all product pauses.

Fresh Astra High worker in `worktrees/isolated-image-transport-20260913`, branch
`bootstrap/isolated-image-transport-20260913`, base
`8665ff1c962ff2a6aa6a3157eb2251942d438c3f`, owns exactly
`scripts/initiative_control/isolated_model_transport.py`, its existing adjacent
`test_isolated_model_transport.py`, and new
`qa/initiative-control/management-bootstrap/isolated-image-transport.md`.
Target400 changed lines/200 non-test, hard650/350. Parent retains integration,
policy, real credentials and actual native qualification. One fresh Fable High
material review plus necessary scoped corrections, with previous reviews retained.

Add one optional bounded inline PNG observation to the existing public text
execution method. Never accept an image path, URL, guest-selected flags or a new
provider/model. Keep text-only callers unchanged, OFF inert, all existing tool,
credential and process restrictions. Validate byte size and PNG dimensions before
launch; reject unsupported/malformed inputs. Store only validated bytes under one
fixed owner-created exclusive private per-attempt filename. Verify the exact
packaged CLI's supported image attachment syntax, grant read access to that one
file only, and pin/hash its bytes before and after execution. No image editing,
remote fetch, new dependency/parser framework or broad directory read permission.
Tests use synthetic images/auth/fixtures and the existing execution path; cover
OFF, rejection before auth/process, bounds, unchanged text behavior, exact argv,
read policy, image mutation and preserved failure evidence. This is internal
transport engineering, not independent visual functional acceptance. Parent owns
the later actual native screenshot-observation proof and isolated action loop.

Parent owns at most three authored files under a new private
`.codex-work/docker-browser-*` root: `probe.py`, `guest.py`, `README.md`, target300
lines/hard450, plus generated evidence. Inspect and launch only cached ARM64 image
`sha256:fdbf57e27079258be976cf5d8d309c1c00faadfc24cec5c562caf661b27926bd`
with the already-reviewed corrected seccomp policy, non-root/read-only root,
no network/capabilities/new privileges, private IPC/PID and bounded resources.
No pull/install/host service change, auth/source/socket/profile mounts or user
container operations. The single permitted bind is the synthetic guest script,
read-only. Inventory installed browser/Playwright paths and attempt a synthetic
local-page render, screenshot and actual keyboard/mouse interaction. Browser
children remain under the outer container boundary; record any browser-specific
sandbox limitation rather than weakening the container policy. This preflight is
engineering, not acceptance of dashboard, Slack or a full reasoning executor.
Persist create intent before the Docker request; after any ambiguous create, hold
further admission until exact labelled-container owner reconciliation. A missing
immediate inspect result cannot close that uncertainty. Preserve failed attempts,
effective runtime/config hashes, and remove only exact validated owned containers.
Parent reviews the actual observations before allocating the combined action loop.

Actual non-root inventory found Playwright1.59.0 in `/app/.venv`, but its Chromium
1217 cache lives under image-only `/root/.cache/ms-playwright`, inaccessible to
UID65534. Parent authorizes extracting only that pinned image's public
`chromium-1217` runtime from a never-started labelled disposable container to the
same new private probe root, then mounting the extracted directory read-only at
`/browser` for the synthetic probe. Record extraction intent, image identity and
all runtime file hashes; reject links/special files before reuse. No host browser
profile, real user home, credentials, source or socket is copied. This does not
grant root execution, modify the cached image/host permissions or install packages.
The original single-bind inventory remains historical; the browser attempt has
exactly the guest-script and public-runtime read-only binds. `/tmp` stays noexec.

## Launcher interface for coordinator integration

Standalone CLI reads a bounded JSON briefing from `--briefing`, fresh private
`--runs-dir`, exact `--binary`, owner-only `--auth-file`, bounded `--timeout`.
No network on help/import/offline tests. Its stdout returns one redacted JSON
receipt containing run_id, status, model, provider, effort, session_id, decision
and artifact paths. Errors/timeouts return a nonzero exit and retained receipt.
An outer caller supplies credentials privately; never put credential values in
briefing, argv, stdout or exported artifacts. Exact model claude-fable-5-1-plan,
provider claude-plan, effort high; no fallback/resume/shared session history.
Use actual Corbanu interactive TMUX, separate text/Enter, full final response
capture and authoritative session identity. Fresh application home and neutral
packet CWD; tools restricted to required read-only manager work. A new home is
context isolation, not full functional-executor containment.

Manager decision JSON: `state_revision` integer, `actions` list. Each action has
stable `id`, `kind`, `workstream`, `sprint`, `rationale`, `inputs` object,
`timeout_seconds` and exact `expected_revision`. The coordinator validates the
allowed kind, policy authority, scope, dependency/receipt state and revision;
the launcher validates only framing, actual model/session and complete response.
Never treat pane quietness or a JSON snippet in tool output as final acceptance.

## Full completion evidence

1. Fresh Fable: two actual independent runs, correct model/effort/session IDs,
   complete structured decisions, bounded timeout/cancel and clean shutdown.
2. Coordination: durable event → fresh manager → validated action → actual native
   worker dispatch → ACK/result → new event; deduplication, stale decision denial,
   crash/restart and silent-stall watchdog tests. No fake dispatch receipts.
3. Integration: exclusive writer, actual temporary Git merge/receiving test and
   receipt; concurrent writer denial, conflict/failure recovery, branch/candidate
   checks and dependency-complete successor promotion. No unchecked worker claims.
4. Slack/private HTTPS: actual authenticated question, actual human reply,
   canonical decision, native-agent ACK, restart/recovery and stable links usable
   without this Mac; approved audience/access-denial tests. Preserve old journals.
5. Initialization/rehearsal: real three-lane baseline and approvals, aligned policy,
   complete handoff and failure/restart rehearsal, then qualified recurring enablement
   and final Slack completion ping. Until then no completion or running claim.

## Verification and limits

Use exact nonzero test receipts, all prior failures retained. Initial internal
launcher/queue engineering has reasoned internal-only N/A for GUI acceptance;
the later combined dashboard/Slack workflow requires independent code-blind
design/execution/evidence under root policy. Do not relabel smoke tests as that gate.
One fresh Fable material review per returned unit plus necessary correction;
extensions preserve prior PF80 usage and require purpose, not a reset.
Do not grow an unbounded process framework: implement these explicit operations
on existing Corbanu, native tools, private state and supported Slack machinery.
