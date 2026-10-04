# GPT picker cleanup and September 22 model update

Class: bounded fix to existing model catalogue/selection. Product authority: **Shipping MVP — LIVE**, **Multi-provider inference** — “OpenAI, Anthropic/Claude Plan … and custom providers.” Same isolated worktree/branch/base as CHANGE.md. No release or default-model migration.

User asked to remove 5.5 from the selector and check today's OpenAI models. Interpretation explicitly communicated: remove **OpenAI GPT-5.5**, retain **Claude Opus 5.5**. GPT-5.5 metadata remains usable by explicit model ID and existing sessions. Bundled hidden models now remain hidden across remote/cache overlays generally, replacing a one-model retired-Ambient exception. Remote hiding of otherwise visible models still works.

Official sources fetched on 2026-09-22:
- https://learn.chatgpt.com/docs/changelog — September 22 entry announces GPT-6 Sol and GPT-6 Luna for Codex and ChatGPT Work; rollout/account/workspace eligibility applies.
- https://developers.openai.com/api/docs/models/gpt-6-luna — exact ID gpt-6-luna; text/image input, 1,050,000 context, 128,000 output, none/low/medium/high/xhigh/max effort (medium default), Responses tool support. Standard token pricing $0.10 input/$0.01 cached/$0.50 output per million (other processing/long-context conditions apply).

Added GPT-6 Luna using the existing GPT-6 Responses transport/tool contract, exact model ID, verified limits and effort choices. Automatic allocation remains disabled for the new model until plan economics are configured, as for other manual-only models; no invented subscription-burn value. Main already includes GPT-6 Sol. No unverified GPT-6 Terra entry introduced. Existing GPT-5.6 entries retained.

Independent test cases: SELECTOR_TEST_DESIGN.md. Validation results will be recorded after final build and actual-profile TUI verification. Benchmark and public-release scope remain paused.
