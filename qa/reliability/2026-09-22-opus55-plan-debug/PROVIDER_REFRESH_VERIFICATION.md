# Provider refresh verification — 2026-09-22

Companion to PROVIDER_REFRESH.md; local debug install, not a public release. FlashX remains pending explicit approval for a separate paid Z.AI API route. No FlashX Coding Plan support is claimed.

## Installed candidate

`~/.local/bin/corbanu-debug` launches the `0f18b63404-provider-refresh` package under `/mnt/HC_Volume_101713660/pfrpc/build-cache/corbanu-debug-installs/`, with its matching code-mode host. Default profile remains the normal `~/.corbanu`; explicit CODEX_HOME overrides are respected. Source worktree and prior changes are preserved. Version remains 0.1.42; binary identity is recorded in `provider-refresh-binaries.sha256` in the evidence directory, not inferred from the version string.

Evidence directory: `/home/pfrpc/corbanu-debug-evidence`.

## Automated checks

- 133/133 model-manager and provider-info tests: `provider-refresh-model-final.log`.
- 3/3 targeted core tests: `provider-refresh-core-final.log` (exact routes/reasoning, preserved Z.AI tool reasoning, saved effort persistence).
- 10/10 targeted TUI/crew tests: `provider-refresh-tui-accepted.log`; snapshots reviewed after correcting the OpenRouter fixture and price formatting.
- `just fmt`, final build, and `git diff --check` passed. Build log: `provider-refresh-build-final.log`.

## Interactive and upstream checks

Final installed wrapper was launched in a true TUI with the normal profile. Actual `/model` navigation showed all four new OpenRouter options, retired options absent, and Z.AI Flash present alongside existing Z.AI models. Unrelated provider tabs remained available. Captures: `provider-final-openrouter-top.txt`, `provider-final-openrouter-bottom.txt`, `provider-final-zai.txt`.

Five generic upstream readiness probes returned HTTP 200 and exact requested model IDs (`provider-preflight-results.json`). Four real Corbanu tool loops executed harmless shell commands: Z.AI Flash, OpenRouter Flash, Grok 4.7, and DeepSeek 4.1 Flash. Those initial loops used the matching inference code before a label-only price correction. Corresponding `*-tool-loop.txt` and `*-request.json` files record the results. Nemotron Free was checked with generic upstream prompts/tool calls only, not a Corbanu session carrying user workspace context (`provider-tool-preflight-results.json`).

Final installed package also completed a Z.AI Flash shell-tool loop: exact model `glm-5.3-flash`, reasoning effort `low`, one actual tool result in the final request. Important limitation: the first prompt produced the expected text without executing the shell. A correction explicitly requiring the tool caused `printf FINAL_FLASH_OK` to execute, followed by the actual output. This is an instruction-following miss followed by successful tool transport, not an unqualified first-attempt pass. Evidence: `provider-final-live-tool-loop.txt`, `provider-final-live-request.json`.

Normal config, provider eligibility, stable launcher, and stable binary checksums were unchanged (`provider-refresh-after.txt`). The checksum manifest uses home-relative paths; verify it from `/home/pfrpc`. A verification invocation from the source worktree could not resolve those paths; rerunning from home passed all four entries. QA sessions were exited without changing user defaults.

## Independent review and limitations

Independent code-blind design and subsequent evidence review were provided by `provider_refresh_test_design`. Review supports the model/core tests, selector evidence, exact-route preflights, and compatibility-only retention of retired metadata. Retired entries are hidden and disabled for automatic allocation, rather than deleted from explicit-ID compatibility metadata.

FlashX is not implemented or verified. It is a separate API-only model, excluded from the Coding Plan by current official documentation. No silent paid fallback was added. No benchmark quality, supervision reliability, comprehensive failure/retry coverage, or full release qualification is implied by these focused tests. No public release was pushed.
