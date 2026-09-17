# RETURN

Allocation `acct-inventory-92`; model `gpt-6-astra`; effort `high`.
Claim `7646a61f-2dc2-4079-b1be-59ceb49a796a`.
Allocation digest `e4bff57dd34c64896a601dbda5348f2e880a3d2719f9c7710a206844a1432f89`.
Brief SHA-256 verified before work:
`2af13949a001f5f14c89552db9db93f13a407de66ab0eb8cb6905c54188a5e16`.
Assigned and observed source base:
`05b8e6e1aea5a55d7a58f3af3c21621a3fa3fb2f`.

## Inventory and artifact corrections

[Round-91 scope](../acct-guard-91/scope.json) now has 72 entries, down from 103:
exactly the historical membership intersected with the assigned committed tree.
Every retained path is committed, and its bytes, lines and SHA-256 match the
revised working file. Historical base and prior inventory digest remain explicit.

The 31 removed entries were duplicate intermediate outputs under the replay's
`target/`, beside disposable local clones. Root `.gitignore` line 104 explicitly
ignores `qa/portfolio/agent-cost-accounting/pf-60-s03/*/target/`. The clones and
their Git metadata are scratch, not an omitted delivery: scripts and raw
stdout/stderr/receipts are retained in `replay-results/`, `simulation-initial/`
and `simulation-final/`. Readers can rerun the scripts against retained history
without the original temporary directories. Absolute paths and synthetic commits
in those run receipts describe execution provenance; they are not promises that
scratch checkouts or their synthetic Git objects are in the repository.
The explanation is also in the [round-91 record](../acct-guard-91/RETURN.md).

All ten historical simulation stderr artifacts are now exactly zero bytes:
five cases in each of `simulation-initial/` and `simulation-final/`.
Before correction, each contained only one newline; each original local raw
capture still existed and was zero bytes, matching `stderr_empty: true`.
All ten retained stdout files also matched their original captures and receipt
hashes. This corrects retention, not the recorded execution. The previous
newline artifacts remain recoverable from the assigned base. The empty-file
SHA-256 is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

## Exit meanings and recheck

[Acceptance document](../acct-acceptance-75/acceptance.md) now gives meanings and
reader actions for codes 0, 2, 3 and 1, plus historical nextest code 100.
It distinguishes evidence status 1 inside the drift tuple from final verifier
exit 3, helper exit 0 from nested incomplete verifier exit 2, and argument-parser
or uncaught errors from a complete final RESULT.

The acceptance inventory, correction history, exact-output reference and digest
were refreshed together. Only the inventory totals line changed in expected
stdout. New reference SHA-256:
`25450ea3837dfc7624189eb89f7067fda71b76dbec21e2853122875f03dd1fce`.

**I found no additional unverifiable-claim defect in the acceptance-package
recheck beyond the issues in the brief.** The [repeatable audit](recheck.py) and
[results](recheck.json) cover all 15 direct acceptance links; inventories with
55, 18 and 72 entries; all historical and fresh simulation streams; outer replay
receipts and 32 nested replay streams; gate log hashes/counts; and the current
reference/baseline. The acceptance verifier also rechecks its numerical claims,
mutation coverage, capture bindings and historical gate receipts.
The qualitative statement above is a manual reading by the round-92
`acct-inventory-92` revise worker (`gpt-6-astra`, high; claim
`7646a61f-2dc2-4079-b1be-59ceb49a796a`), not a computed audit result or a human
sign-off. That reading included the linked engineering, exclusion and residual
records. Round 95 removes the script's literal empty-defect field and the
corresponding JSON member; neither scanned prose nor derived that verdict.
No named human reader is evidenced in this record.
The three historical package binaries remain explicitly unavailable; worded
claims and authorization/qualification limitations remain disclosed, not waived.
This is a bounded retained-evidence audit, not a proof of arbitrary prose or
independent functional acceptance.

Fresh [simulation](simulation/simulation.json) returns 2/2 for normal/optimized
baseline, 3/3 for the unrefreshed edit, and 2 after restoration, all with empty
stderr. The failure stdout still exactly matches the acceptance document's
embedded historical block.
Fresh [replay](replay-results/replay-checks.json) passes a simulated landed
candidate normally and under `-O` (helper exits 0/0; nested acceptance exits 2/2).
A deliberately wrong reference with matching digest fails explicitly in both
modes (helper exits 1/1). Each of the four helper runs reports all 13 verifier
controls passed. These are local simulated landings; source refs were not moved.
Raw outer and nested streams are retained alongside the receipts.
New absolute scratch paths in these receipts have the same provenance-only
meaning described above.

## Required gates

Prerequisites built first with locked offline Cargo build for codex-cli,
codex-rmcp-client and codex-code-mode-host. All tests used guarded `just test`
from `codex-rs`, shared dedicated target
`qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-33/feature/target`,
and `NEXTEST_TEST_THREADS=4`. [Raw logs and receipts](gates-01/) are retained;
the audit independently matches their summaries and SHA-256 digests.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Core accounting default | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| Core accounting developer-accounting | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| TUI usage | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

Total: 342 passing executions across overlapping filters, not unique tests.
The historically intermittent test passed in 31.597s default and 32.160s feature;
its old timeout remains historical evidence. Isolation banners were present.
No native credential prompt or live-profile access was observed. No raw Cargo
test/nextest, workspace formatter, product-code edit, push or source commit.

## Changed lines and launch observations

Existing-file diff against assigned base: **+108/-250 across 17 files**.
Per-file counts and hashes, including new scripts and retained evidence, are in
[scope.json](scope.json); that inventory excludes itself to avoid self-hashing.
New files are under `acct-inventory-92/`; all changes are within the assigned
PF-60-S03 QA subtree. New compressed logs have byte counts and hashes, not text
line counts.

ACK was sent before START; no tools ran before START. Model, effort, worktree,
base commit and brief hash matched dispatch, and the worktree initially was clean.
Current approval policy is `never`, with unrestricted filesystem access.
No startup mismatch against the frozen assignment was observed. This fresh
session has no prior-round launch prompt or approval-policy record to compare;
I cannot certify those were unchanged. No manager approval is inferred.

One statement in the brief is inaccurate: “wired into the acceptance
verification” describes the exact-output replay as automatic. The retained
round-91 code and acceptance text instead implement its explicitly selected
manual-check option: `verify_acceptance.py` does not call
`check_current_tip.py`. The manual replay must be invoked separately.
The claimed 31 scratch entries, ten newline stderr artifacts, and missing
exit-one explanation were confirmed. No other brief error was found.

Routine internal evidence correction under active PF-60 / in-progress PF-60-S03.
Exact product heading **Product measurement**, subsection **Measurement targets**,
excerpt “No commercial performance numbers have been supplied.”
No product behavior changed. Independent functional design/execution, true-TUI,
live-repository and benchmarks are N/A to this internal correction; final-package
functional qualification remains due for S03. Manager acceptance of this N/A and
canonical plan/sprint records remain with the integrator. No S03 acceptance,
named-human sign-off, functional handoff or release qualification is claimed.
