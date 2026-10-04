# Opus 5.5 Claude Plan local debug candidate

User instruction (2026-09-22): install `corbanu-debug --yolo` from Travis's merged model updates, and include Opus 5.5 in Claude Plan as well.

Class: bounded fix to existing model selection. Product authority: **Shipping MVP — LIVE**, **Multi-provider inference** — “OpenAI, Anthropic/Claude Plan … and custom providers.” Adds an exact supported subscription model to the existing authentication/transport flow; no new auth mechanism or credential storage, purchase, default-model change, or public release.

Base: `0f18b63404` (`origin/main`), which includes GPT-6 Sol and the Anthropic API Opus 5.5 entry. Isolated branch: `fix/opus-5-5-claude-plan-20260922`. Worktree: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-debug-main-20260922`. Dedicated target created with `prepare-cargo-storage`; canonical checkout and stable installation unchanged.

Plan route: `claude-plan` / `claude-opus-5-5-plan` → `claude-opus-5-5` upstream. Bare API row remains owned by `anthropic`. Explicit `claude-plan` with the bare model normalizes to the exact plan alias. The new alias receives subscription identity/cache handling and the existing adaptive effort mapping. No fallback alias to Opus 5 is introduced. Plan orchestration weight inherits the existing Opus Plan internal weight, not a claim about vendor subscription pricing.

Independent starting-state/action/outcome design is frozen in `TEST_DESIGN.md`. This is a local testing handoff, not complete release qualification. No public release or Git push is authorized by this change record.

Upstream preflight: existing Claude Code Max login reported authenticated; `claude -p --model claude-opus-5-5` with no tools returned `OPUS55_OK` and structured usage under the exact `claude-opus-5-5` model. This confirms account access; it does not replace testing Corbanu's own request path.
