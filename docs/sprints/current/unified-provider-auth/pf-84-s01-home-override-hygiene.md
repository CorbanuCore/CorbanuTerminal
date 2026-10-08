---
sprint_id: "PF-84-S01"
title: "Home override hygiene for per-account workers"
status: draft
plan_file: "docs/plans/proposed/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 23
owner: "UNALLOCATED"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-10-08
updated: 2026-10-08
---

# PF-84-S01 — Home override hygiene for per-account workers

## Execution mandate

- Deliver: a worker started with an explicit home lands on that home, and a
  conflicting home environment is reported instead of silently ignored.
- Fixes the multiacct1 pitfall: `~/.local/bin/corbanu` sources `activate.sh`,
  which re-exports `CORBANU_HOME` and `CODEX_HOME` to the real home, so
  `CODEX_HOME=<other> corbanu ...` silently used account A (measured: the
  wrapper returned the real home's token fingerprint).
- Excludes: named accounts themselves (PF-84-S02/S03). Separate homes stay an
  advanced workaround, not the product path.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/proposed/unified-provider-auth.md)
- Feature: `PF-84`
- Acceptance advanced: "Wrapper/home override" flow.

## Code boundaries

- Existing: `codex-rs/utils/home-dir/src/lib.rs:17-50`: precedence is
  `CORBANU_HOME` > `PFTERMINAL_HOME` > `CODEX_HOME`, so an inherited
  `CORBANU_HOME` silently beats a caller's `CODEX_HOME`.
- Existing: `scripts/install/install.sh:1123-1145`: the release wrapper defaults
  only `CODEX_HOME`. It must never override a home the caller set.
- Existing (outside the repo): the Mac dev launcher `~/.local/bin/corbanu` and
  `.codex-work/corbanu-terminal/activate.sh`, which export the homes unconditionally.
- Planned: check the dev launcher into `scripts/dev/` so it gets reviewed. It
  keeps a caller-set `CORBANU_HOME`/`CODEX_HOME` after sourcing `activate.sh`.
- Planned: one stderr warning when the home variables name different paths,
  naming the variable that wins. Precedence itself does not change.
- Tests: `utils/home-dir` unit tests; a shell test for both wrappers.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.

## Remaining

- [ ] Add the conflict warning in `find_codex_home` (paths only, no secrets).
- [ ] Make both wrappers preserve a caller-set home; document in `docs/authentication.md`.
- [ ] Regression: the wrapper with `CORBANU_HOME=<home2>` resolves home2 (fingerprint check).
- [ ] Code-blind functional design frozen before test-result disclosure, or non-user-facing N/A reason recorded.

## Verification

- [ ] Focused test: `just test -p codex-utils-home-dir` plus the wrapper shell test.
- [ ] tmux (`docs/tmuxHarness.md`, `test-tui` skill): disposable homes A and B,
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`. Start the wrapper with home B, check
  the status line shows home B, and send text and Enter separately.
- [ ] GLM run: the same flow driven by `-m glm-5.2 -c model_provider="zai"`. The ZAI
  key comes only from `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"`.
- [ ] Video: `python3 scripts/demo_video.py record qa/demos/specs/pf84-home-override.toml --bin <candidate> --sprint PF-84-S01 --publish`.
- [ ] One independent review (Opus 5.5 High); findings dispositioned.

## Security notes

- Debug candidates never touch the real login keychain: every run uses
  disposable homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- The warning prints paths only, never vault contents or fingerprints.

## Exit evidence

- [ ] Implementation commit and PR recorded.
- [ ] Final-tree test output, tmux log and video path linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
