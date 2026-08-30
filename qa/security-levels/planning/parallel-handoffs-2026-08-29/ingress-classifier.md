# Ingress/classifier lane handoff

Owner: a separately named ingress/classifier agent; integration owner Jim
Ricketts controls shared surfaces, blind-evaluation custody, and merges unless a
different evaluator is explicitly named.

## Coordinates and authority gate

- Proposed worktree:
  `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-ingress-classifier`
- Proposed branch: `feat/p0-security-ingress-classifier`
- Parallel lane: `ingress-classifier`
- Build/cache root:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`
- Dispatch base: exact 40-character post-handoff `main` commit. The analysis
  baseline was `a753283f9cd1a59ff2ae3b03319c3c4a3264326f`; re-pin if main advances.

Put `CARGO_HOME`, `CARGO_TARGET_DIR`, `TMPDIR`, `UV_CACHE_DIR`, `PIP_CACHE_DIR`,
and all model/corpus caches beneath that CorbanuDrive cache root.

This document does not activate a sprint. Before implementation, update the plan
and owning sprint with a named owner, exact coordinates, literal scope, and
integration gate. Both governance checkers must pass before `ready`.

Product citations:

- **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected
  paths fail visibly rather than falling back to raw secrets or unscreened
  execution.”
- **Non-negotiable controls** — “Classify instruction intent and provenance
  before external content can influence tools or financial actions.”

## Sequence and stop points

Early preparation chain, strictly one at a time:

1. PF-34-S04 — screening segment contract and fixtures.
2. PF-35-S01 only after PF-34-S04 is integrated, archived, and its contract and
   fixture hashes are frozen.
3. PF-35-S02 only after PF-35-S01 is integrated, archived, and its evaluator
   split and independent custody are frozen.
4. Return the active slot. These sprints create no protected ingestion route.

Later re-entry, only after all listed dependencies are completed and archived:

```text
PF-31-S02 + PF-30-S01 + PF-34-S04 -> PF-34-S01
PF-35-S02 + PF-34-S01 + PF-30-S03 + PF-23-S01 -> PF-35-S03
PF-35-S03 + PF-41-S03 -> PF-34-S02
PF-34-S02 + PF-24-S01 -> PF-34-S03
```

## Literal early scopes

PF-34-S04:

```text
codex-rs/content-security/src/contract.rs
codex-rs/content-security/src/contract_tests.rs
qa/security-levels/ingress-contract/
qa/security-levels/sprints/PF-34-S04/
docs/sprints/current/p0-security-levels/pf-34-s04-screening-contract-and-fixtures.md
```

PF-35-S01, after PF-34-S04 archive:

```text
codex-rs/content-security/src/evaluation.rs
codex-rs/content-security/src/evaluation_tests.rs
scripts/security-classifier-eval
qa/security-levels/classifier/corpus-manifest.json
qa/security-levels/classifier/split-manifest.json
qa/security-levels/sprints/PF-35-S01/
docs/sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md
```

PF-35-S02, after PF-35-S01 archive:

```text
codex-rs/content-security/src/classifier.rs
codex-rs/content-security/src/classifier_tests.rs
tools/security-classifier/
qa/security-levels/classifier/artifact-manifest.json
qa/security-levels/sprints/PF-35-S02/
docs/sprints/current/p0-security-levels/pf-35-s02-local-cpu-detector-artifact.md
```

Exclude content-security `lib.rs`, crate/workspace Cargo and Bazel files, locks,
root registries, shared Core/TUI paths, plan/index/nav, and other lane scopes.
PF-27-S03 and PF-34-S04 new-crate registration is serialized at G1.

## Deliverables and gates

PF-34-S04 freezes immutable content/source/transformation binding, segment
index/count/reassembly, size/time budgets, and typed
allow/suspicious/hostile/unavailable verdicts with model/threshold identities.
Missing, malformed, stale, or mismatched verdicts are unavailable; no unexamined
prefix is released. Fixtures cannot authorize tools or clear taint.

PF-35-S01 inventories licensed data and freezes leakage-resistant grouped splits,
independent blind ownership, unseen-source/language/topic/position/adaptive
holdouts, hard benign negatives, confidence intervals, family metrics, FPR, and
weakest-supported CPU methodology. The implementation agent must not tune on
blind labels.

PF-35-S02 pins model, tokenizer, runtime, licenses, seed/config, signature and
hashes, CPU features, distribution method, repeatable export, offline behavior,
memory/latency envelope, and missing/corrupt/timeout/resource-exhaustion failure.
It returns typed unavailable, never benign, on unsupported or broken execution.

## Verification and review

Run `just fix -p codex-content-security`, `just fmt`, the focused named sprint
tests, the full content-security suite, Bazel parity when manifests change, both
governance checkers, and `git diff --check`. Record actual test counts and safe
artifact identities. PF-34-S04 additionally covers malformed, partial, duplicate,
digest/version mismatch, cancellation, timeout, forced-allow, and no-prefix-
release cases. PF-35-S01 records source/license and split/leak audits with CPU
feasibility. PF-35-S02 covers reproducibility, signature/hash/tokenizer mismatch,
offline, unsupported CPU, exhaustion, RSS/latency, and shutdown cleanup.

Follow the common Claude Opus 5.0 Max Computer Use protocol. Ask explicitly about
fail-closed verdict handling, digest/source/version/reassembly binding, prefix
release, provenance/taint becoming authority, corpus leakage and licenses,
artifact reproducibility and offline guarantees, resource exhaustion, and
unsupported CPU behavior. Never expose private corpus data, blind labels, or
signing keys to the reviewer.

Later PF-34-S03 requires true tmux TUI evidence with actual keys; send text and
Enter separately. Shared bottom-pane registration and snapshots serialize with
PF-31-S03 and remain integration-owner-only.

## Blockers and handback

The weakest supported CPU, licensed corpus sources/hashes/permitted uses, blind
evaluator ownership, model/tokenizer/runtime/licenses, reproducible build inputs,
signature root, and artifact distribution are unresolved. An available machine
does not prove it is the weakest supported CPU: record CPU model, architecture,
features, RAM, OS, and cold/warm method, or obtain a qualifying host. The
100,000-benign held-out/FPR requirement is long-lead work. Large artifacts must
not be committed without an explicit release-asset/LFS/package and signing
decision.

Hand back candidate/base commits, literal scope audit, schema/fixture/artifact
hashes, source/license inventory, evaluator custody record, commands/counts,
hardware matrix, limitations, immutable review evidence, and recommended shared
registration changes. The integration owner performs shared edits, combined-tree
reruns, plan/nav updates, and archives.

