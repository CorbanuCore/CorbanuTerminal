---
sprint_id: "PF-84-S01"
title: "Home override hygiene for per-account workers"
status: ready
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 23
owner: "Codex PF-84 home-hygiene worker"
parallel_lane: "home-hygiene"
write_scope: "codex-rs/utils/home-dir/, scripts/install/install.sh, scripts/install/test_install_sh.py, scripts/dev/corbanu-launcher.sh, scripts/dev/test_corbanu_launcher.sh, docs/authentication.md, qa/demos/specs/pf84-home-override.toml, qa/demos/specs/pf84-home-override-openai.toml, qa/demos/index/PF-84-S01.md, qa/provider-auth/pf-84/s01-gate.md, docs/sprints/current/unified-provider-auth/pf-84-s01-home-override-hygiene.md"
integration_gate: "Codex PF-84 lane owner merges to main after just test -p codex-utils-home-dir, python3 -m pytest scripts/install/test_install_sh.py, sh scripts/dev/test_corbanu_launcher.sh, RTX clippy and the tmux/GLM run"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s01-home"
branch: "feat/pf-84-s01-home-override"
base_commit: "051f9747225776a5d85ad00c2e5d0a8f5f4036bf"
depends_on: "none"
created: 2026-10-08
updated: 2026-10-10
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

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md)
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

- [x] Plan is active.
- [x] Dependencies are completed.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Conflict warning in `find_codex_home`, once per process, paths only.
- [x] Both wrappers keep a caller-set home; `scripts/dev/corbanu-launcher.sh` checked in; documented in `docs/authentication.md`.
- [x] Regression: a launcher worker with `CORBANU_HOME=homeB` uses homeB's account (fake OpenAI key -> 401).
- [x] Focused tests (home-dir, installer wrappers, launcher shell test); RTX Linux clippy clean.
- [x] tmux + GLM 5.3 Flash runs with real keys sent; OpenAI second-provider run; videos published.
- [x] Independent Opus 5.5 High review; findings dispositioned in the [gate evidence](../../../../qa/provider-auth/pf-84/s01-gate.md).

## Remaining

- [ ] Independent code-blind functional design and execution (acceptance step, not the implementer).

## Verification

- [x] Focused test: `just test -p codex-utils-home-dir` plus the wrapper shell test.
- [x] tmux via `scripts/demo_video.py` (`docs/tmuxHarness.md` rules): disposable homes A and B,
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`; doctor output shows home B; text and Enter sent separately.
- [x] GLM run on `-m glm-5.3-flash -c model_provider="zai"` with the vault-helper ZAI key.
- [x] Videos: `pf84-home-override`, `pf84-home-override-openai` (index `qa/demos/index/PF-84-S01.md`).
- [x] One independent review (Opus 5.5 High); findings dispositioned.
- [ ] Independent code-blind acceptance run linked (then Travis sign-off).

## Security notes

- Debug candidates never touch the real login keychain: every run uses
  disposable homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- The warning prints paths only, never vault contents or fingerprints.

## Exit evidence

- [x] Implementation commit and PR recorded (code `84f2797b62`; PR in the gate evidence).
- [x] Final-tree test output, tmux log and video path linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
