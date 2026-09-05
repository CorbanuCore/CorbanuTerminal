---
sprint_id: "PF-55-S04"
title: "Mixed-model native subagent runtime convergence"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-55"
execution_order: 21
owner: "Codex primary subagent runtime integration agent"
parallel_lane: "subagent-runtime"
write_scope: "codex-rs/core/src/tools/handlers/multi_agents_common.rs, codex-rs/core/src/tools/handlers/multi_agents_spec.rs, codex-rs/core/src/tools/handlers/multi_agents_spec_tests.rs, codex-rs/core/src/tools/handlers/multi_agents_tests.rs, codex-rs/core/src/tools/handlers/multi_agents_v2.rs, codex-rs/core/src/tools/spec_plan.rs, codex-rs/core/src/tools/spec_plan_tests.rs, codex-rs/core/tests/suite/spawn_agent_description.rs, scripts/subagent_tui_acceptance.py, scripts/test_subagent_tui_acceptance.py, docs/features/model-providers.md, docs/plans/active/unified-provider-auth.md, docs/sprints/index.md, docs/sprints/current/unified-provider-auth/, docs/sprints/archive/unified-provider-auth/, qa/release/0.1.38/"
integration_gate: "Primary agent audits the exact runtime and provider boundaries, runs final affected Core tests and real TUI child launch/result, failure/recovery and resume checks, records binary identity, and installs the verified debug candidate without interrupting unrelated sessions or publishing a release."
worktree: "/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile"
branch: "integration/reconcile-release-0.1.38"
base_commit: "43f4f187ba585e231b0bafed2bbcd9d9b4bffa54"
depends_on: "PF-55-S03"
created: 2026-09-05
updated: 2026-09-05
---

# PF-55-S04 — Subagent runtime convergence

## Execution mandate

- Deliver exact authorized Luna and Kimi K3 child launches from the native
  orchestration engine, fixing the general runtime boundary, not model names.
- Excludes new providers/auth, automatic allocation changes, pricing, release
  publication, credential copying and changes to the user's live human thread.

## Plan linkage

- [Unified provider authentication](../../../plans/active/unified-provider-auth.md), PF-55.
- Product: **Shipping MVP — LIVE**, “Sauron → Nazgul → Troll → Orc orchestration,
  model-aware delegation, durable mailboxes, supervision, resume, and recovery.”

## Code boundaries

- Native spawn runtime config/model selection and generated bounded catalog in
  `multi_agents_common.rs` and `multi_agents_spec.rs`; colocated regression files.
- `codex-rs/core/src/tools/spec_plan.rs`: remove the obsolete catalog-version
  option at its two construction sites; no provider-policy changes.
- `multi_agents_v2.rs`: live testing proved the sole exact-runtime adapter rejects
  OpenAI recipients, while the reserved native schema has no selection fields.
  Admit explicitly plaintext assignments to OpenAI too, preserving native
  ciphertext and refusing native encrypted payloads to external providers.
- Request-boundary integration in `core/tests/suite/spawn_agent_description.rs`.
- Explicit opt-in live TUI acceptance script and versioned evidence.

## Preconditions

- [x] Active plan and completed PF-55-S03 dependency verified.
- [x] Clean worktree and exact base recorded; no overlapping active write scope.
- [x] Root/Rust/Core policies and development, test-tui, remote-tests skills read.

## Done

- [x] Reproduced metadata mismatch: remote Luna prefers V1, Kimi defaults unset;
  the V2 catalog rejects both even with the user's V2 runtime override enabled.
- [x] Repaired discovery, selection, V2 inheritance after role loading, and the
  explicit plaintext adapter's OpenAI-recipient dead end without model-name cases.
- [x] Final affected Rust tests pass 180/180; harness checks pass 19/19.
- [x] First complete live run passes in both disposable repositories, including
  exact child tool/results, parent cancellation/recovery and cold resume/followup.
- [x] Installed binary `bdc666c3098d48030e8474173ae53b169708a803dbac1b92088c44ce42e88ff1`
  behind `corbanu-debug`; a fresh `corbanu-agents` session preserves the old session.

## Remaining

- [ ] Record the installed-binary rerun, finished docs and source identity; archive.

## Verification

- [x] `just fmt`; focused `just test -p codex-core` after formatting.
- [x] Native outbound-request integration with `build_with_auto_env`.
- [x] True TUI with separate prompt/Enter actions; no fallback models.
- [x] Exact child provider/model and completed responses verified from structured events.
- [x] Provider policy and no-regex LLM-path boundary preserved.

## Exit evidence

- [ ] Candidate source identity, binary hash and artifact paths recorded.
- [ ] Actual scope and final-tree test evidence linked; no unrelated edits.
- [ ] Named-human acceptance and remaining release gates accurately disclosed.
- [ ] Done/Remaining ledgers current; completed record archived only after proof.
