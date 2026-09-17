# PF-60-S03 adversarial prose read — acct-final-100

Reader: the acct-final-100 revise worker, gpt-6-astra/high, claim
94b32c0c-2b07-4bc0-9157-df2e5d9fe9e8, September 17, 2026.
Source record: 967425cc72ea28b5f59ecc38a56419b7db6885e8, with the two
explicit round-100 corrections described in RETURN.md. This is a manual
judgement, not a computed clean bill, independent acceptance, or human sign-off.

I read across the retained S03 prose: implementation, bounds and activation
RETURNs; the full-suite and activation manager notes; qualification and reader
reports, rendered-output transcription, preflights and reproduction notes;
inherited/residual/next-step records; acceptance and engineering preparation;
inventory/replay rounds; and the current and frozen operator notes. I followed
the material claims into their receipts, including capture/read-back limits,
timeout attribution, package exclusion and final operator execution. Historical
corrections and supersession matter: an old result is not automatically a claim
about today's candidate. This was a prose/evidence review, not a rerun of the
historical campaigns or a fresh validation of every saved byte.

The three strongest candidates are below. The original sentences remain in their
historical files; this review records the disagreement without rewriting them.

## 1. A failure comparison becomes a categorical absence of regressions

**Sentence:** “Zero regressions attributable to this change.”

**Location:** [full-suite-attribution-20260916.md](../full-suite-attribution-20260916.md),
line 38. The following explanation also says the candidate was slower
“because” of concurrent work, that the failures are ones the code “cannot
reach,” and that a reviewer's scope analysis “covers” possible regressions
coinciding with already-failing cases.

**Evidence needed:** retained baseline/candidate failure identities and run
conditions, plus discriminating evidence for the candidate-only failures.
A controlled comparison or causal trace could support the load attribution.
Code review is useful supporting analysis, but cannot itself rule out runtime
regressions or changes hidden by existing failures.

**Evidence present:** the memo reports 92 matching failures, 47 candidate-only
timeouts and one candidate-only final failure that was flaky at baseline.
The candidate's [failure list](../impl-03-tui-full-failures.json) and
[JUnit](../impl-03-tui-full-final.junit.xml) retain the failed candidate run.
The memo reports different wall times and concurrent work, and describes an
independent review. It supplies no linked baseline JUnit or causal/load-control
receipt in this S03 record. Even accepting its comparison numbers and review
description, timeout classification and elapsed-time correlation do not establish
the categorical conclusion. The memo itself acknowledges that the comparison
cannot rule out a regression coinciding with a baseline failure.

**Verdict: not supported at the stated strength.** This is not a newly
demonstrated product regression. The defensible wording is that the comparison
found substantial inherited failure overlap and suggested load sensitivity;
candidate-only timeouts and masked regressions were not ruled out.

**Beyond wording, left for the manager:** establishing the original causal claim
would require locating/retaining its baseline and review evidence and, if still
needed for a current decision, allocating a discriminating execution or
investigation. I did not run a full suite, change tests/timeouts, or start that work.

## 2. Three denials become proof of a code-blind executor

**Sentence:** “The three source hosts were denied from inside the same runs, so
the executor was genuinely code-blind.”

**Location:** [no-activation-path-20260916.md](../no-activation-path-20260916.md),
lines 17–18.

**Evidence needed:** the exact package and launch identity, fresh-context
separation, enforced filesystem/tool/process/IPC/network and credential
boundaries, actual executor and child negative probes, positive package/PTY
controls, and independent evidence review under the repository's code-blind
contract. Blocking three source hosts is one possible supporting observation,
not the whole contract.

**Evidence present:** the note names candidate b4513f6cc, a sealed-guest root,
a reported root/child success and the three reported denials. It links no
executor/package/probe receipt establishing the remaining boundaries. Searching
the retained accounting/code-blind Markdown, JSON and text records for that
candidate, the root/child phrase and source-host claim located this narrative,
not a supporting qualification packet. The later
[engineering record](../acct-fitness-76/engineering.md) explicitly keeps
independent execution and probed enclosure qualification incomplete.

**Verdict: unverified; the stated inference is insufficient.** This does not
prove the historical executor had access, nor that no evidence exists elsewhere.
The honest local claim is that three source-host denials were reported; this
memo alone does not establish full code-blind qualification.

**Beyond wording, left for the manager:** locate the actual historical packet
and independent review, or separately allocate the missing qualification.
I did not provision an enclosure, launch an executor or reinterpret these
denials as a pass.

## 3. An activation blocker is described as an absent installer call

**Sentence:** “The store-side install path exists in `codex-state`
(`accounting_native.rs`, "explicitly installed accounting") but nothing in
`core` or `tui` calls it.”

**Location:** [no-activation-path-20260916.md](../no-activation-path-20260916.md),
lines 30–32. The subsequent “on any machine” / “exactly one reachable output”
claim is also broader than the reported fresh-profile observation.

**Evidence needed:** the historical call path and configuration reachability,
distinguishing an existing installer from whether the ordinary loader enables
the collector. A fresh absent-ledger result does not prove every possible
installed-profile result.

**Evidence present:** [round 20](../acct-activation-20-return.md), “Blockers
found before editing,” already explicitly corrects this claim:
`Sampling::start_request` calls `AccountingStore::open`; disabled collection
prevents reaching it. A read of the memo's own historical commit
`b4513f6cc699773a7ca922d08b67683f47586a96` confirms:
`codex-rs/core/src/accounting.rs:194` calls `AccountingStore::open`;
`state/src/runtime/accounting_store.rs:288–291` opens and invokes
`install_on_connection`; lines 652–661 contain its migration path.
This is historical source evidence, not a conclusion inferred from today's
developer-activation change. The later
[engineering reconciliation](../acct-fitness-76/engineering.md) also labels
the old activation note historical.

**Verdict: contradicted as an installer-call statement; activation was the
blocker.** The broader note is superseded by developer-only activation and must
not serve as current acceptance evidence. A historical correction/cross-reference
would suffice for this prose error. It does not justify another installer or any
runtime change. No such change was made.

These candidates do not negate the current operator note's narrower statement:
its replay establishes internal candidate agreement, with three historical
package binaries unavailable, not independent truth or acceptance. Nor do
current passing filters erase historical failures. The acceptance/engineering
records still explicitly leave scope-zero, independent functional qualification,
settlement, inherited collection and other named gates open. The brief's
“finished for everything inside its authority” can describe this bounded worker
lane; it cannot mean those S03 obligations are complete.
