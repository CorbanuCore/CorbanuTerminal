# RETURN — tasknode-readiness-01

Status: implementation candidate; no live posting or human-test readiness claim.

## Allocation and authority

- Model/effort: gpt-6-astra / high, dispatched by Fable manager.
- Allocation digest: `bada8ab80820b3d4572b1f6c855b3de231f4ff7d571f41bb5486f9e327976c00`.
- Claim: `2ecf7e6a-bb91-4fc2-9893-e718fdaf7f9a`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-tasknode-readiness-20260914`.
- Branch: `bootstrap/tasknode-readiness-20260914`.
- Verified starting HEAD: `1fdb3fc1d85fdfaa041577146da0f0de0c1c3704`.
- Class: product initiative, existing PF-80-S01, in_progress, under active
  `docs/plans/active/initiative-delivery-control.md`.
- Product citation: **Internal delivery control — TO BUILD**:
  “Task Node receives only explicitly mapped, supported progress; no automatic
  reward, signing, financial action or task-completion claim is implied.”
- Frozen user allocation supplies this worker's exact coordinates and four-file
  scope. Shared plan/sprint front matter still names the manager checkout;
  manager must reconcile this allocation and evidence there before integration.
  Neither shared record is writable by this worker.

## Implementation

The separate `send` command requires one exact cc-ID and explicit dry-run/live
selection. Hash/schema, fresh pending observation, retry timing, PF-80-S01
mapping, workspace and enrollment are checked. Batch posting must be explicitly
OFF. An owner-only activation file binds the exact logical request digest,
event, origin, targets, expiry and owner attestations for identity, entitlement,
enrollment, lifecycle and reviewed payload. No activation/real credentials were
created. The request digest excludes credentials.

The sender never invokes flush/enqueue/retry or rewrites queue records. It
exclusively creates and fsyncs intent before transport, then a separate immutable
result receipt. Receipts record request digest, stable event-ID idempotency key,
HTTP status and allowlisted response facts, never raw response text. Repeated
calls do not POST again. Missing result, failure, timeout or interrupted receipt
write remains uncertain and requires external reconciliation. A successful
receipt does not accept or complete a task.

The activation is trusted local owner input, not a cryptographic human signature
or a native identity/credential-rotation fence. Native authority qualification
remains necessary before this candidate may be used with actual credentials.
Receipts are create-once by the adapter, not resistant to deliberate OS-owner
tampering. Future batch enablement must reconcile retained queue records first.

## Redacted identity-check receipt

Command: the documented SDK interpreter below, with
`-B scripts/initiative_control/tasknode.py identity-check`.

The actual check at 2026-09-14T21:54:31Z ran only installed
`corbanu tasknode --help`. Its help has no `--profile`, so the command stopped
before any account/session read. No helper was built, no profile was overridden,
and no tokens/auth files, wallet or signing operations were accessed.
The three tasks' Proposed status is historical, not newly verified.

```json
{
  "schema": 1,
  "mode": "identity_check",
  "checked_at": "2026-09-14T21:54:31+00:00",
  "commands": [["tasknode", "--help"]],
  "profile": "unverified",
  "entitlement": "unverified",
  "enrollment": "unverified",
  "tasks": {
    "task_865f75c6911f953c6586cdc1f4531e4f": "unverified",
    "task_b3e8506327fb906173fd68b2f642221b": "unverified",
    "task_789a0f3bd75b41d1eca20cae698f04cf": "unverified"
  },
  "network_writes": false,
  "blockers": ["installed_helper_lacks_profile_scope"]
}
```

On a supported helper, the command uses only link status, status and exact task
show commands. The CLI does not expose authoritative Tracker entitlement or
enrollment; both remain unverified even when account/task reads succeed.
Synthetic tests cover that path, scope/origin mismatch, changed task state,
home-alias conflict and stop-on-error/prompt behavior. Those tests do not access
the operator profile or prove native credential-prompt isolation.

## Verification

Exact requested suite:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
```

- Initial focused readiness run: 14 tests passed before adding receipt-write
  failure and home-alias/task-state cases.
- Final focused readiness run: 16 tests passed (0.090s), after persisting the
  receipts directory's parent entry as well as the intent file. Command: the
  same SDK suite invocation with `-p test_tasknode.py`.
- Full documented suite: 559 tests passed in 292.251s, exit 0. It started
  before that final durability correction, whose 16 affected tests were rerun
  on the final code. Existing synthetic HTTPError 500/429 cleanup ResourceWarnings
  appeared; no test failure or native credential prompt occurred.
- Plan/sprint checkers: passed, 3/3 active plans; 116 current / 126 archived
  sprints. They validate existing manager records, not this new allocation.
- No Rust changed; no cargo, Rust test campaign, deployment, push, live send,
  enrollment, task acceptance, financial action or native prompt occurred.

## Exact remaining gates (manager-owned)

1. Independent code/security review of this candidate, preserving existing
   review-budget usage; reconcile plan/sprint coordinates and four-file scope,
   then integrate and run receiving-tree checks.
2. Resolve installed helper profile support through an approved installation;
   rerun read-only identity checks in the authorized isolated qualification lane.
   Establish actual profile/account/origin and all three current task states.
3. Verify current Campaign Tracker entitlement, workspace enrollment, task
   ownership, supported Proposed lifecycle, audience/history grants and intended
   credential scope through supported native authority observations. Missing
   enrollment requires its own authorization; do not accept tasks as a workaround.
4. Review one newly observed PF-80 payload/cc-ID/digest/source time and explicit
   mapping; reconcile historical PF-76 records without rewriting/replaying them.
   Owner must create the exact bounded activation only after these gates pass.
5. Qualify identity/expiry/revocation/rotation and uncertain-outcome reconciliation
   against the exact candidate; no automatic re-POST or receipt deletion.
   A local activation and old enrollment JSON alone do not qualify live use.
6. This internal CLI preparation slice changes no TUI/product interaction.
   Request integrator acceptance of internal-only functional N/A; this worker
   does not grant it. Before live/user handoff, complete applicable independent
   code-blind design, enforced-isolation execution and evidence review, plus
   true-TUI recovery proof. Synthetic tests cannot qualify that workflow.
7. Separately authorize and execute one live progress event/recovery under the
   approved native credential boundary. This allocation explicitly forbids it.
   Keep batch posting OFF; no ongoing enablement, release or push is authorized.

Default-repository TUI acceptance, named-human acceptance and due benchmark/
release evidence are not claimed. No release is requested. The manager owns
applicability and later qualification in TensorCash and Isometric Game.

## Follow-up — tasknode-flush-guard-01

- Class: bounded fix to existing single-event duplicate-delivery protection.
  Product citation: **Internal delivery control — TO BUILD**:
  “Task Node receives only explicitly mapped, supported progress; no automatic
  reward, signing, financial action or task-completion claim is implied.”
- Allocation digest: `f8174d6fddd8f1d0b60be2e0d09aec7ab031f78a6e39039a1be4f72d33f80a70`.
  Claim: `cc14d17e-d169-46ba-98c1-c9e4aca8691b`.
  Model/effort: gpt-6-astra / high, dispatched by Fable manager.
- Frozen brief SHA-256 verified:
  `5a6ac257d6bc0f436cbeea0dc9bdd14b3597cce56ca23787c8e5482cbe439677`.
  Base: `e3bd579bf4e0c7c58ad863af2a9c6098e2297f98`.
  Branch: `bootstrap/tasknode-flush-guard-20260915`.
  Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-tasknode-flush-guard-20260915`.
- `flush` now skips any visited outbox record with a corresponding
  `send-receipts/<event-id>.intent.json`, without changing that record. This
  covers completed sends and uncertain intent-only attempts. The returned
  integer-compatible delivered count exposes a `single_sent` list of skipped
  IDs; CLI output includes that list as JSON when nonempty. Intent presence
  does not assert successful delivery. The integer contract preserves existing
  callers and their delivered-count serialization.
- Added a regression exercising production
  `send(..., live=True, transport=None)` with patched `tasknode.post`. It
  asserts exactly one call to `/events` with the exact prepared payload,
  synthetic credentials and immutable event ID as `idempotency_key`.
  The flush regression checks completed and intent-only skips, record/receipt
  immutability, another eligible event's delivery, and OFF refusal.
- No gate relaxed; no outbox/index/config/enrollment format changed. Posting
  stays OFF and `tasknode.enabled` stays false outside disposable synthetic
  fixtures. No live state, real credentials, activation, native identity read,
  Rust changes/tests, deployment or push is part of this follow-up.
- Internal transport regression only: TUI and code-blind functional execution
  are N/A for this scoped implementation return; integrator acceptance of that
  disposition and the later live/user functional gates above remain outstanding.
  No human-test readiness, release or live qualification is claimed.

Verification command (profile aliases removed; synthetic/mocked suite only):

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p "*test*.py"
```

Verification results:

- Full suite: **562 tests, 561 passed, 1 failed**, 291.590s, exit 1.
  `test_decision_feed.FeedTests.test_chain_history_links_notices_facilities_and_repeat_assessment`
  fails at `test_decision_feed.py:190`: expects “7 registered interfaces” while
  the facilities page reports 8. The assigned base already contains both this
  assertion and `facilities.py:102`'s “8 registered interfaces”; neither file
  changed in this allocation. The required green full-suite gate remains blocked
  by this out-of-scope mismatch; no test was skipped or expectation relaxed.
- Focused Task Node suite: **18 tests passed**, 0.096s, exit 0. Exact command:
  the same command above with `-p test_tasknode.py` instead of `-p "*test*.py"`.
- `git diff --check`: passed. Only the four authorized files changed.
- Full-suite output included synthetic HTTPError 500/429 cleanup ResourceWarnings.
  No native credential prompt occurred. All mocked activation/publication output
  came from test fixtures; no live publication or posting was performed.
