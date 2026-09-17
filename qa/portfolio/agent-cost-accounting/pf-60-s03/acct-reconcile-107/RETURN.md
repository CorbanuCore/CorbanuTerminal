# RETURN

Allocation `acct-reconcile-107`; claim `028a5d99-bb57-4f22-8553-6e83ea8417f5`.
Worker: `gpt-6-astra`, effort `high`. ACK preceded START and tools.
Brief SHA-256 verified:
`791f9f6e3eb96da9f1b7a3c4c139584eba9854ff31023bf4f3d12f4ff952c4c2`.

**The final clean-checkout acceptance verification returned its documented
healthy exit 2 with 20 agreements, zero disagreements and three unavailable
package artifacts.** This is matching incomplete evidence, not complete
qualification or acceptance of S03.

Source/integration base: `6a7128ccc4e10c92dd583ca65b62e70d84483012`.
The refresh describes proposed QA changes on that base. The final full clean
local checkout was temporary snapshot commit
`78b7f9c92b62af63ed81bf6c13a1d984078c1f09`, with empty Git status before and
after both normal and optimized verification. [Receipt](final-check.json)
records its tree, exact commands, input hashes and comparison results. This
scratch commit is not an integration commit; no source-branch commit or push
was made. The source checkout remains at the assigned base with proposed edits.
This RETURN and the final run's output/receipt are post-run reporting artifacts;
the verifier, reference and evidence inputs have not changed since the clean run.

## Verdict on the two disagreements

Both were documentation drift, not contradictory monetary or functional
evidence. The [clean base run](before.stdout.txt) reproduces exactly two
disagreements and exit 3. Comparison of the acceptance document before the
disclosure revision (`a72f03c3cc895a28df55b26bf7be756ecfbc3d5a`) with the
assigned base shows only the review-history paragraph changed.

- The acceptance inventory retained earlier bytes/lines/SHA-256. Its historical
  entry was 17,913 bytes / 236 lines. The refreshed document, including the
  maintenance disclosure, is 18,490 bytes / 244 lines. The inventory's other
  seventeen entries, their membership/classifications and all numerical
  evidence remain unchanged.
- The digit guard flagged `103` in `round-103` and `5` in
  `claude-opus-5-plan`. These identify a revision and a reported model, not
  measured quantities. The guard now excludes the exact backtick-delimited
  model name and the known revision identifier. It still rejects adjacent
  quantities, a different numbered model, a longer revision number and an
  unmapped dollar amount; [five targeted controls](identifier-controls.json)
  record the results. The output/disclosure now names model identifiers among
  the exclusions. This does not verify the manager's history.

Scope-zero, saved-page arithmetic, mutation counts, capture hashes, historical
gate receipts and the open timeout continue to agree with the same retained
evidence. No dollar amount, coverage count, qualification gap or expected
unavailable identity was changed. The expected baseline remains
`RESULT agreement=20 disagreement=0 unavailable=3 exit=2`; changing it to
tolerate a disagreement was neither necessary nor done.

## Worked shelf-life maintenance

The [refresh record](refresh.json) preserves before/after totals, the sole
changed inventory entry, source commit and the exact reference-line delta.
The `before-*` files preserve the prior document, inventory, correction,
verifier and reference bytes without replacing prior round evidence.

A future maintainer should follow this order:

1. Record the actual source commit, clean state and failing stdout/stderr.
   Identify every disagreement against the underlying evidence before editing.
   Stop on any real claim/evidence mismatch; refreshing hashes cannot resolve it.
2. Classify legitimate documentation or verifier-contract changes explicitly.
   Here, only metadata exclusions changed in the verifier. Review-history
   assertions remain attributed to the manager.
3. Refresh current sizes/hashes/totals for the existing inventory membership,
   then refresh its correction digest. Preserve the previous state and explain
   what changed. The current inventory and correction now carry an explicit
   `reconcile_107_refresh` marker bound to the assigned source commit.
4. Reproduce the newline drift in a disposable checkout, normally and with
   Python optimization; restore the exact file afterwards. The
   [fresh simulation](simulation.json) binds command, checkout and input
   hashes. Both runs return 19 agreements, one disagreement, three unavailable,
   exit 3, with empty stderr. Their stdout exactly matches the refreshed
   transcript in [acceptance.md](../acct-acceptance-75/acceptance.md).
   The initial working-tree simulation remains separately preserved in
   [simulation-working-tree.json](simulation-working-tree.json) and
   `drift*.txt`; the disposable replay is in `drift-disposable*.txt`.
5. Run the healthy verifier, review its changed lines, and refresh the
   [exact stdout](../acct-reference-88/expected.stdout.txt) and
   [digest binding](../acct-reference-88/reference.json) together. Only the
   inventory totals and the explicit metadata-exclusion wording changed.
   The new reference SHA-256 is
   `c0c52beb096b3aeb4a54676e57356b3ac3b804f28ccdb42f49f07dfdc081eecc`.
6. Run adversarial controls and the required gates, then verify from a clean
   proposed snapshot against that frozen reference. Never regenerate the
   expectation inside the verifier. Preserve final status, streams and exit.

The one-shot helpers `refresh.py`, `simulate_disposable.py` and
`verify_clean.py` retain this execution procedure, pinned to this allocation
and source base. They are not a blanket authorization to refresh future failures.
The manual integration-tip replay is still a separate post-integration check;
this worker did not advance that ref or claim to execute that helper against
an unlanded candidate.

## Review provenance

The ephemeral launch-script paths were removed from
[review-provenance.json](../acct-disclose-105/review-provenance.json), and the
earlier [RETURN](../acct-disclose-105/RETURN.md) withdraws its claim of
independently confirmed runtime. No script content or contemporaneous hash had
been retained. The record now treats model, effort, count, separation and
receipt-blocking history entirely as manager-reported information. Old citations
remain in Git history, not as current inspectable evidence. No review execution,
independence, human approval or acceptance is inferred.

## Verification and exact output

Prerequisites built first, exit 0. All three gates then ran sequentially from
`codex-rs`, using guarded `just test`, shared dedicated target
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`, two build jobs
and debug assertions. [Receipts and lossless raw logs](gates-01/) bind commands,
source commit and hashes.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Core accounting default | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| Core accounting developer-accounting | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| TUI usage | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

All test lanes emitted the isolation banner. No native credential prompt or
live-profile read was observed; no failing gate was retried. The 342 passing
executions overlap. They do not erase the historical round-66 timeout.
All thirteen existing verifier controls passed, alongside the five identifier
controls. [Control output](controls.stdout.txt) and the final receipt record
the wrong-claim, corrupted-viewport, missing-log, baseline and local-package
refusal cases.

Final command, run from the full clean temporary checkout's repository root:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inventory-79/verify_acceptance.py
```

Verbatim [stdout](final.stdout.txt), with empty [stderr](final.stderr.txt);
process exit **2**:

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
AGREE acct-acceptance-75 inventory: entries=18; new_files=15; new_bytes=60554; new_text_lines=790; modified_files=3; modified_bytes=16713; modified_text_lines=230; current_bytes=77267; current_text_lines=1020; full-file sizes, not diff lines
AGREE inventory classifications and correction: 15 added paths and 3 modified paths against original base; round-72 changed entries=3
AGREE package build evidence: build receipt and raw log agree; launch/functional acceptance not claimed
UNAVAILABLE committed package codex: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex is not tracked; unavailable from a clean checkout; cannot re-derive bytes=607785336, mode=0555 or SHA-256
UNAVAILABLE committed package codex-code-mode-host: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex-code-mode-host is not tracked; unavailable from a clean checkout; cannot re-derive bytes=93937576, mode=0555 or SHA-256
UNAVAILABLE committed package rmcp_test_server: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/rmcp_test_server is not tracked; unavailable from a clean checkout; cannot re-derive bytes=11508704, mode=0555 or SHA-256
AGREE acceptance numerical coverage: no unmatched digit-form quantities outside matched claim spans, fenced code and link targets; known round/sprint/severity/model identifiers excluded; worded quantities and excluded regions not audited
BASELINE MATCH: documented counts, evidence exit and unavailable identities match
RESULT agreement=20 disagreement=0 unavailable=3 exit=2
```

Normal and optimized stdout both exactly match the retained reference. The
final ordinary-Python invocation ran last, after gates, controls and optimized
verification.

## Changed lines and brief precision

Existing files: **+58/-35 across eight files**, with exact hunks in
[changed-lines.patch](changed-lines.patch). New execution helpers, preserved
prior inputs and generated evidence are listed with line counts, sizes and
hashes in [scope.json](scope.json), excluding that manifest itself.
No path outside the assigned QA scope changed; final diff validation passed.

The brief's “first time it has been exercised” is not supported by the retained
history: round 103 already has an
[exit-3 before-refresh transcript](../acct-guarantee-103/acceptance-before-refresh.stdout.txt)
and an [exit-2 final transcript](../acct-guarantee-103/acceptance-final.stdout.txt).
This revision supplies an explicit worked maintenance example without relabeling
those earlier attempts. No other factual error in the requested work was found.

Classification: routine internal evidence maintenance. Used
`corbanu-terminal-development`; exact product heading **Measurement targets**,
excerpt “No commercial performance numbers have been supplied.” Active PF-60 /
in-progress PF-60-S03 remain unchanged. True-TUI, independent functional
execution, live-repository and benchmark qualification are N/A to this internal
record/verifier maintenance: no product workflow changed and no functional
handoff is claimed. Fable retains the integrator decision on that N/A; S03's
later functional gate, scope-zero finding and human acceptance remain open.
No credential access, Rust/product edits, workspace formatter, source-branch
commit, push, release or fabricated approval.
