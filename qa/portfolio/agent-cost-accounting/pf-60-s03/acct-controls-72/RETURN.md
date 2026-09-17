# RETURN

Allocation `acct-controls-72`; model `gpt-6-astra`; effort `high`.
Claim `97f6e5e1-9ead-41b3-b091-947cd5d536b3`.
Allocation digest `cee81fd4da547964f78a44156660f2c2aab7c5bde42fc4debc6437d0c018eb01`.
Brief SHA-256 verified before other work:
`7c955be6f054c2efac080fa3bfdef98ab83e36f93edb525e412239dc4af4be0d`.
Clean launch HEAD matched `17371d192e52de71ad7efde9088cb47b6ae3a049`.

**Semantic coverage repaired.** The default auditor still requires digest binding;
explicit `--semantics-only` evaluates monetary assertions without digest checks.
It reports diagnostic-only status and zero content bindings. Monetary failures
accumulate so overlapping narrow/general checks all run, then fail before any
success receipt. An independent explicit mutation plan must match all check IDs;
deleting a check fails that inventory assertion, and disabling one fails its
targeted mutation.

[Per-assertion map](monetary-coverage.json) names **147 check instances**, each
mapped to a reached, specifically failed mutation in the
[159-case plan](controls-04/plan.json.gz) and
[raw results](controls-04/semantic-results.json.gz).
Coverage includes every page's exact/displayed count and value, each component's
displayed and exact amount independently, rates, unknown/incomplete costs,
missing-price/no-usage states, refused amounts, narrow qualifiers, token
metrics, emitted arithmetic and native price bindings.
All 147 checks execute on each semantic mutant; its intended check ID must be
in the failure list. [The existing 28 removal controls](removal-controls-02/results.json.gz)
now require exact-count/display-count semantic failures, not generic rejection.

**Viewports bound.** All **45 decompressed viewport contents** and their exact
filename set are pinned, alongside the unchanged 24 JSON content hashes.
[Provenance](viewport-provenance.json) verifies original gzip bytes against the
assigned base before checking their content hashes.
[Integrity results](controls-04/integrity-results.json.gz) reject 24 JSON-only
and 45 viewport-only mutations for their respective digest reasons.
All 45 viewport mutants pass semantics-only, demonstrating the gap: changing
visible amounts, warnings or screen state could otherwise leave JSON unchanged.
Gzip headers are deliberately outside the content hash.
Earlier 75 subtotal-injection and 74 page-integrity controls also pass.

**Read-back separation checked.** [Script](readback_separation.py) and
[receipt](readback-01/results.json) prove four saved attempts per dataset,
disjoint attempt/request/thread identifiers (4/4/3 per dataset), and exact
round-62 before/during store equality. Four negative controls reject identity
overlap and changed store contents. This establishes separation of saved store
rows only. The byte-identical empty pages still do not bind which root or
population either session resolved, and read-back provenance is not independent
authentication. No distinct-page or distinct-capture claim is restored.

**Acceptance remains open.** Cite [acceptance-gap.md](acceptance-gap.md) from
PF-60-S03. It lists independent design, sealed independent execution and evidence
review, live repositories, platforms/profiles and named-human acceptance; for
each it states this lane's limitation, required allocation/authority and smallest
honest first slice. It also preserves receiving-tree gates, unresolved sprint
dispositions and the developer-only activation/shipping boundary. No approval,
functional acceptance or human-test readiness is claimed.

**Required guarded gates**, shared dedicated target, prerequisites first,
`NEXTEST_TEST_THREADS=4`, from `codex-rs`:

| Lane | Passed/run | Skipped | Failed / timed out / flaky / leaky |
| --- | ---: | ---: | --- |
| Core accounting default | 124/124 | 3545 | 0 / 0 / 0 / 0 |
| Core accounting developer-accounting | 127/127 | 3545 | 0 / 0 / 0 / 0 |
| TUI usage | 91/91 | 4078 | 0 / 0 / 0 / 0 |

**342/342 executions**, overlapping default/feature test sets, not 342 unique
tests. Failure names: **none**. [Summary](test-results.json) preserves exact
commands, environments, log hashes and timing; lossless logs are in `gates-01/`.
No native credential prompt or live-profile access was observed.
Sprint checker passes (115 current, 127 archived); whitespace check passes.
[Intermediate harness failures](intermediate-attempt.md) remain disclosed.

**Changed lines:** existing tracked files **+75/−39**; four new Python helpers
**455 lines**, plus the 47-line viewport digest contract, prose and generated
receipts. [Scope inventory](scope.json) enumerates new file sizes/lines separately,
including copied historical auditors and compressed raw evidence.
All changes are inside the assigned QA scope; Rust/product changes: zero.
No workspace formatter, commit or push.
[Reproduction](REPRODUCE.md).

**Brief precision:** no material finding was wrong. Literally, the prior auditor
already checked emissions/arithmetic before its digest loop, and some viewports
had requested-day checks. Neither protected rendered monetary mutation coverage
or bound viewport content; the reported defects were real.

Classification: routine internal evidence repair under active PF-60 and
in-progress PF-60-S03. Product heading **Product measurement**, excerpt
“No commercial performance numbers have been supplied.” This allocation does
not grant the integrator's acceptance or amend implementation scope.
