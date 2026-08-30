# Compatibility/drift lane handoff

Sprint: PF-21-S02, currently `ready`.

Owner: Codex compatibility/drift lane. Integration owner: Codex
ingress/classifier lane.

Work only in `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift`
on `feat/p0-security-compatibility-drift`, allocated from
`5521b681fff0ecb50b17c10bc1dd1356cbecc1b6`. Put all build and temporary output
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/`.

## Authority and scope

This is a live allocation: set PF-21-S02 to `in_progress`, run both governance
checkers, then implement. No additional worktree, branch, owner, or G0 decision
is missing.

Literal scope:

```text
scripts/security-level-compat
scripts/security_level_compat.py
scripts/test_security_level_compat.py
qa/security-levels/compatibility/
qa/security-levels/sprints/PF-21-S02/
docs/sprints/current/p0-security-levels/pf-21-s02-expanded-compatibility-and-upstream-drift.md
```

This round is deliberately scripts-and-evidence-only. Do not edit the immutable
`qa/security-levels/permissive-baseline-v1.json`, Rust/Core/Vault code, manifests,
locks, plan/index/MkDocs, or another sprint. A discovered product regression is
evidence for a later scoped repair, not authority to modify runtime code here.

## Deliverable

Preserve the accepted baseline and pre-feature commit
`3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`. Add an independently constructed
upstream-aligned control and a reviewed drift ledger. Expand executable inventory
to environment/auth helpers, web history/native search, browser, MCP/plugins,
children, wallet, clipboard/export, and persisted sessions. Broker, screening,
secretless launch, and migration must remain opt-in above Permissive.

Record baseline/upstream/candidate commits, configuration/environment digests,
exact probes, and owner-reviewed intentional differences. Unknown drift fails;
never regenerate expectations from the candidate. Add self-tests for missing
surfaces, mismatched identities, candidate-derived expectations, stale evidence,
and failures in every new case.

## Verification and review

Format/lint the Python changes, run
`python3 -m unittest scripts.test_security_level_compat -v`, execute the extended
harness against the independent controls, run both governance checkers, and
`git diff --check`. Record actual CLI arguments, artifact hashes, and counts.

Use TMUX + Corbanu Terminal + Claude Opus 5.0 Max for independent review. Ask
about oracle contamination, candidate-derived expectations, incomplete surface
inventory, environment nondeterminism, stale upstream controls, hidden opt-in
changes, false drift acceptance, and secret leakage into evidence. Repeat until
clean.

Hand back the candidate/base, immutable-oracle hash, control identities, drift
ledger, scope audit, exact tests, limitations, and transcript hash. The
integration owner merges this after the two contract lanes and archives it before
PF-22-S02 allocation.

