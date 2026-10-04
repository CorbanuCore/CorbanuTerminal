# RETURN

Allocation `acct-baseline-81`; model `gpt-6-astra`; effort `high`.
Claim `7e96d83d-f3da-4f69-91a1-2fd98c09a4de`.
Allocation digest `cbb319a9228f36858dceecd535da8d274c265a81a2c1d3beba017660e351cc68`.
Brief SHA-256 matched `1b4dc8ca97094dbe7376fed86755cd1cfc365e760d110733b1b37440bf13dbe6`.
Starting HEAD matched `8fdc6ac60e5d2f42fb34e303083c68759b26a551`.

Routine evidence maintenance under PF-60 / PF-60-S03. Product heading
**Product measurement**, excerpt “No commercial performance numbers have been
supplied.” Product/Rust code is unchanged. Independent functional, true-TUI,
live-repository and benchmark qualification are not applicable to this internal
evidence revision; the final-package independent functional gate remains due for
S03. No new approval, human acceptance, shipping readiness or release is claimed.

## Five findings

1. The acceptance paragraph now states the expected baseline explicitly:
   `agreement=20 disagreement=0 unavailable=3 exit=2`. The only expected
   unavailable items are `committed package codex`,
   `committed package codex-code-mode-host` and
   `committed package rmcp_test_server`. A different count or item is a deviation,
   even with exit two. Legitimate changes require updating the document.
2. These are deliberately excluded local debug build products under
   `acct-fitness-76/.gitignore`'s `/package/` rule. Manifest sizes are
   **607,785,336**, **93,937,576** and **11,508,704 bytes**, respectively:
   **713,231,616 bytes / 680.191 MiB** total. The record supports build-output
   exclusion and storage cost, not a licence prohibition or technical inability
   to commit every file. [The exclusion record](package-exclusion.md) distinguishes
   retained manifest claims from missing package bytes.
3. `inventory_kinds()` now counts the kinds derived against the original Git
   tree: **15 added paths / 3 modified paths**. The refreshed round-75 inventory
   contains **48,919 new bytes / 653 new text lines**, **16,713 modified bytes /
   230 modified text lines**, **65,632 bytes / 883 text lines** combined. These
   are full-file sizes. The previous correction totals/hash are preserved.
   A disposable membership control reported **14 added / 2 modified** after
   removing one of each, proving the printed counts are derived.
4. The coverage claim now says it detects unmatched digit-form quantities
   outside matched spans, fenced code and link targets, excluding known
   round/sprint/severity identifiers. It does not audit worded quantities,
   excluded regions (including the documented expected baseline), or the truth
   of nonnumerical statements. Specific checks still cover the three previously
   matched worded counts.
5. The original review's fifth finding was that `check_verifier.py` could not
   run from an unchanged committed tree and would overwrite stored controls.
   It now uses `--allow-empty`, a fresh `target/controls-*` directory per run,
   and writes every output/receipt there. [Replay checks](replay-checks.json)
   show two successful replays from clean committed proposed snapshot
   `0809790fd80f7bbc85a4a4826ac5fb78e1ed0c0c`: **5/5 controls each**, distinct
   output directories, clean tracked state before/after, all existing tracked
   output hashes unchanged. Raw attempts are retained in `replay-1/` and
   `replay-2/`. Wrong-claim/corrupt-viewport returned exit one; missing-log
   returned **18 agreements / 0 disagreements / 5 unavailable, exit two**.
   The latter demonstrates why the documented baseline matters.

These harness replays are worker-run evidence-tool checks, not independent
functional acceptance. Their proposed-snapshot commits exist only in disposable
clones; they are distinct from the exact integration checkout below.

## Exact clean integration run

`clean_integration.py` made a full local clone and detached checkout of
**ff3f74f6ae66efbf3dfcc4f6291af2ce97f0f6cc**, the receiving commit recorded by the
manager. It copied no working-tree changes, ignored package or build state.
No network, credentials or package launch was used. Git status was empty before
and after, and the package directory was absent. [The receipt](clean-integration.json)
records the checkout, command, exit and output digests.

Command, from that clone's repository root:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inventory-79/verify_acceptance.py
```

Exit **2**, stderr empty. Complete verbatim stdout follows; the old coverage
wording is preserved because this is the unmodified integration commit:

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
AGREE acct-acceptance-75 inventory: entries=18; new_files=15; new_bytes=47679; new_text_lines=632; modified_files=3; modified_bytes=16713; modified_text_lines=230; current_bytes=64392; current_text_lines=862; full-file sizes, not diff lines
AGREE inventory classifications and correction: 15 added paths and 3 modified paths against original base; round-72 changed entries=3
AGREE package build evidence: build receipt and raw log agree; launch/functional acceptance not claimed
UNAVAILABLE committed package codex: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex is not tracked; unavailable from a clean checkout; cannot re-derive bytes=607785336, mode=0555 or SHA-256
UNAVAILABLE committed package codex-code-mode-host: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex-code-mode-host is not tracked; unavailable from a clean checkout; cannot re-derive bytes=93937576, mode=0555 or SHA-256
UNAVAILABLE committed package rmcp_test_server: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/rmcp_test_server is not tracked; unavailable from a clean checkout; cannot re-derive bytes=11508704, mode=0555 or SHA-256
AGREE acceptance numerical coverage: all numerical quantities in acceptance.md mapped; round/sprint/severity identifiers excluded
RESULT agreement=20 disagreement=0 unavailable=3 exit=2
```

This proves the received acceptance command re-derives its retained evidence on
an actual clean integration checkout with exactly the expected package gaps.
It does not prove package-byte availability, rebuild identity, independent
functional acceptance or complete qualification. The revised proposed-snapshot
command separately reproduces the same baseline with the narrower coverage
message and updated inventory totals, in both replay records.

## Required fresh gates

Prerequisites built first from `codex-rs`, using shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`.
Only guarded `just test` was used, after reading test-isolation guidance.

| Lane | Passed/run | Skipped | Failed / timed out / flaky / leaky | Exit |
| --- | ---: | ---: | --- | ---: |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 / 0 / 0 / 0 | 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 / 0 / 0 / 0 | 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 / 0 / 0 / 0 | 0 |

**342/342 executions**, overlapping filters; failure names: **none**.
The historically intermittent auxiliary-scope test passed at **31.511s** default
and **33.679s** feature; the old round-66 failures remain unresolved history.
[Test results](test-results.json) and `gates-01/` retain commands, environment,
raw compressed logs, SHA-256 and exact counts. The isolation banner was present;
no native credential prompt or live-profile access was observed.
Python syntax and whitespace checks passed.

## Changed lines and brief precision

Six existing files: **+81/-25**. Acceptance **+21/-0**, its inventory **+7/-7**,
round-79 RETURN **+15/-3**, harness **+10/-6**, inventory correction **+19/-5**,
verifier **+9/-4**. [The new scope receipt](scope.json) records exact changed-file
sizes/hashes and separates new scripts, documentation and raw evidence from
these diff counts, excluding itself to avoid self-reference.
Round-79's self-inventory and raw outputs remain historical snapshots.

All changes and disposable clones are within the assigned QA subtree. No
workspace-wide formatter, source-worktree commit or push was run.

The five review findings are supported; the fifth was recovered from the original
review log rather than guessed. The brief's “cannot be committed” is too strong:
the demonstrated reason is deliberate exclusion of local build products, not a
proven blanket Git or licence prohibition. “Twenty quoted numbers” is imprecise:
twenty is the count of agreement items, several of which reconcile multiple
quantities; the three unavailable items are binary files/properties, not three
numbers. Reviewed/received status is manager-supplied provenance; this worker did
not perform an independent review or grant approval. The requested clean run
succeeded at its documented incomplete baseline; it did not reveal additional
evidence loss.
