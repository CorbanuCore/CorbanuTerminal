# PF-60-S03: acceptance decision for Travis and Fable

**Use the inspector only for estimates within a known session root: open P2
scope-zero renders “No recorded attempts in this day; collection coverage
unknown.” and “Known subtotal exact USD: 0” despite four priced attempts totaling
USD 0.00284 in other roots in the same store that day, inviting a false day-wide
zero conclusion; it is not fit for day-wide spend or complete-bill decisions.**

[The reproduced finding](../acct-scope-62/reader-findings.md) binds actual keys,
store read-backs and root reopens. The title and secondary text name root and
resolved descendants, so the entire page is not literally unqualified; the
primary empty-state and zero fail to attach that scope to their conclusions.
No scope-zero fix or accepted waiver exists. Developer-only activation is
authorized; shipping `/usage requests` remains held. S03 is not accepted or ready
for an unqualified human-test handoff.

Proven: saved evidence reconciles 9 priced views (not 9 independent attempts),
4 unknown-cost views and 2 zero-recorded views. The [coverage inventory](../acct-controls-72/monetary-coverage.json)
maps 147 named checks to 159 failing diagnostic mutations. Integrity controls
bind 24 JSON pages and 45 viewports; read-back controls distinguish saved IDs
and unchanged store rows. Qualifying mode stops at digest failures before most
monetary assertions; only diagnostic mode separates them. These are saved-corpus
checks, not independent functional acceptance or measured provider bills.

The [round-76 gates](../acct-fitness-76/test-results.json) report fresh default,
feature and TUI counts plus a separate alone-feature replay. **Historical gate
failure remains open as an unexplained intermittent timeout:** in round 66,
`codex-core::all suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`
timed out twice in the feature lane, at 60.013s and 60.012s (126/127 passed,
1 timed out, exit 100). Default passed in 58.790s. Round 69 passed alone in
31.743s and in the full feature lane in 31.335s; round 75 passed in-lane in
31.283s. [Attribution](../acct-bindings-69/timeout-attribution.md) supports load
sensitivity but identifies no contended resource and excludes no intermittent
defect. No causal fix, timeout relaxation or exemption occurred. Subsequent
passes do not turn round 66 green; current replay details are in the linked receipt.

Other limits remain: absent settlement evidence; unqualified inherited callers;
whole-store scan budgets that can refuse small scopes; partial/expired ranges
without complete totals; incomplete independent package, platform/profile and
live-repository proof. [Completed engineering preparation](../acct-fitness-76/engineering.md)
now supplies the sprint reconciliation, neutral designer input, platform/profile
matrix, exact evidence references and a local read-only developer package with
hashed binaries. Preparation is not execution or acceptance.
The [regenerated inventory](../acct-controls-72/scope.json) and
[change record](../acct-fitness-76/inventory-refresh.json) disclose the three
round-75 edits that invalidated round-72 digests.

Recheck the numerical claims and linked gate/inventory receipts from the repository
root with Python and Git (no build, credentials, network or profile required):

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inventory-79/verify_acceptance.py
```

The command prints agreement, disagreement or unavailable evidence for each item.
It uses committed-evidence membership and current checkout contents; ignored local
package binaries cannot establish clean-checkout package availability. Exit status
is three when retained results differ from the documented baseline (or that
baseline is malformed). It is two when the baseline matches but evidence is
unavailable, or zero for complete agreement. Optional `--local-package` checks
run after the retained comparison, but their combined results have no documented
baseline: this mode always prints `BASELINE REFUSED` and returns three, whether
local files agree, disagree or are absent. Its counts are diagnostic only and
must not be cited as agreement with the retained baseline.
Read exit codes in their command context; an evidence status inside a diagnostic
tuple is not necessarily the final process exit. The following table also covers
the historical test exit shown below.

```text
Code | Context and meaning | Reader action
0 | Verifier: all retained evidence agrees, nothing unavailable, baseline matches. Replay helper: exact-output and control checks passed (its verifier may still exit 2). Test/build: success. | Check which command returned it; none establishes functional acceptance.
2 | Verifier: documented baseline matches, but named evidence is unavailable. | Read UNAVAILABLE items and preserve their qualification limits; do not claim complete evidence.
3 | Verifier: baseline drift/malformed baseline, or --local-package has no documented combined baseline. | Inspect DISAGREE/UNAVAILABLE and BASELINE DRIFT/REFUSED; repair evidence or review intentional contract changes before refreshing inventories/reference together. Local mode is diagnostic only.
1 | Evidence status in actual counts/exit: at least one DISAGREE; the baseline guard turns this into final verifier exit 3. Replay helper: a raised check or execution error. An uncaught verifier error can also exit 1 without a final RESULT. | Follow the disagreement or traceback; restore/reconcile the evidence or fix the execution prerequisite and rerun. Never treat it as a match.
100 | Historical nextest lane: unsuccessful test run, here the retained timeout. | Keep the failure open, inspect its raw log and investigate; later passing runs do not erase it.
```

Invalid command-line usage can return an argument-parser error with the same code
as incomplete agreement. A traceback, usage error, or missing final `RESULT`
is an execution failure, not the documented incomplete baseline.

This baseline reproduces only while any change to the inventoried evidence set
is accompanied by a refresh of the affected inventories and membership bindings.
This includes edits, removals and additions, including a new file matching a
membership glob. The retained evidence, verifier contract and historical Git
objects must also remain available and consistent.
The expected retained-evidence baseline is:

```text
RESULT agreement=20 disagreement=0 unavailable=3 exit=2
```

The only expected unavailable items are `committed package codex`,
`committed package codex-code-mode-host` and `committed package rmcp_test_server`,
all under the [package directory](../acct-fitness-76/.gitignore). These are deliberately ignored local debug
build products; [the exclusion record](../acct-baseline-81/package-exclusion.md)
gives their sizes and the applicable repository rule. Any different counts or
unavailable item, or any disagreement, now prints `BASELINE DRIFT` and returns
three. A matching incomplete run prints `BASELINE MATCH` and returns two.
The fenced result states the expected evidence exit before the baseline check;
the last printed result carries the actual process exit. Legitimate changes must
update this documented baseline.
This expected baseline describes retained evidence, not complete qualification.

**Recognizing an unrefreshed later edit:** in a disposable local checkout of the
proposed QA files, append one newline to this acceptance document without
refreshing its inventory. The claims are unchanged, but the inventoried bytes
differ. [The recorded simulation](../acct-guard-91/simulation-final/simulation.json)
runs the verifier normally and with Python optimization; both return three with
empty stderr and this verbatim stdout. Restoring the file restores the baseline.
This is a deliberately failing evidence-integrity control, not a product failure
or a new acceptance baseline.

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
DISAGREE acct-acceptance-75 inventory: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-acceptance-75/acceptance.md bytes/lines/SHA-256 differ
AGREE inventory classifications and correction: 15 added paths and 3 modified paths against original base; round-72 changed entries=3
AGREE package build evidence: build receipt and raw log agree; launch/functional acceptance not claimed
UNAVAILABLE committed package codex: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex is not tracked; unavailable from a clean checkout; cannot re-derive bytes=607785336, mode=0555 or SHA-256
UNAVAILABLE committed package codex-code-mode-host: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/codex-code-mode-host is not tracked; unavailable from a clean checkout; cannot re-derive bytes=93937576, mode=0555 or SHA-256
UNAVAILABLE committed package rmcp_test_server: qa/portfolio/agent-cost-accounting/pf-60-s03/acct-fitness-76/package/rmcp_test_server is not tracked; unavailable from a clean checkout; cannot re-derive bytes=11508704, mode=0555 or SHA-256
AGREE acceptance numerical coverage: no unmatched digit-form quantities outside matched claim spans, fenced code and link targets; known round/sprint/severity identifiers excluded; worded quantities and excluded regions not audited
BASELINE DRIFT: expected counts/exit=(20, 0, 3, 2), unavailable=['committed package codex', 'committed package codex-code-mode-host', 'committed package rmcp_test_server']; actual counts/exit=(19, 1, 3, 1), unavailable=['committed package codex', 'committed package codex-code-mode-host', 'committed package rmcp_test_server']
RESULT agreement=19 disagreement=1 unavailable=3 exit=3
```

**What Fable would be accepting:** this bounded revision makes the saved evidence
reproducible under the stated conditions. It does not accept S03, clear the
scope-zero finding or qualify a shipped inspector. There is no calendar expiry,
but there is a condition on how long this claim stays true: keep the retained
files, their refreshed inventories, the verifier contract and the required Git
history together. An unrefreshed edit, removal or addition to the inventoried
set (including a new file matching a membership glob), missing history, changed
claim or changed verifier contract can stop the baseline from matching.
Refreshing an inventory alone does not justify a changed claim or remove a
qualification gap.

The [exact-output replay](../acct-selfcheck-84/check_current_tip.py) is a separate
manual check a reader may run; the acceptance verification command above does
not invoke it, and it is not an automatic acceptance or CI gate. Run it explicitly
from the repository root after the candidate has landed on the integration ref:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-selfcheck-84/check_current_tip.py
```

When invoked, it uses explicit raises that remain active under Python optimization
and compares with
[retained expected output](../acct-reference-88/expected.stdout.txt), bound by a
[content digest](../acct-reference-88/reference.json) in the same candidate. It
reads those committed bytes from the resolved integration tip; it has no old
commit pin and never regenerates its expectation during verification. Landing
this revision or advancing the tip with unrelated work does not expire the
reference. An intentional change to verifier output, including refreshed
inventory totals, requires a reviewed update to the expected output and its
digest in the same change. Until that happens, running the exact-output replay
rejects the change even if the documented counts still match. Review must check the
changed evidence and claims; replacing a reference is not proof they are true.

The numerical coverage guard checks for unmatched digit-form quantities outside
fenced code and link targets, after removing matched claim spans and known
round/sprint/severity identifiers. The dedicated baseline comparison checks the
fenced expected result and the exact unavailable-item names above. The numerical
coverage guard does not audit other excluded regions, quantities written as
words, or the truth of nonnumerical statements. Specific evidence checks match
only the literal worded attempt count, timeout count and inventory-edit count.
[The residual register](../acct-selfcheck-84/worded-quantity-residual.md) identifies
the unchecked sentence classes, the current sentences, and the cost of closing
this gap. It is a manual disclosure, not an automated natural-language audit.
Round identifiers and severity labels are references, not measured quantities.
The [inventory correction](../acct-inventory-79/inventory-correction.json) supersedes
the refreshed round-75 new-file labels and totals in the earlier change record.

**Owner decisions only:**

- Fable may accept or reject this bounded QA revision for integration under the
  existing allocation. This does not close S03 or authorize shipping.
- Travis may explicitly accept a named limited-test boundary or applicability
  exclusion when presented with its exact cases and gaps; none is requested or
  presumed by this revision. Full named acceptance remains due after qualification.
- New settlement/persistence scope, a change to developer-only activation, or paid
  live-provider use requires its specific product/spend authorization before that
  work. This revision requests none of those expansions.

Routine preparation, assigning independent actors, provisioning the enclosure,
and running already-authorized synthetic QA are engineering execution under
Fable's existing authority, not permission questions for Travis. Remaining
execution and the scope boundaries that prevent this worker from completing it
are explicit in the engineering record; they are not owner approval requests.
No new dollar-spend estimate is asserted without rates or capacity inputs.

Routine evidence correction under active PF-60 / in-progress PF-60-S03; exact
product heading **Product measurement**, excerpt “No commercial performance
numbers have been supplied.” Product/Rust code is unchanged. This documentation
revision adds no user behavior and is not a functional handoff; the independent
functional gate remains due for S03. Canonical sprint/plan updates are outside
this worker's writable QA scope. No release or push is authorized.
