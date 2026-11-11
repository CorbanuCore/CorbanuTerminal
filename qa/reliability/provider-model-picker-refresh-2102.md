# Provider recovery model-picker refresh (#2102)

Class: bounded fix. Product authority: `docs/corbanu-product-spec.md`, **Shipping MVP — LIVE**, **Multi-provider inference**: “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter … and custom providers.” No new activation, credential-storage, or account-selection behavior is introduced. No initiative plan/sprint is needed.

## Change

Provider status discovery updated the shared catalog but left a previously opened model picker displaying its old rows. Refresh the visible session model picker after correlated status and activation updates. Preserve the highlighted model by provider/model identity across single-provider and tabbed layouts. Refresh the parent when a reasoning dialog closes; never reopen a dismissed picker or replace the child dialog.

## Automated evidence

Worktree: `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-readd-model-refresh`, branch `fix/provider-readd-model-refresh`, base `152f62bf6e305d74c4deab00013520bb75d3636e`.

- `just fmt` and `git diff --check`: passed.
- `python3 scripts/release/release_notes.py check`: passed (64 fragments).
- `just test -p codex-tui --lib -E 'test(provider_picker_refresh) | test(model_catalog) | test(provider_model_policy) | test(model_picker) | test(tasknode_refresh_preserves) | test(list_selection_view) | test(reasoning_popup)' --locked --offline`: **82 passed**, 4,697 excluded by the focused filter, on macOS with Rust 1.99.1. Uses the repository's disposable-profile/native-keyring-denial wrapper.
- New app regression covers late OpenAI recovery while the picker is open, preservation of highlighted Claude identity and session configuration, child-dialog preservation and parent refresh, repeated refresh, provider removal, and single-Esc dismissal. Existing test covers stale generations and dismissed-manager behavior.
- Reviewed and accepted snapshot: `codex-rs/tui/src/app/tests/snapshots/codex_tui__app__tests__provider_picker_refresh__model_picker_openai_restored_while_open.snap`. The first test attempt stopped at the expected missing baseline; after review and acceptance, the broader run passed.
- Final follow-up rerun: **82 passed** after retaining the pending child-accept dismissal state across refresh. The regression also covers Enter → queued provider refresh → reasoning popup → effort acceptance → parent dismissed.
- Raw local logs: sibling files `provider-readd-tests.log`, `provider-readd-tests-2.log`, and `provider-readd-tests-3.log` beside the worktree. Compilation emitted the existing macOS large unwind-table linker warning; no Rust source warnings in the final run.

## Supporting interactive check

2026-11-11 04:39–04:42 UTC, macOS, real PTY keys, Corbanu 0.1.48 debug candidate SHA-256 `1050b24e2caa43a9a3604e08cf08ad0e71216e17d637fc56676a6b4ec0d730bf`, built from this worktree. Candidate path: `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target-dev-active/debug/corbanu`.

Isolated fixture: sibling directory `provider-readd-ui`. Empty inherited environment, all profile aliases set to that fixture, native-keyring debug denial, Seatbelt denial of network, personal home and live Corbanu profile reads, SecurityServer/securityd services and `/usr/bin/security`. Synthetic invalid OpenAI input only; no live account, network request, or keychain prompt. RUST_LOG=trace and logs stay inside the fixture.

Actual keys: `/model` then Enter; Esc; `/providers` then Enter; Enter on OpenAI; Down and Enter for API-key setup; synthetic input then Enter; wait for “Enabled · configured”; Esc; `/model` then Enter; Left to OpenAI. The models appeared alongside the unchanged fixture provider. Then `/providers` → OpenAI → Deactivate, `/model` (OpenAI absent); `/providers` → OpenAI → Reactivate, `/model` (OpenAI restored). Exited normally with Esc/Ctrl-C. Raw visible checkpoint is in `provider-readd-ui/openai-models-after-setup.txt`.

After the final child-dismissal follow-up, repeated the deactivate/reactivate/no-restart workflow in a second PTY on candidate SHA-256 `e6b18c4c538c81e4e9c28c5a633ddddf89f97d0caa0f2b549ec3411131387f63` (same isolated fixture, existing synthetic profile). Both withdrawal and reappearance passed, and the current fixture model remained selected. Checkpoint: `provider-readd-ui/openai-models-final-reactivation.txt`. This supersedes the earlier binary for the final activation-flow check; it does not relabel the earlier setup run.

This is implementer-operated supporting TUI evidence, **not independent code-blind acceptance or live OpenAI inference qualification**. It verifies configuration/menu behavior, not credential validity. No operator profile was modified, no installed binary was replaced, and no release was published by this check.

## Independent review and remaining qualification

- Fresh-context functional design: Astra High, Volta, session `01a1c8dd-8293-7b82-afc6-ff0430ce19d9`; frozen response at sibling `provider-readd-functional-cases.md`. Eleven cases preserved; not all are executed or accepted. Designer had instruction-only isolation and used no tools.
- Code review: Astra High, Chandrasekhar, session `01a1c8e0-7f2e-7050-b588-234fe1ecf672`. First pass identified focus preservation and child-dialog refresh gaps, both addressed, plus the then-pending snapshot baseline. Second pass found a pending-child-dismissal race; it was fixed with a corresponding regression. Third pass approved with no remaining must-fix findings. Four opinion reviews total including independent design; final tests passed afterward.
- No named-human acceptance, independent sandboxed functional execution, cross-platform run, live inference, release-wide TensorCash/Isometric Game checks, or demo reel is claimed. Those remain release qualification/backlog work; this narrow picker fix does not claim a qualified release.
