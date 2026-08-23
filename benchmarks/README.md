# Corbanu Terminal competitive benchmark tracker

This file is the sole authority for the Corbanu/Hermes/Kilo Code benchmark
method, cadence, and public ledger.

## Cadence

Complete one qualifying three-way benchmark at least once every three Corbanu
releases. Update the ledger for every release so the count cannot be inferred or
reset informally.

No qualifying three-way baseline has been recorded under this policy. The next
release is therefore the bootstrap run. Development may continue while it is
pending; the release may not be published until it passes.

## Qualifying run

The release owner freezes the following before any harness starts:

- TensorCash or Isometric Game origin and base commit;
- one development task, prompt, and acceptance rubric;
- Corbanu Terminal, Hermes, and Kilo Code versions;
- comparable model/provider access where each harness supports it;
- permissions and available tools;
- Corbanu regression thresholds;
- a common safety or spend cutoff.

“Unconstrained” means no harness-specific turn, tool, or workflow cap. A common
predeclared safety or spend cutoff is allowed.

Each harness starts from its own disposable worktree of the same base commit.
A lane is auditable when the harness starts correctly, receives the frozen task,
and preserves sufficient output, diff, test, timing, and failure evidence.

## Verdicts

| Verdict | Definition | Release effect |
| --- | --- | --- |
| **Pass** | All three lanes are auditable and Corbanu meets the frozen task rubric without crossing a declared regression threshold or exposing a P0 security failure. | Benchmark gate passes. |
| **Fail** | Corbanu fails the task rubric, crosses a declared threshold, or exposes a P0 security failure. | Release blocked. |
| **Incomplete** | Any lane cannot be audited because its harness, credentials, provider route, task input, or evidence is unavailable. | Release blocked until rerun. |

Hermes or Kilo performing poorly on a functioning, auditable lane is a valid
competitive result; it does not make the run incomplete. A due failed or
incomplete benchmark has no release waiver.

## Bootstrap procedure

For the next release, the release owner must:

1. create `qa/release/<version>/benchmarks/`;
2. replace every pending field in the bootstrap ledger row;
3. freeze the task, base commit, rubric, thresholds, and common cutoff;
4. run and preserve all three lanes;
5. enter the verdict and evidence links below; and
6. set “releases since qualifying run” to zero only after a pass.

The current pending row blocks publication, not implementation.

## Cadence ledger

| Release | Releases since qualifying run | Required | Owner | Test repo/base | Frozen task and thresholds | Corbanu | Hermes | Kilo | Verdict | Evidence |
| --- | ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Next release after policy adoption | baseline pending | yes | Release owner named in release record | pending | pending | pending | pending | pending | **Pending—publication blocked** | pending |

## Evidence package

Store the following under `qa/release/<version>/benchmarks/`:

- candidate and harness versions/commits;
- repository origin, base commit, and disposable worktrees;
- frozen prompt, rubric, and Corbanu regression thresholds;
- model, provider, reasoning, permissions, and tools;
- common cutoff;
- timestamps, raw transcripts, final diffs, and test results;
- elapsed time and cost/token use when observable;
- lane verdicts and the final benchmark verdict.

Every public benchmark report must link to its cadence-ledger row here.
