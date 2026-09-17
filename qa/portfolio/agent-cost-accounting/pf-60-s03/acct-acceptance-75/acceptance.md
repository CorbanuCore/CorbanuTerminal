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
is zero for full agreement, one for disagreement, or two for missing evidence.
The expected retained-evidence baseline is:

```text
RESULT agreement=20 disagreement=0 unavailable=3 exit=2
```

The only expected unavailable items are `committed package codex`,
`committed package codex-code-mode-host` and `committed package rmcp_test_server`,
all under the [package directory](../acct-fitness-76/.gitignore). These are deliberately ignored local debug
build products; [the exclusion record](../acct-baseline-81/package-exclusion.md)
gives their sizes and the applicable repository rule. Any different counts or
unavailable item, or any disagreement, deviates from this baseline even if the
exit status is still two. Legitimate changes must update this documented baseline.
This expected baseline describes retained evidence, not complete qualification.

The numerical coverage guard checks for unmatched digit-form quantities outside
fenced code and link targets, after removing matched claim spans and known
round/sprint/severity identifiers. It does not audit quantities written as words,
numbers inside those excluded regions (including the expected baseline above),
or the truth of nonnumerical statements. Specific evidence checks separately
cover the worded attempt count, timeout count and inventory-edit count.
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
