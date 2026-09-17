# RETURN

Allocation `acct-guard-91`; model `gpt-6-astra`; effort `high`.
Claim `287e3967-d143-4a52-b4b2-4a530c2c3181`.
Allocation digest `43c619a1431d3e976a4c3a299f82aac055e2c67114dd47753bd6bb1bd4d3f96d`.
Brief SHA-256 verified before work:
`af7ad752c222218e5a82a0a7b16e740b9c5266c5776de3c75868d643f25168cf`.
Source base `e9d1df54f5b99786151cc1794ee78ec526072f48`.

## Delivered

- [Replay helper](../acct-selfcheck-84/check_current_tip.py): every bare assert
  was replaced with a conditional call to `require`, which explicitly executes
  `raise AssertionError(detail)`. Output equality, tip stability, checkout
  cleanliness, exits, stderr, reference schema/hash and clone checks remain
  active under `python -O`.
- Chose the brief's **manual-check text correction** option. The
  [acceptance document](../acct-acceptance-75/acceptance.md) now explicitly says
  the acceptance command does not invoke the replay and it is not an automatic
  acceptance/CI gate. It gives the reader the command and says it checks the
  landed integration ref. This preserves the separate retained-checkout and
  integration-tip checks without adding recursive acceptance execution.
- Durability now covers any change to the inventoried evidence set: edits,
  removals and additions, including new files matching membership globs.
  Affected inventories and membership bindings must be refreshed.
- Appended one newline to the inventoried acceptance document in a disposable
  local checkout, leaving its inventory unchanged. Both normal Python and
  optimized Python returned three, with empty stderr and identical stdout.
  The complete verbatim output is beside the baseline in the acceptance
  document. It includes `DISAGREE acct-acceptance-75 inventory`,
  `BASELINE DRIFT`, and
  `RESULT agreement=19 disagreement=1 unavailable=3 exit=3`.
  Restoring the document restored the exact baseline. Raw initial and final
  runs are preserved in [simulation-initial](simulation-initial/simulation.json)
  and [simulation-final](simulation-final/simulation.json).
- Refreshed the affected inventory, correction record, expected output and
  content digest together. New expected-output SHA-256:
  `0448345a1e9db7a844139960ae585127d03597bf0820e22c4de705b24c930aa5`.
  Verification never refreshes that reference.

## Verification

[Receipt controls](receipt-controls.json) and
[optimized receipt controls](receipt-controls-optimized.json): each passed one
positive and nine independently injected negative cases.

[Full helper replay](replay-results/replay-checks.json): simulated landed
candidate passed normally and under `-O` (helper exit zero, acceptance exit two).
A deliberately incorrect expected output with an internally matching digest
failed both helper runs with exit one and the explicit
`AssertionError: current integration output differs from pinned replay`.
Each helper run also passed all thirteen existing verifier controls.
The source worktree's branches were not moved.

Prerequisites built first with `cargo build --locked --offline` for codex-cli,
codex-rmcp-client and codex-code-mode-host. All following tests used guarded
`just test` from `codex-rs`, the shared dedicated target
`qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-33/feature/target`,
and `NEXTEST_TEST_THREADS=4`. Raw compressed logs and receipts are in
[gates-01](gates-01); [test-results.json](test-results.json) aggregates them.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Core accounting default | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| Core accounting developer-accounting | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| TUI usage | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

Total: 342 passing executions; overlapping test sets, not 342 unique tests.
The historical timeout remains historical evidence; these passes do not erase it.
Isolation banners were present; no native credential prompt or live-profile
access was observed. No raw Cargo test/nextest or workspace formatter was used.

## Changed lines and scope

Existing-file diff against the assigned base:

| Path within PF-60-S03 | Added | Removed |
| --- | ---: | ---: |
| acct-acceptance-75/acceptance.md | 61 | 9 |
| acct-acceptance-75/scope.json | 7 | 7 |
| acct-inventory-79/inventory-correction.json | 19 | 5 |
| acct-reference-88/expected.stdout.txt | 1 | 1 |
| acct-reference-88/reference.json | 1 | 1 |
| acct-selfcheck-84/check_current_tip.py | 19 | 10 |

Existing files total: +108/-33. New scripts, receipts and raw evidence are under
`acct-guard-91/`; [scope.json](scope.json) lists every changed/new file's current
line count, byte count and SHA-256, excluding only itself. Compressed logs have
no text-line count. [final-check.json](final-check.json) records the final
baseline, verbatim-output, explicit-raise and scope checks.

Nothing in the brief was found factually wrong. Its manual-check option was
used; no automatic enforcement or manager approval is claimed.

Routine internal evidence correction under active PF-60 / in-progress PF-60-S03.
Product heading **Product measurement**, subsection **Measurement targets**,
excerpt “No commercial performance numbers have been supplied.”
Product/Rust behavior is unchanged. Independent functional design/execution,
true-TUI, live-repository and benchmark qualification are N/A to this internal
correction; the independent final-package functional gate remains due for S03.
The manager owns canonical plan/sprint records and acceptance of this N/A.
No human acceptance, S03 completion, release, source commit or push is claimed.
