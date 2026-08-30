# Corbanu Terminal benchmark and performance tracker

This file is the sole authority for the three-release benchmark cadence,
Corbanu/Hermes/Kilo Code competitive method, coding-performance matrix, and
public ledger.

## Repository-owned harnesses

Canonical benchmark source lives in this repository:

| Benchmark source | Location | Purpose |
| --- | --- | --- |
| Coding harness | `benchmarks/coding/` | Portable Corbanu, Hermes, Kilo Code, Codex, and Claude Code runner with unsolved task packets and independent verifiers |
| Website builder | `benchmarks/website-builder/` | Same-model Corbanu versus Claude Code site construction, browser verification, captures, and balanced blind judging |
| Exact-key scanner | `benchmarks/scan_exact_keys.py` | Checks source and run artifacts for literal credential leakage without printing secret values |

Local sibling workspaces and historical run directories may preserve evidence,
but they are not canonical harness source. Do not import credentials, virtual
environments, generated workspaces, caches, or solved/reference candidates into
this repository. Version-specific evidence still belongs under
`qa/release/<version>/benchmarks/`.

## Cadence

Complete one qualifying benchmark cycle at least once every three Corbanu
releases. A qualifying cycle contains both:

1. the three-way Corbanu/Hermes/Kilo Code live-repository benchmark; and
2. the full coding performance matrix across the frozen relevant model set.

Update the ledger for every release so the count cannot be inferred or reset
informally. Neither component may reset the count alone.

No complete qualifying cycle has been recorded under this policy. The next
release is therefore the bootstrap cycle. Development may continue while it is
pending; the release may not be published until both components pass.

## Competitive live-repository component

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

## Coding performance component

Every due cycle must run Corbanu Terminal against every task in
`benchmarks/coding/configs/all-tasks.example.json`, including QueueCraft, for
every model/provider route in the frozen relevant model set. Ad hoc campaigns
may use subsets; the due performance matrix may not. Removing or excluding a
checked-in task requires a product decision before the release cycle starts.

Before execution, the release owner records a machine-readable model-set
manifest. The relevant model set includes every production model/provider route
that the release presents as default, recommended, or suitable for coding or
general agent work, plus every such route materially changed since the previous
qualifying cycle. Any exclusion from the wider production catalog requires a
written rationale and product-authority approval in the release record.

For every task/model pair, preserve:

- correctness and verifier outcome;
- end-to-end wall-clock runtime and timeout state; and
- actual provider spend, or a reproducible calculated spend using recorded
  token usage and a frozen price source.

Unknown spend, missing runtime, a missing task/model pair, failed correctness,
or an unauditable route makes the performance component incomplete. Never
record missing spend as zero. Correctness, runtime, and spend remain separate:
a fast or cheap incorrect result fails.

Freeze absolute runtime and spend ceilings or regression thresholds against the
last qualifying cycle before execution. The bootstrap cycle establishes the
comparison baseline but still requires absolute time and spend caps. A later
cycle that crosses either frozen threshold fails unless the product authority
approved the changed threshold before any lane ran.

## Verdicts

| Verdict | Definition | Release effect |
| --- | --- | --- |
| **Pass** | All three competitive lanes are auditable; Corbanu meets the live task rubric; the full coding task/model matrix is correct and auditable; runtime and spend stay within their frozen thresholds; and no P0 security failure is exposed. | Record the qualifying result. |
| **Fail** | Corbanu fails the live task or a required coding pair, crosses a frozen correctness/runtime/spend threshold, or exposes a P0 security failure. | Record the result and open corrective follow-up. |
| **Incomplete** | Any competitive lane or required coding task/model pair lacks a functioning harness, credential, provider route, task input, runtime, spend, or evidence. | Record the missing evidence and complete it in follow-up. |

Hermes or Kilo performing poorly on a functioning, auditable lane is a valid
competitive result; it does not make the run incomplete. Failed or incomplete
benchmark evidence must be disclosed accurately, but does not override an
explicit release instruction from a human with release authority.

## Bootstrap procedure

For the next release, the release owner must:

1. create `qa/release/<version>/benchmarks/`;
2. replace every pending field in the bootstrap ledger row;
3. freeze the competitive task, base commit, rubric, thresholds, and common
   cutoff;
4. freeze the coding task catalog, relevant model-set manifest, pricing source,
   and runtime and spend caps;
5. run and preserve all three competitive lanes and every required coding
   task/model pair;
6. enter the component results, final verdict, and evidence links below; and
7. set “releases since qualifying cycle” to zero only after both components
   pass.

The current pending row records unfinished benchmark work. It is not an
independent publication veto over an explicitly authorized release.

## Cadence ledger

| Release | Releases since qualifying cycle | Required | Owner | Live repo/task | Corbanu/Hermes/Kilo | Coding catalog/model set | Runtime | Spend | Verdict | Evidence |
| --- | ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Next release after policy adoption | baseline pending | yes | Release owner named in release record | pending | pending | full catalog; models pending | pending | pending | **Pending—must be disclosed** | pending |

## Evidence package

Store the following under `qa/release/<version>/benchmarks/`:

- candidate and harness versions/commits;
- repository origin, base commit, and disposable worktrees;
- frozen prompt, rubric, and Corbanu regression thresholds;
- coding task-catalog digest and per-task verifier versions;
- frozen relevant model-set manifest, including provider, model, reasoning,
  route, inclusion rationale, and approved exclusions;
- permissions and tools;
- common cutoff;
- timestamps, raw transcripts, final diffs, and test results for the competitive
  lanes and every coding task/model pair;
- end-to-end elapsed time, timeout state, token use, pricing source, actual or
  calculated spend, and frozen runtime/spend thresholds;
- per-pair results, component verdicts, and the final benchmark verdict.

Every public benchmark report must link to its cadence-ledger row here.
