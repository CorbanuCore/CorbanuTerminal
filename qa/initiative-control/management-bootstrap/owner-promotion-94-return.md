# RETURN — owner-promotion-94

Allocation digest:
`476fe30fbdfc41ad78f1ed1799d0fe8006cf45d7ec0c81f508c413e99a02a9d4`.
Claim: `08c00dd6-f79a-4351-8389-c2b019940a55`.
Runtime: gpt-6-astra / high.
Base HEAD verified: `0b9a6c3576420119a3f4b369be553afb610c4fb5`.
Brief SHA-256 verified with shasum -a 256:
`6f8556f619e783e9c2f9f48f7aa0cb8cbb1c74fa4d4c93479c0443278088d9b4`.

## Returned changes

The [recipe](owner-handoff-80-promotion.md) now runs the five-item
[preflight](owner_promotion_94_preflight.py) before destructive effects, stages
and validates the replacement config, exhaustive handoff and activation decision
before disarm, and repeats the complete gate immediately before disarm. The
handoff uses that prevalidated revision/selection. Existing later command fences
and actual postcondition checks remain.

The named `stale_publication_pending` refusal checks every direct publication
*.pending entry, including nonstandard filenames and dangling links, and gives
preserve/inspect/quiesce/reconcile instructions. It never deletes the artifact.
The uninstall implementation independently checks this before any bootout and
validates the plist before removal. Unapplied operations, old holds, interrupted
schedule state, invalid authority revision and other known blockers are checked
up front. The explicit OFF tick verifies persisted owner_off, including the
case where launchd's kickstart already latched it.

The field instruction now matches the existing runtime validator:
**flat reasoning_effort → nested worker.effort**. No runtime inference or
compatibility change was needed.

The [runbook](owner-promotion-94-preflight.md) provides the command, evidence
schema, refusal names, authority boundaries and all six prerequisites split
between the manager and Travis. The program reports all five items and exits
2 if any is unmet. Evidence Markdown references are hashed, not printed.
Read-only SQLite reads and service observations are separate from effects.
A passing preflight is advisory, not an authenticated approval or an atomic
reservation. The evidence reviewer must verify the actual referenced conclusions.

## Stale-publication rehearsal

The exact Python block extracted from the Markdown recipe was executed against
a disposable armed owner, installed receipt, private runtime, accepted replacement
allocation and synthetic service observation. A stale
owner-recurrence.json.pending existed before entry.

Both the initial regression and focused evidence record:

- recipe exit **2**, named **stale_publication_pending**;
- **0 disarms, 0 uninstalls, 0 launchctl mutations**;
- **49 existing files byte-for-byte unchanged**, including owner/coordinator
  databases, installation receipt, plist, runtime and pending evidence;
- admission remained **armed**, installation remained **installed**.

This is an in-process exact-recipe refusal rehearsal with a mocked service,
not an actual launchd cutover or real inference. The independent uninstall
regression also proves the guard occurs before bootout and preserves armed state.
The positive read-only gate test compares all files before/after.
Synthetic qualification declarations in tests/harnesses are fixtures only;
they do not establish a real ACK/START/RETURN or a human approval.

## Yours versus Travis's

Within existing management-bootstrap authority, the manager/integrator can do
1 (adopt the bridge), 2 (settle/replace/fresh decision), 3 (diagnose and reconcile
the publication prerequisite), 5 (quiescent audit) and 6 (execute an already
authorized promotion). The manager can also provision and run 4 if the isolated
test lane/account/inference use is already authorized. New provider/policy,
spend/credential, product scope or promotion authority still needs its actual
authority owner; these are not permissions invented by this return.

**Travis is required for any named limitation in item 4**, as final product
authority. He is also needed for missing product/go-no-go authority in the
other items. Routine technical diagnosis or a bounded repair within existing
authority does not automatically require asking him again.

An isolated inference profile requires a fresh per-run profile in an isolated
VM/test account with enforced filesystem, process/IPC, network and native-store
boundaries; all profile aliases point there. Supply a deliberately authorized
test credential/account or mediated endpoint for the exact provider/model/effort,
never an operator-profile/auth/Keychain copy. The auth link must target only that
approved isolated test credential. Preserve negative probes from the separate
code-blind executor and children, positive package/PTY controls, exact candidate
pins, correlated lifecycle evidence and independent review. env -i or an empty
CODEX_HOME alone is insufficient.

A named limitation accepted by Travis can legitimately permit **limited testing
of the named scope**. It cannot substitute for actual proof or justify claiming
full functional qualification. The gate labels that route LIMITED and retains
its missing proof/allowed scope/remaining proof. No such acceptance was supplied
or obtained here.

## Verification

Read docs/development/test-isolation.md before testing. A new venv was built under
env -i at /private/tmp/owner-promotion-94.KreyMW/venv from exactly the pinned
scripts/initiative_control/requirements.txt (markdown-it-py 3.0.0, mdurl 0.1.2,
slack-sdk 3.44.1). Runs originate from the repository root. Test environment:

```text
HOME=CODEX_HOME=CORBANU_HOME=PFTERMINAL_HOME=/private/tmp/owner-promotion-94.KreyMW
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control[:qa/initiative-control/management-bootstrap]
PYTHONDONTWRITEBYTECODE=1
```

- [Initial regressions](owner-promotion-94-regressions-first.txt): 8 tests passed,
  0.689s, zero failures/errors/skips, exit 0.
- [Focused run](owner-promotion-94-focused.txt): 252 tests passed, 100.798s,
  zero failures/errors/skips, exit 0. Breakdown: 133 owner daemon, 52 owner TMUX,
  38 feed, 22 attention/renderer, 7 preflight. The later schedule-hold test and
  final preflight changes are covered separately below.
- [Full discovery](owner-promotion-94-suite.txt):
  `python -B -m unittest discover -s scripts/initiative_control -p 'test_*.py'`;
  **831 passed**, 441.726s, zero failures/errors/skips, exit 0.
- [Final preflight tests](owner-promotion-94-preflight-final-tests.txt):
  **8 passed**, 0.802s, zero failures/errors/skips, exit 0. Includes the new
  schedule-hold refusal and final code. The stale-file replay again preserved
  all 49 files, with zero disarms/uninstalls.
- [Positive disposable recipe replay](owner-promotion-94-positive-rehearsal.jsonl):
  harness and exact Markdown recipe both exited **0**; actual disposable launchd
  promotion reached **tmux-workers / generation 3**, the synthetic worker returned,
  and cleanup observed the test service **absent**. The explicit OFF tick had
  exactly **one expected exit 2 / owner_off**; every other recipe subprocess
  exited 0. stderr is empty. No real inference/profile was used. Disposable root:
  /private/tmp/op80-lr_vfivg. This verifies the success path without upgrading
  synthetic declarations to live qualification.

Failure names: **none** in any verification run. Existing ResourceWarnings and
intentional failure-path JSON from suite fixtures remain in the raw logs.

Nonzero accounting: **0 failed top-level verification commands**. **1 incidental
command nonzero**: an inspection command's trailing rg found no matches (exit 1).
The new tests intentionally observed **6 exit-2 refusals** across the three
runs (three missing-evidence CLI subprocesses and three in-process stale-file
recipe SystemExit results). These are expected passes, not verification failures.
The positive recipe's explicit OFF tick separately returned its expected exit 2;
other pre-existing suite negative-test children are not counted as top-level
verification failures.

No formatter/fix tool was run. Six changed Python files plus the extracted recipe
pass AST parsing; all five recipe shell blocks pass bash -n. git diff --check
passes. Sprint checker: 115 current, 127 archived. No Rust changes/tests.

## Changed lines

Tracked changes, added/deleted:

| File | Added | Deleted |
| --- | ---: | ---: |
| scripts/initiative_control/activate.py | 5 | 0 |
| scripts/initiative_control/owner_daemon.py | 10 | 0 |
| scripts/initiative_control/test_owner_daemon.py | 20 | 0 |
| qa/.../owner-handoff-80-promotion.md | 55 | 38 |
| qa/.../owner-shape-93-assertion-audit.md | 6 | 0 |
| qa/.../owner_handoff_80_promotion_rehearsal.py | 25 | 12 |

Tracked total: **121 added / 50 deleted**. New runnable preflight: **273 lines**;
new preflight tests: **225 lines**; new command/authority runbook: **194 lines**.
This return and raw logs are additional QA evidence. All paths are within the
frozen writable scope. The production runtime/installer change is 15 added lines;
the effort fix is documentation-only.

## Brief corrections and limits

The stale-publication and field-naming findings are correct. The request to
evaluate **every condition that can refuse before any effect** is literally
impossible for future OS failures, concurrent writers and postconditions that
depend on performing the effect. Known current/input refusals now precede
destruction; later revision/generation/service/pin fences must still refuse if
state changes. The procedure is not transactional rollback. Keep publishers
as well as dispatchers quiescent.

The preflight checks pinned records and current technical state. It cannot
authenticate a human authorship claim, prove external senders are quiescent
from a boolean, or turn reported evidence into independently observed proof.
The runbook makes those obligations explicit. The brief's six-item ordering
separates the stale-publication prerequisite from round 93's five-item written
list; this implementation follows the requested six, with 1–5 executable.

Real counts: **0 live promotions, 0 real worker launches, 0 real ACKs, 0 real
STARTs, 0 real RETURNs, 0 native credential prompts, 0 new human approvals**.
No live owner root, coordinator or installation was accessed. No credential
contents were read/printed. No commit or push.

Bounded fix under product-spec heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog”; “preserve the last good
publication on failure.” Existing initiative-delivery-control / PF-80-S01 context;
no new plan/sprint scope or authorization boundary is claimed. This is internal
promotion tooling and evidence, not a changed user TUI flow or qualified human
handoff. Proposed code-blind/TUI N/A for this internal repair requires integrator
acceptance; later real-worker functional qualification remains open. No release,
benchmark, TensorCash/Isometric qualification or human sign-off is claimed.
