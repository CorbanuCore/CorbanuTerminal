# Independent PF-58 functional loop — completed, not qualified

Both extra passes authorized by the user are complete (seven total reviews).
The [case report](report.html) retains all 26 independently designed cases.
The [raw evidence verdict](evidence-check.md) is **fail** for human readiness.

Fresh execution: 32/32 Linux synthetic regression checks (30 actual-key checks
and two support checks), both platform package-tool smokes, two bounded Mac
existing-profile menu launches, and empty/whitespace/masked-cancel/invalid-key
probes on both packages. Component passes are not complete-journey passes.

Accepted findings: Linux recovery warnings omit the affected provider (F12),
and custom duplicate-model chat status omits provider identity (F15). These
variants are marked failed. Other cases remain incomplete, not silently waived.
The “configured” label on an unvalidated key and environment recovery requiring
restart remain explicit expectation/ownership decisions. Native consent, live
auth/billing and full supported-provider coverage are not certified.

The review input was checkpointed at `76c1b6f4c`; a separate unmodified code-free
copy remains under `.codex-work/pf58-blind-20260910/evidence`. Following the final
review, the coordinator corrected classifications, scoped the configuration
preservation wording, added explicit platform applicability and hash-indexed raw
artifacts. Those bookkeeping corrections were not sent through an eighth review.
The original proposal and reviewer response are unedited. No product code was
changed during this loop; no native permissions were bypassed or credentials
replaced. The candidate has not been merged or approved for another human round.

Rerun instructions: `run-linux.sh` pins the remote synthetic integration/package
suite; `probe-inputs.py --candidate <binary> --evidence <new-directory>` runs the
new black-box input probes with an external-drive `TMPDIR`. Original-case
dispositions are in `results-mac.json` and `results-linux.json`. The reusable
handoff checker must remain non-green while these failed/incomplete cases exist.
