# RETURN

Allocation `acct-decay-87`; model `gpt-6-astra`; effort `high`.
Claim `d1274d2d-3a24-4fc2-910e-cb7784c5de42`.
Allocation digest `a01b8589044a57cadd83c8d0abb3bd585c3d3514c5d695c2f31123d02a48ab76`.
Brief SHA-256 verified before work:
`6ae042707deba06554e21f1cae1e93ee8b64dfe01faef7c839113de570135e4e`.

Routine evidence-tool maintenance under PF-60 / PF-60-S03.
Product heading **Product measurement**, subsection **Measurement targets**:
“No commercial performance numbers have been supplied.”
Assigned worker branch `bootstrap/acct-activation-20260916`, base
`dfd5c09bfd2b37eeaa3e00443b176e34d0c991a9`.
No Rust/product behavior changed. Functional design/execution, true-TUI,
live-repository and benchmark qualification are not applicable to this internal
verifier revision; independent final-package qualification remains due for S03.
The manager owns canonical sprint/plan updates and acceptance of that N/A.
No human acceptance, review approval, sprint completion or release is claimed.

## Local-package refusal

`--local-package` now always returns **3**, prints **BASELINE REFUSED**, and
labels the earlier comparison **RETAINED BASELINE MATCH**. It still hashes local
files without launching them. It never claims that the combined diagnostic
counts agree with the retained baseline. The document and legacy finalizer
description now state that contract.

[Thirteen proposed-snapshot controls](proposed-controls/verifier-checks.json)
passed, including the following synthetic local-package cases:

| Case | Agreements | Disagreements | Unavailable | Exit |
| --- | ---: | ---: | ---: | ---: |
| No local package | 20 | 0 | 6 | 3 |
| One incorrect local file; two absent | 20 | 1 | 5 | 3 |
| Three matching synthetic files | 23 | 0 | 3 | 3 |

All three explicitly require the refusal and reject an unqualified
`BASELINE MATCH:` line. The synthetic matching case changes only its disposable
manifest and inert files, then restores the manifest; no binary is executed.
The other controls retain wrong-claim, corruption, missing-log and baseline
count/identity/exit/missing/duplicate rejection. The unchanged default contract
remains **20 / 0 / 3, exit 2**.

## Asserted flag and meaningful pins

[check_current_tip.py](../acct-selfcheck-84/check_current_tip.py) now resolves
`refs/heads/integrate/management-workstreams-20260911`, rather than worker HEAD.
The actual tested commit is **4dcf6da1f324b2fd8773c26b3a296b51b3f28fdf**.
A fresh full local shared clone was detached there, with empty Git status before
and after and no ignored package or working-tree edits copied.

The reference is the round-84 proposed-snapshot stdout committed at
**dfd5c09bfd2b37eeaa3e00443b176e34d0c991a9**:
`acct-selfcheck-84/proposed-controls/clean-checkout.stdout.txt`.
It includes the baseline assertion that the old round-81 reference lacked.
Reference bytes are read with `git show`, not from mutable working-tree files.
Their SHA-256 is
`023c3e6dcc8dc42b0d01d477454328084ae30b7074a3887dbf9c68841395b07c`.

The runner asserts `stdout_matches_prior_replay`, matching exits, empty stderr,
clean status and unchanged integration ref. It saves the receipt and raw outputs
before asserting, so a failed replay is retained. [Receipt controls](match-flag-checks.json)
accept the actual receipt and reject both a false match flag and a changed tip.

## Verbatim current-integration-tip acceptance run

Command, from the clean clone's repository root:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inventory-79/verify_acceptance.py
```

Stdout, verbatim (also retained as [raw stdout](current-tip/acceptance.stdout.txt)):

```text
Acceptance reconciliation: retained evidence only; no new functional qualification.
AGREE scope-zero: priced_attempts=4; USD=0.00284; fresh_root_rendered_USD=0; selected quote revisions only
AGREE priced-page counts: priced=9; unknown_cost=4; zero_recorded=2; derived from bound page contents
AGREE mutation coverage: named_checks=147; failing_diagnostic_mutations=159; each named reason reached
AGREE capture bindings: JSON_pages=24; viewports=45; content SHA-256 agrees
AGREE round-76 prerequisites: build exit=0; raw log SHA-256 agrees
AGREE round-76 core-default: passed/run=124/124; skipped=3545; failed=0; timed_out=0; flaky=0; leaky=0; exit=0; failure_names=[]
AGREE round-76 alone-feature: passed/run=1/1; skipped=3671; failed=0; timed_out=0; flaky=0; leaky=0; exit=0; failure_names=[]
AGREE round-76 core-feature: passed/run=127/127; skipped=3545; failed=0; timed_out=0; flaky=0; leaky=0; exit=0; failure_names=[]
AGREE round-76 tui: passed/run=91/91; skipped=4078; failed=0; timed_out=0; flaky=0; leaky=0; exit=0; failure_names=[]
AGREE round-76 aggregate: gate_executions=342; gate_passed=342; alone_executions=1; overlapping test sets
AGREE round-66 historical timeout: TRY durations=60.013,60.012s; passed/run=126/127; timed_out=1; exit=100; test=suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity; cause remains unproven
AGREE round-66 default timing: 58.790s; target test raw PASS line agrees
AGREE round-69 alone timing: 31.743s; target test raw PASS line agrees
AGREE round-69 feature timing: 31.335s; target test raw PASS line agrees
AGREE round-75 feature timing: 31.283s; target test raw PASS line agrees
AGREE acct-controls-72 inventory: entries=55; current_bytes=350442; current_text_lines=6352; hashes agree
AGREE acct-acceptance-75 inventory: entries=18; new_files=15; new_bytes=49726; new_text_lines=664; modified_files=3; modified_bytes=16713; modified_text_lines=230; current_bytes=66439; current_text_lines=894; full-file sizes, not diff lines
AGREE inventory classifications and correction: 15 added paths and 3 modified paths against original base; round-72 changed entries=3
AGREE package build evidence: build receipt and raw log agree; launch/functional acceptance not claimed
UNAVAILABLE committed package codex: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex is not tracked; unavailable from a clean checkout; cannot re-derive bytes=607785336, mode=0555 or SHA-256
UNAVAILABLE committed package codex-code-mode-host: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex-code-mode-host is not tracked; unavailable from a clean checkout; cannot re-derive bytes=93937576, mode=0555 or SHA-256
UNAVAILABLE committed package rmcp_test_server: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/rmcp_test_server is not tracked; unavailable from a clean checkout; cannot re-derive bytes=11508704, mode=0555 or SHA-256
AGREE acceptance numerical coverage: no unmatched digit-form quantities outside matched claim spans, fenced code and link targets; known round/sprint/severity identifiers excluded; worded quantities and excluded regions not audited
BASELINE MATCH: documented counts, evidence exit and unavailable identities match
RESULT agreement=20 disagreement=0 unavailable=3 exit=2
```

Process exit **2**; stderr empty. The [receipt](current-tip/receipt.json) records
the exact command/interpreter, full commit, reference and output hashes.
The current-tip control harness additionally passed **11/11** with exit **0**;
its [stdout](current-tip/controls.stdout.txt), stderr and individual attempts are
retained. This is the untouched integration verifier, not the proposed worker
patch or a replay from its parent.

There is **no observed retained-evidence decay at this integration tip**.
The integration tree differs from this worker's base by **212 files,
20,229 insertions and 382 deletions**, including other workstreams. Those changes
did not alter the acceptance output.

Durability comes from a bounded retained corpus: committed page/log/receipt
bytes, content hashes, exact evidence identities, explicit expected counts and
the historical Git tree used to reconstruct inventory membership. These checks
do not use today's clock, require ignored local binaries, or rebuild current
product code. Additional unrelated rounds do not change those inputs.
This proves saved-record reproducibility, not present-day product correctness,
settlement accuracy, package availability or functional qualification.

The strict no-decay helper compares all stdout bytes, including inventory
byte/line totals. This revision intentionally edits the acceptance document and
refreshes its inventory, so its proposed default stdout differs in those totals
while still producing **20 / 0 / 3, exit 2**. After integration, the historical
exact-output helper will detect that change; it must not silently call it the
same output. A reviewed new reference can be pinned for a later exact-output
comparison. The documented baseline verifier itself continues to agree, as
the proposed clean-checkout control and final-tree check demonstrate.

**One-week answer:** No unconditional guarantee: a full-history clone will get
the documented **20 / 0 / 3, exit 2** while later rounds preserve the retained
evidence and verifier contract; arbitrary edits or missing historical Git
objects cause reported drift, not automatic agreement.

## Required fresh gates and exact counts

Read `docs/development/test-isolation.md` first. Built prerequisites before tests
from the assigned checkout's `codex-rs`, using shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`, build jobs two and
debug assertions enabled. All Rust tests used guarded `just test`.
These are fresh worker-tree gates at the assigned base, distinct from the
current integration-tip retained-evidence replay above.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 |

**342/342 executions** across overlapping filters; exact failure names: **none**.
Prerequisite build exit **0**. The historically intermittent auxiliary-scope
test passed at **31.818s** default and **32.800s** feature; those passes do not
resolve its earlier unexplained timeouts.

[Test results](test-results.json) and `gates-01/` retain commands, counts, hashes
and compressed raw logs. Isolation banners were present; no native credential
prompt or live-profile access was observed. No auth/token files were read.

[Final-tree checks](final-tree-check.json) compare every changed acceptance/control
script, document and inventory against the tested proposed snapshot and compare
its stdout with the final working tree. Python syntax and `git diff --check`
passed. No workspace-wide formatter ran.

## Changed lines and corrections to the brief

Seven existing files: **+135 / -70**.

| File | Added | Removed |
| --- | ---: | ---: |
| `acct-acceptance-75/acceptance.md` | 4 | 2 |
| `acct-acceptance-75/scope.json` | 7 | 7 |
| `acct-fitness-76/finalize.py` | 2 | 1 |
| `acct-inventory-79/check_verifier.py` | 28 | 4 |
| `acct-inventory-79/inventory-correction.json` | 19 | 5 |
| `acct-inventory-79/verify_acceptance.py` | 8 | 4 |
| `acct-selfcheck-84/check_current_tip.py` | 67 | 47 |

New runner/finalizer/check scripts, this return and raw evidence are inventoried
separately with line counts, sizes and SHA-256 in [scope.json](scope.json);
that generated inventory excludes itself. Prior round evidence is preserved.
All writes and disposable clones are inside the authorized QA subtree.
No source-worktree commit or push was performed.

The brief's substantive P3s are supported. “Still passes” needs qualification:
the earlier local-package missing-evidence run already returned nonzero **2**,
but misleadingly printed baseline agreement while its final counts differed.
It did not return complete success **0**. The previous replay resolved worker
HEAD, not the integration ref. This run resolves the latter explicitly.
Manager-supplied “REVIEWED”, “RECEIVED” and submission to Travis are provenance,
not acceptance independently established by this worker.
