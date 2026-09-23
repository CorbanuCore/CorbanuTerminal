# Independent selector cleanup test design

Frozen before implementation, 2026-09-22. Author: independent code-blind agent `selector_cleanup_test_design`. Intent: remove OpenAI GPT-5.5 from the picker only; retain Claude Opus 5.5 and other configured providers.

1. Fresh/default configuration: actual-key model picker has no GPT-5.5, retains Claude Opus 5.5 Plan and other configured providers/models.
2. Existing saved GPT-5.5: restart/open picker, no selectable GPT-5.5; do not silently replace saved ID.
3. Stale cache advertises GPT-5.5: cancel/reopen/restart must not resurrect it; preserve non-OpenAI entries.
4. Explicit model ID remains accepted/reaches backend; removal is selector-only.
5. Cancel/restart leaves selection intact; deliberate supported-model selection persists after restart.
6. Verify actual existing user profile/provider visibility, not just an alternate QA profile; preserve credentials/config.
7. If picker has search/filter, exact-name search cannot expose a selectable GPT-5.5; Escape exits cleanly.

Nuance from independent reviewer: a saved explicit GPT-5.5 ID remains valid; no silent migration upon opening/cancelling the picker. Extra isolated test fixtures are appropriate for saved-model persistence; do not substitute them for existing-profile verification.

Scope amendment: official September 22 OpenAI changelog confirms GPT-6 Luna alongside already-included Sol. Add exact Luna catalogue entry and verify actual selection/response/restart; no claim of availability on every subscription.

Independent add-on cases (same reviewer, before final build/QA): exact Luna row under OpenAI; selection/save/restart persists; cancel after highlighting Luna preserves prior selection; explicit Luna ID accepted while GPT-5.5 compatibility remains; stale cache lacking Luna does not suppress the new bundled row or create duplicates; all existing providers remain visible.
