# Text Improvement Harness Score Packet

## Subject

`OPUS5-ANTHROPIC-PROMPT-CACHE-STABILITY-SPEC-20260726.md`

Harness project:
`pfterminal-opus5-cache-stability-20260726`

Harness database:
`.tih/harness.sqlite3`

Scoring method:

- full gate;
- GPT-5.6-sol, Claude Fable 5, and GLM-5.2;
- five independent runs per model;
- 15 runs per scored revision;
- unweighted arithmetic mean across all 15 scores;
- raw prompts, responses, reasoning, scores, document content, and hashes retained in
  the harness database.

## Results

| Revision | Document SHA-256 | GPT-5.6-sol | Fable 5 | GLM-5.2 | Overall |
|---|---|---:|---:|---:|---:|
| Initial | `63984a71d2ed82bffbf70b1cddb54e1a6f16c45799f6718e074f6be1883fdc58` | 95.60 | 91.40 | 91.00 | **92.67** |
| Critique-driven revision | `11a25acca0b6adc8634848d5ffa0dd1864e20f12771315a55b9beed18a6fdf23` | 95.60 | 91.60 | 89.40 | **92.20** |

The final full-gate score is **92.20/100**. The 0.47-point decrease from the
initial score is within the spread of the individual judgments; it came from GLM's
stronger penalty for length and repeated requirements, not from a newly identified
technical defect. GPT remained unchanged and Fable increased by 0.20.

### Initial runs

| Judge | Scores | Average | Range |
|---|---|---:|---:|
| GPT-5.6-sol | 95, 96, 95, 96, 96 | 95.60 | 95–96 |
| Claude Fable 5 | 91, 92, 92, 91, 91 | 91.40 | 91–92 |
| GLM-5.2 | 92, 89, 92, 93, 89 | 91.00 | 89–93 |

Run group:
`round-20260726-cache-stability-spec-initial`

### Critique-driven revision runs

| Judge | Scores | Average | Range |
|---|---|---:|---:|
| GPT-5.6-sol | 95, 95, 96, 96, 96 | 95.60 | 95–96 |
| Claude Fable 5 | 90, 93, 91, 91, 93 | 91.60 | 90–93 |
| GLM-5.2 | 93, 89, 88, 89, 88 | 89.40 | 88–93 |

Run group:
`round-20260726-cache-stability-spec-final`

## Changes driven by the initial gate

The revision:

- defined a single linearizable state machine and its compare/exchange transition
  semantics for concurrent successes and failures;
- required both parsed equality and production-serialized byte equality for the
  complete Anthropic tool array;
- defined the cache-stability lifetime within a turn and behavior at user-turn
  boundaries;
- added an explicit before/after state diagram;
- showed the complete denominator arithmetic behind the 87.91% attribution;
- documented rejected alternatives, including a single immutable edit gateway;
- added an implementation effort and change budget;
- required measurement of initial schema-token overhead on a supported provider
  without prompt caching;
- removed the self-referential harness requirement from the implementation
  definition of done;
- made the evidence command root relocatable.

## Final copy edit and score provenance

After reading the second gate's rationales, three mechanical defects were corrected:

1. a duplicated sentence in Section 5.1 was removed;
2. “deterministic” concurrent ordering was corrected to “linearizable”;
3. a multiline `rg` pattern in the evidence commands was repaired.

The delivered specification after those copy edits has SHA-256:

`4d1a84f58fa2f28273a8934e34bbd396e73e7b755ffceedb1a8885127779cf05`

Those edits do not change the architecture, requirements, evidence, task list, or
acceptance thresholds. No third paid scoring gate was run. Therefore, **92.20 is the
exact score of the critique-driven revision immediately before these three
mechanical corrections**, not a fabricated score attached to a different content
hash. Both revisions are retained immutably in the harness database for audit.

## Stable judge consensus

All three judge families agreed that the specification:

- identifies the causal boundary rather than special-casing the benchmark;
- ties the economic claim to request-level and settled-billing evidence;
- gives implementation-ready code ownership and state semantics;
- supplies deterministic, live-provider, and economic qualification gates;
- treats alternatives, compatibility, rollback, telemetry, and secrets explicitly.

The recurring deduction was editorial: Sections 7, 9, 10, and 14 intentionally
repeat some requirements as invariants, executable tasks, acceptance tests, and
release criteria. That increases review length and maintenance risk. It was retained
because this document is also the requested `[ ]` remediation worklist, but future
implementation updates should reference requirement IDs rather than copy the prose
again.

## Delivery validation

- `git diff --check`: passed.
- Evidence command block parsed by `bash -n`: passed.
- Evidence command block executed read-only from the repository: passed.
- Reproduced zero-cache calls: `118536/0` and `110177/0` input/cached tokens.
- Reproduced settled cache creation: PFTerminal `516656`, Claude Code `256500`.
- All four evidence artifact SHA-256 values matched the values embedded in the spec.
- Final specification contains 54 unchecked `[ ]` remediation items.
