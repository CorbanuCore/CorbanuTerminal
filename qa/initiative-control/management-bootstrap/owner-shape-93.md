# RETURN — owner-shape-93

Allocation digest: `a1ac0c11805eaf08e8ba6244e5c7ab18c477c8143342e7ef9d1d64ca9c606065`.
Claim: `dd13005b-a2f6-4b4b-9c8f-6020e054d7c9`.
Runtime: `gpt-6-astra`, high.
Base verified: `dc8dbf5cffce83ac773c92571901f704cfa0366d`.
Frozen brief SHA-256 verified:
`486d1a5a4d58c2e4a71bfefd7bcc4326eb112f82813312b8a12768662f6943af`.

## Reconciliation

The named bridge is `owner_tmux.freeze_worker_inputs(inputs, provider=..., policy=...)`.
It takes the cycle's original input object, preserves every original value, and
adds the five-field worker block. Provider and policy have no defaults. It
validates the explicit --yolo contract and duplicate flat/nested values. The
manager must invoke it BEFORE registering the allocation and accepting the
decision. Existing nested-only allocations remain supported.

The coordinator's existing exact-input acceptance and allocation digest freeze
this binding. The daemon uses the same `worker_runtime` validator before a
claim. A missing or conflicting binding is held with a named reason, with no
claim, prepare, launch or keys. No runtime is inferred from ambient profiles,
transport config, model naming or mutable defaults.

Rejected alternatives:
- Dispatch-time derivation from the eight fields cannot recover provider or
  policy: neither is present. Assuming them would manufacture unrecorded runtime
  authority, and make later dispatcher changes alter old offers.
- Ad hoc worker-block construction in each manager cycle duplicates mapping and
  conflict rules. The single preparation bridge provides the same validation to
  manager preparation, recipe preflight and dispatch.
- Editing a prepared/claimed action in place invalidates its audit identity.
  Normal put_allocation replacement cancels old prepared actions, refuses active
  reservations, and preserves prior allocation evidence. A new manager decision
  with a fresh action ID is required.

The external live manager's preparation code was not modified. It still needs
to adopt the bridge; receipt of this patch does not retroactively repair its
already-frozen flat actions.

## Real-worker evidence and exact stop

[Disposable preflight](owner-shape-93-preflight.json) starts with exactly the
eight reported input keys and model gpt-6-astra/high. It records the expected
flat-input refusal, replaces the allocation through the coordinator, confirms
the old action cancelled and new action prepared, verifies the new digest and
unchanged original fields, then observes a disposable fixture-only owner armed
and disarms it.

Real qualification stopped **before transport validation, tmux promotion or
launch**. No approved isolated inference profile or non-live binary transport
was supplied. Using the manager's real auth-link/profile or live installation
would violate the assignment; this environment is not an isolated native
credential qualification lane. The available agent tool runtime is not a
credential bridge for an owner-launched Corbanu process.

Counts: **0 real promotions, 0 real launches, 0 real ACKs, 0 real STARTs,
0 real returns**. The probe exited 0 because it successfully recorded its
BLOCKED result; that is not a real-worker success. No auth/token contents or
native credential store were read. Shell command lookup/metadata identified an
installed corbanu wrapper, but its contents were not opened and it was not run.

The synthetic recipe evidence remains explicitly separate:
- [First replay](owner-shape-93-promotion.jsonl) completed the exact recipe
  (child exit 0), promoted generation 3 and reached coordinator returned, then
  the harness failed its immediate status-generation assertion (outer exit 1).
  Original status was not logged. Later
  [inspection](owner-shape-93-first-replay-inspection.jsonl) found armed
  tmux-workers generation 3 and explicitly disarmed it. Service cleanup was absent.
  [Retained journal read](owner-shape-93-first-replay-effects.json) confirms
  returned with exactly one applied row for each of all eleven lifecycle effects.
- The harness now logs every post-return advisory status and bounds its reread
  to five seconds; it does not change admission semantics.
- [Second replay](owner-shape-93-promotion-final.jsonl) exited 1 at uninstall:
  publication raised FileExistsError on owner-recurrence.json.pending. Cleanup
  hit the same exception. The filename “final” identifies that attempted replay,
  **not a passing outcome**. [Inspection](owner-shape-93-second-replay-inspection.json)
  confirms phase uninstalled, service absent, admission off, pending artifact
  preserved. No worker was launched in this attempt.
- Neither replay is an unqualified end-to-end pass or real-inference evidence.
  No leftover pending file was deleted to force success.

## Ordered live assertion audit and recipe

The [complete assertion audit](owner-shape-93-assertion-audit.md) walks every
explicit assertion, stateful validation and operational observation.

Established failures, in original execution order:
1. worker.worktree lookup raises KeyError before testing configured membership.
2. worker.policy lookup also cannot evaluate. Provider and policy are absent.

Both are consequences of the missing worker block. All other live-state
assertions are **unknown**, because the brief forbids live access and provides
no live snapshot or installation receipt. The separate prepared promotion action
is reported by the manager, not independently observed. Current dispatched
owner-shape-93 is not itself a prepared candidate.

The [corrected recipe](owner-handoff-80-promotion.md) supplies exact bridge,
replacement and fresh-decision instructions, checks the shared runtime validator
and allocation/input integrity, checks executable/resource prerequisites, and
checks recovery/holds/owner claims before cutover. The authority, transport,
selected IDs and live paths remain real manager inputs.

**Promotion cannot proceed merely because this round lands.** The manager must
adopt the bridge and reprepare selected flat actions, provide real isolated
qualification (or record actual product-authority acceptance of a named limited
first action), evaluate the unknown live prerequisites, and resolve the observed
publication cutover failure. This return grants no approval or waiver.

## Scope and policy

Bounded fix under product-spec heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog”; “One manager owns plan
records and publication; workers emit separate redacted reports.” The fix keeps
the existing runtime/policy boundary rather than guessing missing authority.
Context is initiative-delivery-control / PF-80-S01; no new plan/sprint scope is
claimed. The sprint checker passes: 115 current, 127 archived.

Internal implementation/evidence return only. No user-facing TUI flow is changed,
no human-test readiness, code-blind acceptance, integrator N/A acceptance,
TensorCash/Isometric qualification, benchmark, release or human sign-off is
claimed. The later real-worker functional qualification remains open.
No commit or push. No live coordinator or owner state changed.

## Brief corrections

The reported shape mismatch is correct. The request to determine every live
assertion's current truth conflicts with the explicit prohibition on accessing
live state. The audit therefore reports known shape failures and unknown state
without inventing a pass. The real-worker exception in the brief applies here.
An additional disposable publication failure is preserved rather than hidden by
a successful recipe-only result.

## Verification

Read docs/development/test-isolation.md before any tests. Created
`/private/tmp/owner-shape-93.l9zwy4/venv` under env -i and installed exactly
scripts/initiative_control/requirements.txt: markdown-it-py 3.0.0,
mdurl 0.1.2, slack-sdk 3.44.1. Commands ran from the repository root with:

```text
HOME=CODEX_HOME=CORBANU_HOME=PFTERMINAL_HOME=/private/tmp/owner-shape-93.l9zwy4
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control
PYTHONDONTWRITEBYTECODE=1
```

- [New regressions](owner-shape-93-new-tests.txt): 6 tests, 0.222s,
  0 failures/errors/skips, exit 0.
- [First full discovery](owner-shape-93-suite.txt):
  `python -B -m unittest discover -s scripts/initiative_control -p 'test_*.py'`;
  **830 tests**, 468.496s, **1 failure / 0 errors**, exit 1.
  Failing case:
  `test_owner_tmux.TmuxTests.test_bridge_live_pane_changes_and_stale_capture_refused`.
  Its fixture never showed READY before the setup timeout; capture was empty.
- [First focused run](owner-shape-93-focused.txt):
  `python -B -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention`;
  **244 tests**, 125.686s, **0 failures / 1 error**, exit 1.
  Error:
  `test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence`,
  subcase `mode='malformed'`: decisions.Invalid at the 0.7-second handoff deadline
  before the expected invalid_json observation.
- These first runs overlapped for part of execution. Load sensitivity is an
  inference, not a proven cause. Original evidence is retained.
- [Sequential focused replay](owner-shape-93-focused-replay.txt): **244 passed**,
  100.767s, zero failures/errors/skips, exit 0. Breakdown: **132 owner daemon,
  52 owner TMUX, 38 feed, 22 attention/renderer**.
- [Sequential full discovery replay](owner-shape-93-suite-replay.txt):
  **830 passed**, 447.387s, zero failures/errors/skips, exit 0.
  No code or test expectations changed between the failed runs and successful
  sequential replays. Existing ResourceWarning output is retained in the logs.

No workspace formatter or fix tool was run. Python AST validation passed for
six changed Python files, bash -n passed all five recipe shell blocks, and
git diff --check passed. No Rust tests were needed. The candidate manifest
records exact file hashes and package digest.

Tracked changes (added/deleted lines):

| File | Added | Deleted |
| --- | ---: | ---: |
| scripts/initiative_control/owner_daemon.py | 2 | 5 |
| scripts/initiative_control/owner_tmux.py | 38 | 0 |
| scripts/initiative_control/test_owner_daemon.py | 38 | 1 |
| scripts/initiative_control/test_owner_tmux.py | 48 | 0 |
| qa/.../owner-handoff-80-promotion.md | 54 | 3 |
| qa/.../owner_handoff_80_promotion_rehearsal.py | 18 | 4 |

Production/test source totals: **126 added / 6 deleted**. All tracked changes:
**198 added / 13 deleted**. New QA artifacts include this report, assertion
audit, disposable preflight runner, manifest and retained raw results.
All changes remain inside the frozen writable scope. The new disposable
preflight runner is 92 lines; the assertion audit is 109 lines.

Nonzero accounting (top-level commands, not intentional negative-test children):
**4 verification failures**: first full discovery, first focused run, and both
synthetic replay harnesses. Failure names and stages are above.
**3 incidental command nonzeros**: two rg no-match exits (1 each) and one
diagnostic Python command with an unmatched parenthesis (SyntaxError, exit 1).
That diagnostic was corrected and its successful state inspection retained.
Final total: **7 top-level nonzero exits**. One rejected identical-text edit
request made no file change and is not a command exit.

Nested replay commands are counted separately, not double-counted above:
the first recipe had **1 expected exit 2 / owner_off** and otherwise exited 0;
the second had **1 unexpected uninstall exit 1**, propagated as recipe exit 1
and then harness exit 1. Its cleanup raised the same FileExistsError within the
already-failing harness. No native credential prompt occurred.
