# Worker brief: stop non-OpenAI providers from collapsing distinct skills catalogs

You are an Opus 5.5 High worker. Work autonomously; do not ask questions. Travis approved this fix.

## Problem
- `codex-rs/core/src/client_common.rs::retain_latest_contextual_developer_fragments` (called for non-OpenAI providers
  in `core/src/client.rs` at the Responses, Chat and Anthropic request builders) keeps only the newest developer
  content item per *tag*, keyed by `core/src/event_mapping.rs::contextual_dev_fragment_key` over
  `CONTEXTUAL_DEVELOPER_PREFIXES`. The intent is good: world-state/context fragments are re-sent when they change,
  and old copies shouldn't accumulate.
- But up to three different sources render with the same `<skills_instructions>` tag:
  1. executor catalog — world-state section `SKILLS_WORLD_STATE_ID = "skills"` (`ext/skills/src/world_state.rs`);
  2. host catalog — world-state section `HOST_SKILLS_WORLD_STATE_ID = "host_skills"` (same file);
  3. orchestrator catalog — thread context from `ext/skills/src/extension.rs::contribute_thread_context` via
     `core/src/context/available_skills_instructions.rs` (enabled only when the thread has no local environment).
  When two are present, the dedupe treats them as versions of one list and drops the older; the model loses a
  catalog. Default provider Ambient (non-OpenAI) is affected. Tests in `app-server/tests/suite/v2/mcp_resource.rs`
  (`orchestrator_skill_can_read_referenced_resource_without_an_executor`) and siblings were switched to the OpenAI
  provider to dodge this (commit `132fbe4f98`; see comments mentioning `retain_latest_contextual_developer_fragments`).

## Do
1. Make the dedupe key identify the *source*, not just the tag: repeated copies of the same source still collapse to
   the newest; different sources sharing a tag are all kept. Find the cleanest stable discriminator (e.g. the
   world-state section id the fragment was rendered for, or a source marker carried on the content item / message
   metadata), without changing what OpenAI receives and without changing the text the model sees beyond keeping the
   missing catalog. Check every other tag in `CONTEXTUAL_DEVELOPER_PREFIXES` for the same multi-source issue and
   handle them the same way if they have it. Keep legacy persisted history (older rollouts without the new
   discriminator) working: they should behave as today.
2. Tests: unit tests in `client_common_tests.rs` (two sources same tag kept; same source repeated collapses; legacy
   items unchanged), plus switch the app-server tests that were moved to the OpenAI provider for this reason back to
   the default (non-OpenAI) provider so they cover the real path, and make them pass. Run codex-core and
   app-server suites for the touched areas, `just fmt`, clippy for touched crates. Linux verification on the RTX box
   (`ssh ambient@100.99.88.49`, fresh clone under `~/skills-dedupe-20261004/`, TMPDIR on ext4, cleaned up after).
3. Independent review with `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c
   model_reasoning_effort="high"`; fix findings.
4. Branch `codex/skills-catalog-dedupe-20261004` in a new worktree off `origin/main`. PR #155 is also touching core
   (tests); if it merges first, merge main in. Open a PR to main; report when checks pass. Don't merge.
5. Also push the same change to `integrate/management-workstreams-20260911` only if it applies cleanly as a merge of
   main later — don't cherry-pick it there yourself.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't change CI timeouts.

## Final report (your last message)
Root cause, the discriminator chosen and why, tests before/after, review verdict, PR URL and check status.
